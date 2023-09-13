use std::{collections::HashMap, marker::PhantomData};

use crudkit_shared::{SaveResult, Saved};
use crudkit_web::{
    prelude::{CreateOne, CrudRestDataProvider, CustomCreateFields},
    requests::RequestError,
    CrudDataTrait, CrudFieldValueTrait, CrudIdTrait, CrudMainTrait, CrudSimpleView, FieldMode,
    TabId, Value,
};
use leptonic::prelude::*;
use leptos::*;
use uuid::Uuid;

use crate::{
    crud_fields::CrudFields,
    crud_instance::CrudInstanceContext,
    crud_instance_config::{CreateElements, DynSelectConfig},
    crud_leave_modal::CrudLeaveModal,
    IntoReactiveValue, ReactiveValue,
};

#[derive(Debug, Clone, PartialEq)]
struct EntityReq<T: CrudMainTrait + 'static> {
    reload: Uuid,
    id: T::UpdateModelId,
    data_provider: CrudRestDataProvider<T>,
}

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
            tracing::error!("CrudInstance is configured to be a nested instance but no parent id was passed down!");
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
    #[prop(into)] api_base_url: Signal<String>,
    #[prop(into)] data_provider: Signal<CrudRestDataProvider<T>>,
    #[prop(into)] create_elements: Signal<CreateElements<T>>,
    #[prop(into)] custom_fields: Signal<CustomCreateFields<T, leptos::View>>,
    #[prop(into)] field_config: Signal<
        HashMap<<T::CreateModel as CrudDataTrait>::Field, DynSelectConfig>,
    >,
    on_edit_view: Callback<T::UpdateModelId>,
    on_list_view: Callback<()>,
    on_create_view: Callback<()>,
    // TODO: consolidate these into one "on_entity_creation_attempt" with type Result<CreateResult<T::UpdateModel>, SomeErrorType>?
    on_entity_created: Callback<Saved<T::UpdateModel>>,
    on_entity_creation_aborted: Callback<String>,
    on_entity_not_created_critical_errors: Callback<()>,
    on_entity_creation_failed: Callback<RequestError>,
    on_tab_selected: Callback<TabId>,
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
        store_value({
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
    let (input, set_input) = create_signal::<T::CreateModel>(default_create_model.clone());

    let input_changed = Signal::derive(move || input.get() != default_create_model);

    // The state of the `input` signal should be considered to be erroneous if at least one field is contained in this error list.
    let (input_errors, set_input_errors) =
        create_signal(HashMap::<<T::CreateModel as CrudDataTrait>::Field, String>::new());

    let (user_wants_to_leave, set_user_wants_to_leave) = create_signal(false);
    let (show_leave_modal, set_show_leave_modal) = create_signal(false);

    let force_leave = move || instance_ctx.list();
    let request_leave = move || set_user_wants_to_leave.set(true);

    create_effect(
        move |_prev| match (user_wants_to_leave.get(), input_changed.get()) {
            (true, true) => set_show_leave_modal.set(true),
            (true, false) => force_leave(),
            (false, _) => {}
        },
    );

    let save_action = create_action(move |(create_model, and_then): &(T::CreateModel, Then)| {
        let create_model: <T as CrudMainTrait>::CreateModel = create_model.clone();
        let and_then = and_then.clone();
        async move {
            (
                data_provider
                    .get() // TODO: This does not track!!
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
    create_effect(move |_prev| {
        if let Some((result, and_then)) = save_action_value.get() {
            match result {
                Ok(save_result) => match save_result {
                    SaveResult::Saved(saved) => {
                        let id = saved.entity.get_id();
                        on_entity_created.call(saved);
                        match and_then {
                            Then::OpenEditView => on_edit_view.call(id),
                            Then::OpenListView => on_list_view.call(()),
                            Then::OpenCreateView => on_create_view.call(()),
                        }
                    }
                    SaveResult::Aborted { reason } => {
                        on_entity_creation_aborted.call(reason);
                    }
                    SaveResult::CriticalValidationErrors => {
                        tracing::info!("Entity was not created due to critical validation errors.");
                        on_entity_not_created_critical_errors.call(());
                    }
                },
                Err(request_error) => {
                    tracing::warn!(
                        "Could not create entity due to RequestError: {}",
                        request_error.to_string()
                    );
                    on_entity_creation_failed.call(request_error);
                }
            }
        }
    });

    let trigger_save = move || save_action.dispatch((input.get(), Then::OpenEditView));

    let trigger_save_and_return = move || save_action.dispatch((input.get(), Then::OpenListView));

    let trigger_save_and_new = move || save_action.dispatch((input.get(), Then::OpenCreateView));

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
        <Grid spacing=Size::Em(0.6) class="crud-nav">
            <Row>
                <Col xs=6>
                    <ButtonWrapper>
                        <Button
                            color=ButtonColor::Primary
                            disabled=save_disabled
                            on_click=move |_| trigger_save()
                            variations=view! {
                                <Button
                                    color=ButtonColor::Primary
                                    disabled=save_disabled
                                    on_click=move |_| trigger_save_and_return()
                                >
                                    "Speichern und zurück"
                                </Button>
                                <Button
                                    color=ButtonColor::Primary
                                    disabled=save_disabled
                                    on_click=move |_| trigger_save_and_new()
                                >
                                    "Speichern und neu"
                                </Button>
                            }
                                .into_view()
                        >
                            "Speichern"
                        </Button>
                    </ButtonWrapper>
                </Col>

                <Col xs=6 h_align=ColAlign::End>
                    <ButtonWrapper>
                        <Button color=ButtonColor::Secondary on_click=move |_| request_leave()>
                            <span style="text-decoration: underline;">{"L"}</span>
                            {"istenansicht"}
                        </Button>
                    </ButtonWrapper>
                </Col>
            </Row>
        </Grid>

        {move || match create_elements.get() {
            CreateElements::None => view! { "Keine Felder definiert." }.into_view(),
            CreateElements::Custom(create_elements) => {
                view! {
                    <CrudFields
                        custom_fields=custom_fields
                        field_config=field_config
                        api_base_url=api_base_url
                        elements=create_elements
                        signals=signals
                        mode=FieldMode::Editable
                        current_view=CrudSimpleView::Create
                        value_changed=value_changed
                        // active_tab={ctx.props().config.active_tab.clone()}
                        on_tab_selection=on_tab_selected
                        entity=input.into()
                    />
                }
                    .into_view()
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
