use crate::generic::crud_fields::CrudFields;
use crate::generic::crud_instance::CrudInstanceContext;
use crate::generic::crud_instance_config::CreateElements;
use crate::generic::custom_field::CustomCreateFields;
use crate::shared::crud_leave_modal::CrudLeaveModal;
use crate::{IntoReactiveValue, ReactiveValue};
use crudkit_shared::{SaveResult, Saved};
use crudkit_web::generic::prelude::*;
use crudkit_web::TabId;
use leptonic::components::prelude::*;
use leptonic::prelude::*;
use leptos::prelude::*;
use std::{collections::HashMap, marker::PhantomData};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Then {
    OpenEditView,
    OpenListView,
    OpenCreateView,
}

// TODO: Make this a signal? How would we act upon changes?
fn default_create_model<T: CrudMainTrait + 'static>(
    ctx: &CrudInstanceContext<T>,
) -> T::CreateModel {
    let mut entity: T::CreateModel = Default::default();
    if let Some(parent) = ctx.parent.get_value() {
        if let Some(parent_id) = ctx.parent_id.get_untracked() {
            let (_field_name, value) = parent_id
                .0
                .iter()
                .find(|(field_name, _value)| field_name == parent.referenced_field.as_str())
                .expect("related parent field must be part of the parents id!");
            T::CreateModel::get_field(parent.referencing_field.as_str())
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
pub fn CrudCreateView<T>(
    _phantom: PhantomData<T>,
    #[prop(into)] data_provider: Signal<CrudRestDataProvider<T>>,
    #[prop(into)] create_elements: Signal<CreateElements<T>>,
    #[prop(into)] custom_fields: Signal<CustomCreateFields<T>>,
    #[prop(into)] on_edit_view: Callback<T::UpdateModelId>,
    #[prop(into)] on_list_view: Callback<()>,
    #[prop(into)] on_create_view: Callback<()>,
    // TODO: consolidate these into one "on_entity_creation_attempt" with type Result<CreateResult<T::UpdateModel>, SomeErrorType>?
    #[prop(into)] on_entity_created: Callback<Saved<T::UpdateModel>>,
    #[prop(into)] on_entity_creation_aborted: Callback<String>,
    #[prop(into)] on_entity_not_created_critical_errors: Callback<()>,
    #[prop(into)] on_entity_creation_failed: Callback<RequestError>,
    #[prop(into)] on_tab_selected: Callback<TabId>,
    // /// Required because when creating the initial CreateModel, we have to set the "parent id" field of that model to the given id.
    // /// TODO: Only a subset of the parent id might be required to for matching. Consider a CreateModel#initialize_with_parent_id(ParentId)...
    // pub parent_id: Option<SerializableId>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let instance_ctx = expect_context::<CrudInstanceContext<T>>();

    let default_create_model: T::CreateModel = default_create_model::<T>(&instance_ctx);

    let signals: StoredValue<HashMap<<T::CreateModel as CrudDataTrait>::Field, ReactiveValue>> =
        StoredValue::new({
            let mut map = HashMap::new();
            for field in T::CreateModel::get_all_fields() {
                let initial = field.get_value(&default_create_model);
                map.insert(field, initial.into_reactive_value());
            }
            map
        });

    // The CreateModel enforces a `Default` value! We cannot deserialize a loaded model, so we have to create one from scratch with which the UI can be initialized.
    // We therefore do not have to deal with None states in the create case, compared to the edit view.
    // All modifications made through the UI are stored in this signal.
    let (input, set_input) = signal::<T::CreateModel>(default_create_model.clone());

    let input_changed = Signal::derive(move || input.get() != default_create_model);

    // The state of the `input` signal should be considered to be erroneous if at least one field is contained in this error list.
    let (_input_errors, set_input_errors) =
        signal(HashMap::<<T::CreateModel as CrudDataTrait>::Field, String>::new());

    let (user_wants_to_leave, set_user_wants_to_leave) = signal(false);
    let (show_leave_modal, set_show_leave_modal) = signal(false);

    let force_leave = move || instance_ctx.list();
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
        Action::new_local(move |(create_model, and_then): &(T::CreateModel, Then)| {
            let create_model: <T as CrudMainTrait>::CreateModel = create_model.clone();
            let and_then = and_then.clone();
            let data_provider = data_provider.get(); // TODO: This does not track!!
            async move {
                (
                    data_provider
                        .create_one_from_create_model(CreateOne {
                            entity: create_model,
                        })
                        .await,
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
                Ok(save_result) => match save_result {
                    SaveResult::Saved(saved) => {
                        let id = saved.entity.get_id();
                        on_entity_created.run(saved);
                        match and_then {
                            Then::OpenEditView => on_edit_view.run(id),
                            Then::OpenListView => on_list_view.run(()),
                            Then::OpenCreateView => on_create_view.run(()),
                        }
                    }
                    SaveResult::Aborted { reason } => {
                        on_entity_creation_aborted.run(reason);
                    }
                    SaveResult::CriticalValidationErrors => {
                        tracing::info!("Entity was not created due to critical validation errors.");
                        on_entity_not_created_critical_errors.run(());
                    }
                },
                Err(request_error) => {
                    tracing::warn!(
                        "Could not create entity due to RequestError: {}",
                        request_error.to_string()
                    );
                    on_entity_creation_failed.run(request_error);
                }
            }
        }
    });

    let save = Callback::new(move |then| {
        save_action.dispatch((input.get(), then));
    });

    // TODO: Refactor this code. Much of it is shared with the edit_view!
    let value_changed = Callback::<(
        <T::CreateModel as CrudDataTrait>::Field,
        Result<Value, String>,
    )>::new(move |(field, result)| {
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
                        custom_fields=custom_fields
                        elements=create_elements
                        signals=signals
                        mode=FieldMode::Editable
                        current_view=CrudSimpleView::Create
                        value_changed=value_changed
                        // active_tab={ctx.props().config.active_tab.clone()}
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
