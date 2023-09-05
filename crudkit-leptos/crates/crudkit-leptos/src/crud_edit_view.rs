use std::{collections::HashMap, marker::PhantomData};

use crudkit_condition::{merge_conditions, Condition, IntoAllEqualCondition};
use crudkit_id::{Id, IdField};
use crudkit_shared::{SaveResult, Saved};
use crudkit_web::{
    prelude::{CrudRestDataProvider, CustomUpdateFields, ReadOne, UpdateOne},
    requests::RequestError,
    CrudDataTrait, CrudFieldValueTrait, CrudMainTrait, CrudSimpleView, DeletableModel, Elem,
    FieldMode, TabId, Value,
};
use leptonic::prelude::*;
use leptos::*;
use uuid::Uuid;

use crate::{
    crud_action::{CrudEntityAction, States},
    crud_action_buttons::CrudActionButtons,
    crud_action_context::CrudActionContext,
    crud_fields::CrudFields,
    crud_instance::CrudInstanceContext,
    crud_instance_config::DynSelectConfig,
    crud_leave_modal::CrudLeaveModal,
    crud_table::NoDataAvailable,
    IntoReactiveValue, ReactiveValue,
};

#[derive(Debug, Clone, PartialEq)]
struct EntityReq<T: CrudMainTrait + 'static> {
    condition: Option<Condition>,
    reload: Uuid,
    data_provider: CrudRestDataProvider<T>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Then {
    DoNothing,
    OpenListView,
    OpenCreateView,
}

// TODO: CrudEditView tracks changes, but CrudCreateView does not. Consolidate this logic into a shared component.

// When the entity state is loaded, its data is put into a map of signals.
// One key-value pair for each field and its corresponding signal.
// We need to know if some inputs were changed.
// We do this by comparing every fields signal value against the value of the entity loaded.
// If any field value differs, the input was changed. This can be memoized.

#[component]
pub fn CrudEditView<T>(
    _phantom: PhantomData<T>,
    #[prop(into)] api_base_url: Signal<String>,
    /// The ID of the entity being edited.
    #[prop(into)]
    id: Signal<T::UpdateModelId>,
    #[prop(into)] data_provider: Signal<CrudRestDataProvider<T>>,
    #[prop(into)] actions: Signal<Vec<CrudEntityAction<T>>>,
    #[prop(into)] elements: Signal<Vec<Elem<T::UpdateModel>>>,
    #[prop(into)] custom_fields: Signal<CustomUpdateFields<T, leptos::View>>,
    #[prop(into)] field_config: Signal<
        HashMap<<T::UpdateModel as CrudDataTrait>::Field, DynSelectConfig>,
    >,
    on_list_view: Callback<()>,
    on_create_view: Callback<()>,
    on_entity_updated: Callback<Saved<T::UpdateModel>>,
    on_entity_update_aborted: Callback<String>,
    on_entity_not_updated_critical_errors: Callback<()>,
    on_entity_update_failed: Callback<RequestError>,
    on_tab_selected: Callback<TabId>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let instance_ctx = expect_context::<CrudInstanceContext<T>>();

    // The input is `None`, if the `entity` was not yet loaded. After the entity is loaded for the first time,
    // the this signal becomes a copy of the current (loaded) entity state.
    // We cannot use a `Default` value. The UpdateModel type may contain fields for which no default is available.
    // All modifications made through the UI are stored in this signal.
    let (input, set_input) = create_signal(Option::<T::UpdateModel>::None);

    let entity_resource = create_local_resource(
        move || {
            tracing::debug!("entity_req");
            let id = id.get();
            let equals_id_condition =
                <T as CrudMainTrait>::UpdateModelId::fields_iter(&id) // TODO: This is complex and requires several use statements. Should be made easier.
                    .map(|field| (field.name().to_owned(), field.to_value()))
                    .into_all_equal_condition();
            EntityReq {
                condition: merge_conditions(
                    instance_ctx.base_condition.get(),
                    Some(equals_id_condition),
                ),
                reload: instance_ctx.reload.get(),
                data_provider: data_provider.get(),
            }
        },
        move |req| async move {
            req.data_provider
                .read_one(ReadOne {
                    skip: None,
                    order_by: None,
                    condition: req.condition,
                })
                .await
        },
    );

    // Stores the current state of the entity or an error, if no entity could be fetched.
    // Until the initial fetch request is completed, this is in the `Err(NoDataAvailable::NotYetLoaded` state!
    let (entity, set_entity) = create_signal(Result::<T::UpdateModel, NoDataAvailable>::Err(
        NoDataAvailable::NotYetLoaded,
    ));

    let (signals, set_sig) = create_signal::<
        StoredValue<HashMap<<T::UpdateModel as CrudDataTrait>::Field, ReactiveValue>>,
    >(store_value(HashMap::new()));

    // Update the `entity` signal whenever we fetched a new version of the edited entity.
    create_effect(move |_prev| {
        set_entity.set(match entity_resource.get() {
            Some(result) => {
                tracing::info!("loaded entity data");
                match result {
                    Ok(data) => match data {
                        Some(data) => {
                            let update_model = data.into();
                            // Creating signals for all fields of the loaded entity, so that input fields can work on the data.
                            set_sig.set({
                                let mut map = HashMap::new();
                                for field in T::UpdateModel::get_all_fields() {
                                    let initial = field.get_value(&update_model);
                                    map.insert(field, initial.into_reactive_value());
                                }
                                store_value(map)
                            });

                            // Copying the loaded entity data to be our current final input.
                            set_input.set(Some(update_model.clone()));

                            Ok(update_model)
                        }
                        None => Err(NoDataAvailable::RequestReturnedNoData(format!(
                            "Eintrag existiert nicht."
                        ))),
                    },
                    Err(reason) => Err(NoDataAvailable::RequestFailed(reason)),
                }
            }
            None => Err(NoDataAvailable::NotYetLoaded),
        })
    });

    let input_changed = Signal::derive(move || match (input.get(), entity.get()) {
        (Some(input), Ok(entity)) => input != entity,
        _ => false,
    });

    // The state of the `input` signal should be considered to be erroneous if at least one field is contained in this error list.
    let (input_errors, set_input_errors) =
        create_signal(HashMap::<<T::UpdateModel as CrudDataTrait>::Field, String>::new());

    let (user_wants_to_leave, set_user_wants_to_leave) = create_signal(false);
    let (show_leave_modal, set_show_leave_modal) = create_signal(false);

    let force_leave = move || on_list_view.call(());
    let request_leave = move || set_user_wants_to_leave.set(true);

    create_effect(
        move |_prev| match (user_wants_to_leave.get(), input_changed.get()) {
            (true, true) => set_show_leave_modal.set(true),
            (true, false) => force_leave(),
            (false, _) => {}
        },
    );

    let save_action =
        create_action(move |(entity, and_then): &(T::UpdateModel, Then)| {
            let entity: <T as CrudMainTrait>::UpdateModel = entity.clone();
            let and_then = and_then.clone();
            async move {
                (
                    data_provider
                        .get() // TODO: This does not track!!
                        .update_one(UpdateOne {
                            entity: entity.clone(),
                            condition:
                                merge_conditions(
                                    instance_ctx.base_condition.get(),
                                    Some(
                                        <T as CrudMainTrait>::UpdateModelId::fields_iter(&id.get()) // TODO: Simplify this!
                                            .map(|field| {
                                                (field.name().to_owned(), field.to_value())
                                            })
                                            .into_all_equal_condition(),
                                    ),
                                ),
                        })
                        .await,
                    and_then,
                )
            }
        });

    let save_disabled = Signal::derive(move || save_action.pending().get() || !input_changed.get());

    let delete_disabled =
        Signal::derive(move || save_action.pending().get() || input.get().is_none());

    let save_action_value = save_action.value();
    create_effect(move |_prev| {
        if let Some((result, and_then)) = save_action_value.get() {
            match result {
                Ok(save_result) => match save_result {
                    SaveResult::Saved(saved) => {
                        set_entity.set(Ok(saved.entity.clone()));
                        on_entity_updated.call(saved);
                        match and_then {
                            Then::DoNothing => {}
                            Then::OpenListView => on_list_view.call(()),
                            Then::OpenCreateView => on_create_view.call(()),
                        }
                    }
                    SaveResult::Aborted { reason } => {
                        on_entity_update_aborted.call(reason);
                    }
                    SaveResult::CriticalValidationErrors => {
                        tracing::info!("Entity was not updated due to critical validation errors.");
                        on_entity_not_updated_critical_errors.call(());
                    }
                },
                Err(request_error) => {
                    set_entity.set(Err(NoDataAvailable::RequestFailed(request_error.clone())));
                    warn!(
                        "Could not update entity due to RequestError: {}",
                        request_error.to_string()
                    );
                    on_entity_update_failed.call(request_error);
                }
            }
        }
    });

    let trigger_save = move || save_action.dispatch((input.get().unwrap(), Then::DoNothing));

    let trigger_save_and_return =
        move || save_action.dispatch((input.get().unwrap(), Then::OpenListView));

    let trigger_save_and_new =
        move || save_action.dispatch((input.get().unwrap(), Then::OpenCreateView));

    let trigger_delete = move || {
        instance_ctx.request_deletion_of(DeletableModel::Update(
            input.get().expect("Entity to be already loaded"),
        ));
    };

    let action_ctx = CrudActionContext::<T>::new();

    let value_changed = Callback::<(
        <T::UpdateModel as CrudDataTrait>::Field,
        Result<Value, String>,
    )>::new(move |(field, result)| {
        tracing::info!(?field, ?result, "value changed");
        match result {
            Ok(value) => {
                set_input.update(|input| match input {
                    Some(input) => field.set_value(input, value.clone()),
                    None => {}
                });
                set_input_errors.update(|errors| {
                    errors.remove(&field);
                });
                signals.with(|signals| {
                    signals.update_value(|map| {
                        map.get(&field).expect("field must be present").set(value);
                    })
                });
            }
            Err(err) => {
                set_input_errors.update(|errors| {
                    errors.insert(field, err);
                });
            }
        }
    });

    let expect_input = Signal::derive(move || input.get().expect("input"));

    view! {
        {move || match (entity.get(), signals.get()) {
            (Ok(_entity), signals) => {
                view! {
                    {move || {
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
                                            <Button
                                                color=ButtonColor::Danger
                                                disabled=delete_disabled
                                                on_click=move |_| trigger_delete()
                                            >
                                                "Löschen"
                                            </Button>

                                            <CrudActionButtons
                                                action_ctx=action_ctx
                                                actions=actions
                                                input=input
                                                required_state=States::Update
                                            />
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
                        }
                    }}

                    <CrudFields
                        custom_fields=custom_fields
                        field_config=field_config
                        api_base_url=api_base_url
                        elements=elements
                        signals=signals
                        mode=FieldMode::Editable
                        current_view=CrudSimpleView::Edit
                        value_changed=value_changed
                        // active_tab={ctx.props().config.active_tab.clone()}
                        on_tab_selection=on_tab_selected
                        entity=expect_input
                    />
                }
                    .into_view()
            }
            (Err(no_data), _) => {
                view! {
                    // active_tab={ctx.props().config.active_tab.clone()}

                    // active_tab={ctx.props().config.active_tab.clone()}

                    // active_tab={ctx.props().config.active_tab.clone()}

                    // active_tab={ctx.props().config.active_tab.clone()}
                    <Grid spacing=Size::Em(0.6) class="crud-nav">
                        <Row>
                            <Col h_align=ColAlign::End>
                                <ButtonWrapper>
                                    <Button color=ButtonColor::Secondary on_click=move |_| force_leave()>
                                        <span style="text-decoration: underline;">{"L"}</span>
                                        {"istenansicht"}
                                    </Button>
                                </ButtonWrapper>
                            </Col>
                        </Row>
                    </Grid>
                    <NoDataAvailable no_data/>
                }
                    .into_view()
            }
        }}

        <CrudLeaveModal
            show_when=show_leave_modal
            on_cancel=create_callback(move |()| {
                set_show_leave_modal.set(false);
                set_user_wants_to_leave.set(false);
            })

            on_accept=create_callback(move |()| {
                set_show_leave_modal.set(false);
                force_leave();
            })
        />
    }
}

#[component]
fn NoDataAvailable(no_data: NoDataAvailable) -> impl IntoView {
    view! { <div>{format!("Daten nicht verfügbar: {:?}", no_data)}</div> }
}
