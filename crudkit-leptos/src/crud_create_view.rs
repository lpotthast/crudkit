use crate::crud_fields::CrudFields;
use crate::crud_instance::CrudInstanceContext;
use crate::crud_instance_config::{CreateElements, FieldRendererRegistry};
use crate::crud_leave_modal::CrudLeaveModal;
use crate::ReactiveValue;
use crudkit_core::{Saved, Value};
use crudkit_core::id::{SerializableId, SerializableIdEntry};
use crudkit_web::prelude::*;
use crudkit_web::request_error::{CrudOperationError, RequestError};
use crudkit_web::{FieldMode, TabId};
use leptonic::components::prelude::*;
use leptonic::prelude::*;
use leptos::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Then {
    OpenEditView,
    OpenListView,
    OpenCreateView,
}

// TODO: Make this a signal? How would we act upon changes?
fn default_create_model(ctx: &CrudInstanceContext) -> DynCreateModel {
    let mut entity = ctx
        .static_config
        .read_value()
        .model_handler
        .get_default_create_model
        .run(());

    if let Some(parent) = ctx.parent.get_value() {
        if let Some(parent_id) = ctx.parent_id.get_untracked() {
            let SerializableIdEntry {
                field_name: _,
                value,
            } = parent_id
                .entries()
                .find(|entry| entry.field_name.as_str() == parent.referenced_field.as_ref())
                .expect("related parent field must be part of the parents id!");
            ctx.static_config
                .read_value()
                .model_handler
                .get_create_model_field
                .run(parent.referencing_field)
                .set_value(&mut entity, value.clone().into());
            tracing::info!("successfully set parent id to reference field");
        } else {
            tracing::error!(
                "CrudInstance is configured to be a nested instance but no parent id was passed down!"
            );
        }
    }
    entity
}

/// The create view shows the form with which the user can CREATE a new entity of the given resource.
/// NOTE: The instance configuration allows to specify the fields shown when updating the entity (required)
/// as well as specifying the fields shown when creating the entity (optional).
/// If the model for creating and updating an entity is the same, the user may only specify the required fields for updating.
/// These fields are then also used for creation, requiring this component to be able to work with the create and the update model!
/// This component decides on its own, depending on the instance configuration, which fields to display.
#[component]
pub fn CrudCreateView(
    #[prop(into)] data_provider: Signal<DynCrudRestDataProvider>,
    #[prop(into)] create_elements: Signal<CreateElements>,
    #[prop(into)] field_renderer_registry: Signal<FieldRendererRegistry<DynCreateField>>,
    #[prop(into)] on_edit_view: Callback<SerializableId>, // UpdateModel id
    #[prop(into)] on_list_view: Callback<()>,
    #[prop(into)] on_create_view: Callback<()>,
    /// Called when the entity is successfully created.
    #[prop(into)]
    on_entity_created: Callback<Saved<DynUpdateModel>>,
    /// Called when entity creation fails for any reason (permission denied, validation errors, server error, etc.).
    /// Use pattern matching on `CrudOperationError` to handle different failure types.
    #[prop(into)]
    on_entity_creation_failed: Callback<CrudOperationError>,
    #[prop(into)] on_tab_selected: Callback<TabId>,
) -> impl IntoView {
    let ctx = expect_context::<CrudInstanceContext>();

    let default_create_model = default_create_model(&ctx);

    let signals: StoredValue<HashMap<DynCreateField, ReactiveValue>> = // CrateModel field
        StoredValue::new(
            ctx.static_config
                .read_value()
                .model_handler
                .create_model_to_signal_map
                .run(default_create_model.clone()),
        );

    // The CreateModel enforces a `Default` value! We cannot deserialize a loaded model, so we have to create one from scratch with which the UI can be initialized.
    // We therefore do not have to deal with None states in the create case, compared to the edit view.
    // All modifications made through the UI are stored in this signal.
    let (input, set_input) = signal(default_create_model.clone());

    let input_changed = Signal::derive(move || input.get() != default_create_model);

    // The state of the `input` signal should be considered to be erroneous if at least one field is contained in this error list.
    let (_input_errors, set_input_errors) = signal(HashMap::<DynCreateField, String>::new());

    let (user_wants_to_leave, set_user_wants_to_leave) = signal(false);
    let (show_leave_modal, set_show_leave_modal) = signal(false);

    let force_leave = move || ctx.list();
    let request_leave = Callback::new(move |()| set_user_wants_to_leave.set(true));

    Effect::new(
        move |_prev| match (user_wants_to_leave.get(), input_changed.get()) {
            (true, true) => set_show_leave_modal.set(true),
            (true, false) => force_leave(),
            (false, _) => {}
        },
    );

    // TODO: Can we get rid of new_local?
    let save_action =
        Action::new_local(move |(create_model, and_then): &(DynCreateModel, Then)| {
            let create_model = create_model.clone();
            let and_then = and_then.clone();
            let data_provider = data_provider.get(); // TODO: This does not track!!
            async move {
                (
                    data_provider
                        .create_one(DynCreateOne {
                            entity: create_model,
                        })
                        .await
                        .and_then(|json| {
                            ctx.static_config
                                .read_value()
                                .model_handler
                                .deserialize_create_one_response
                                .run(json)
                                .map_err(|de_err| RequestError::Deserialize(de_err.to_string()))
                        }),
                    and_then,
                )
            }
        });

    let save_disabled = Signal::derive(move || {
        save_action.pending().get() // Note (lukas): We deliberately ignore the input_changed state here, as the default input should always be saveable!
    });

    let save_action_value = save_action.value();
    Effect::new(move |_prev| {
        if let Some((result, and_then)) = save_action_value.get() {
            match result {
                Ok(saved) => {
                    let id = saved.entity.id();
                    on_entity_created.run(saved);
                    match and_then {
                        Then::OpenEditView => on_edit_view.run(id),
                        Then::OpenListView => on_list_view.run(()),
                        Then::OpenCreateView => on_create_view.run(()),
                    }
                }
                Err(request_error) => {
                    tracing::warn!(
                        "Could not create entity due to error: {}",
                        request_error.to_string()
                    );
                    on_entity_creation_failed.run(CrudOperationError::from(request_error));
                }
            }
        }
    });

    let save = Callback::new(move |then| {
        save_action.dispatch((input.get(), then));
    });

    // TODO: Refactor this code. Much of it is shared with the edit_view!
    let value_changed =
        Callback::<(DynCreateField, Result<Value, String>)>::new(move |(field, result)| {
            tracing::info!(?field, ?result, "value changed");
            match result {
                Ok(value) => {
                    set_input.update(|input| field.set_value(input, value.clone())); // Clone avoidable? We have to set the signal as well...
                    set_input_errors.update(|errors| {
                        errors.remove(&field);
                    });
                    signals.update_value(|map| {
                        map.get(&field).expect("field must be present").set(value);
                    });
                }
                Err(err) => {
                    set_input_errors.update(|errors| {
                        errors.insert(field, err);
                    });
                }
            }
        });

    view! {
        <Actions save_disabled save request_leave />
        {move || match create_elements.get() {
            CreateElements::None => view! { "Keine Felder definiert." }.into_any(),
            CreateElements::Custom(create_elements) => {
                view! {
                    <CrudFields
                        field_renderer_registry=field_renderer_registry
                        elements=create_elements
                        signals=signals
                        mode=FieldMode::Editable
                        value_changed=value_changed
                        on_tab_selection=on_tab_selected
                    />
                }.into_any()
            }
        }}

        <CrudLeaveModal
            show_when=show_leave_modal
            on_cancel=move || {
                set_show_leave_modal.set(false);
                set_user_wants_to_leave.set(false);
            }
            on_accept=move || {
                set_show_leave_modal.set(false);
                force_leave();
            }
        />
    }
}

#[component]
fn Actions(
    save_disabled: Signal<bool>,
    save: Callback<Then>,
    request_leave: Callback<()>,
) -> impl IntoView {
    view! {
        <Grid gap=Size::Em(0.6) attr:class="crud-nav">
            <Row>
                <Col xs=6>
                    <ButtonWrapper>
                        <Button
                            color=ButtonColor::Primary
                            disabled=save_disabled
                            on_press=move |_| { save.run(Then::OpenEditView); }
                        >
                            "Speichern"
                        </Button>
                        <Button
                            color=ButtonColor::Primary
                            disabled=save_disabled
                            on_press=move |_| { save.run(Then::OpenListView); }
                        >
                            "Speichern und zurück"
                        </Button>
                        <Button
                            color=ButtonColor::Primary
                            disabled=save_disabled
                            on_press=move |_| { save.run(Then::OpenCreateView); }
                        >
                            "Speichern und neu"
                        </Button>
                    </ButtonWrapper>
                </Col>

                <Col xs=6 h_align=ColAlign::End>
                    <ButtonWrapper>
                        <Button color=ButtonColor::Secondary on_press=move |_| request_leave.run(())>
                            <span style="text-decoration: underline;">{"L"}</span>
                            {"istenansicht"}
                        </Button>
                    </ButtonWrapper>
                </Col>
            </Row>
        </Grid>
    }
}
