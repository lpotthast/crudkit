use crate::dynamic::crud_action::{CrudEntityAction, States};
use crate::dynamic::crud_action_buttons::CrudActionButtons;
use crate::dynamic::crud_action_context::CrudActionContext;
use crate::dynamic::crud_fields::CrudFields;
use crate::dynamic::crud_instance::CrudInstanceContext;
use crate::dynamic::crud_table::NoDataAvailable;
use crate::dynamic::custom_field::CustomUpdateFields;
use crate::shared::crud_instance_config::DynSelectConfig;
use crate::shared::crud_leave_modal::CrudLeaveModal;
use crate::ReactiveValue;
use crudkit_condition::{merge_conditions, IntoAllEqualCondition};
use crudkit_id::SerializableId;
use crudkit_shared::{SaveResult, Saved};
use crudkit_web::dynamic::prelude::*;
use crudkit_web::request_error::RequestError;
use leptonic::components::prelude::*;
use leptonic::prelude::*;
use leptos::prelude::*;
use std::collections::HashMap;

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
pub fn CrudEditView(
    /// The ID of the entity being edited.
    #[prop(into)]
    id: Signal<SerializableId>, // UpdateModel id
    #[prop(into)] data_provider: Signal<CrudRestDataProvider>,
    #[prop(into)] actions: Signal<Vec<CrudEntityAction>>,
    #[prop(into)] elements: Signal<Vec<AnyElem>>,
    #[prop(into)] custom_fields: Signal<CustomUpdateFields>,
    #[prop(into)] field_config: Signal<HashMap<AnyField, DynSelectConfig>>, // UpdateModel field
    #[prop(into)] on_list_view: Callback<()>,
    #[prop(into)] on_create_view: Callback<()>,
    #[prop(into)] on_entity_updated: Callback<(Saved<AnyModel>,)>, // UpdateModel
    #[prop(into)] on_entity_update_aborted: Callback<(String,)>,
    #[prop(into)] on_entity_not_updated_critical_errors: Callback<()>,
    #[prop(into)] on_entity_update_failed: Callback<(RequestError,)>,
    #[prop(into)] on_tab_selected: Callback<(TabId,)>,
) -> impl IntoView {
    let instance_ctx = expect_context::<CrudInstanceContext>();

    // The input is `None`, if the `entity` was not yet loaded. After the entity is loaded for the first time,
    // then this signal becomes a copy of the current (loaded) entity state.
    // We cannot use a `Default` value. The UpdateModel type may contain fields for which no default is available.
    // All modifications made through the UI are stored in this signal.
    let (input, set_input) = signal(Option::<AnyModel>::None);

    // TODO: Do not use LocalResouce, allow loading on server.
    let entity_resource = LocalResource::new(move || async move {
        let _ = instance_ctx.reload.get();
        let equals_id_condition = id.get().0.into_iter().into_all_equal_condition();
        data_provider
            .read()
            .read_one(ReadOne {
                skip: None,
                order_by: None,
                condition: merge_conditions(
                    instance_ctx.base_condition.get(),
                    Some(equals_id_condition),
                ),
            })
            .await
            .and_then(|json| {
                instance_ctx
                    .static_config
                    .read_value()
                    .model_handler
                    .deserialize_read_one_response
                    .run((json,))
                    .map_err(|de_err| RequestError::Deserialize(de_err.to_string()))
            })
    });

    // Stores the current state of the entity or an error, if no entity could be fetched.
    // Until the initial fetch request is completed, this is in the `Err(NoDataAvailable::NotYetLoaded` state!
    let (entity, set_entity) = signal(Result::<AnyModel, NoDataAvailable>::Err(
        NoDataAvailable::NotYetLoaded,
    ));

    let (signals, set_sig) = signal(StoredValue::<HashMap<AnyField, ReactiveValue>>::new(
        HashMap::new(),
    ));

    // Update the `entity` signal whenever we fetched a new version of the edited entity.
    Effect::new(move |_prev| {
        set_entity.set(match entity_resource.get() {
            Some(result) => {
                match result.take() {
                    // TODO: This code is shared with read_view
                    Ok(data) => match data {
                        Some(read_model) => {
                            let update_model = instance_ctx
                                .static_config
                                .read_value()
                                .model_handler
                                .read_model_to_update_model
                                .run((read_model,));

                            // Creating signals for all fields of the loaded entity, so that input fields can work on the data.
                            set_sig.set({
                                let signals = instance_ctx
                                    .static_config
                                    .read_value()
                                    .model_handler
                                    .update_model_to_signal_map
                                    .run((update_model.clone(),));
                                StoredValue::new(signals)
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
    let (_input_errors, set_input_errors) = signal(HashMap::<AnyField, String>::new());

    let (user_wants_to_leave, set_user_wants_to_leave) = signal(false);
    let (show_leave_modal, set_show_leave_modal) = signal(false);

    let force_leave = on_list_view;
    let request_leave = move || set_user_wants_to_leave.set(true);

    Effect::new(
        move |_prev| match (user_wants_to_leave.get(), input_changed.get()) {
            (true, true) => set_show_leave_modal.set(true),
            (true, false) => force_leave.run(()),
            (false, _) => {}
        },
    );

    let save_action = Action::new_local(move |(entity, and_then): &(AnyModel, Then)| {
        let entity: AnyModel = entity.clone();
        let and_then = and_then.clone();
        async move {
            (
                data_provider
                    .read()
                    .update_one(UpdateOne {
                        entity: entity.clone(),
                        condition: merge_conditions(
                            instance_ctx.base_condition.get(),
                            Some(id.get().0.into_iter().into_all_equal_condition()),
                        ),
                    })
                    .await
                    .and_then(|json| {
                        instance_ctx
                            .static_config
                            .read_value()
                            .model_handler
                            .deserialize_update_one_response
                            .run((json,))
                            .map_err(|de_err| RequestError::Deserialize(de_err.to_string()))
                    }),
                and_then,
            )
        }
    });

    let save_disabled = Signal::derive(move || save_action.pending().get() || !input_changed.get());

    let delete_disabled =
        Signal::derive(move || save_action.pending().get() || input.get().is_none());

    let save_action_value = save_action.value();
    Effect::new(move |_prev| {
        if let Some((result, and_then)) = save_action_value.get() {
            match result {
                Ok(save_result) => match save_result {
                    SaveResult::Saved(saved) => {
                        set_entity.set(Ok(saved.entity.clone()));
                        on_entity_updated.run((saved,));
                        match and_then {
                            Then::DoNothing => {}
                            Then::OpenListView => force_leave.run(()),
                            Then::OpenCreateView => on_create_view.run(()),
                        }
                    }
                    SaveResult::Aborted { reason } => {
                        on_entity_update_aborted.run((reason,));
                    }
                    SaveResult::CriticalValidationErrors => {
                        tracing::info!("Entity was not updated due to critical validation errors.");
                        on_entity_not_updated_critical_errors.run(());
                    }
                },
                Err(request_error) => {
                    set_entity.set(Err(NoDataAvailable::RequestFailed(request_error.clone())));
                    tracing::warn!(
                        "Could not update entity due to RequestError: {}",
                        request_error.to_string()
                    );
                    on_entity_update_failed.run((request_error,));
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
        instance_ctx.request_deletion_of(input.get().expect("Entity to be already loaded"));
    };

    let action_ctx = CrudActionContext::new();

    let value_changed =
        Callback::<(AnyField, Result<Value, String>)>::new(move |(field, result)| {
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
                    signals.with_untracked(|signals| {
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

    view! {
        {move || match (entity.get(), signals.get()) {
            (Ok(_entity), signals) => {
                view! {
                    <Grid gap=Size::Em(0.6) attr:class="crud-nav">
                        <Row>
                            <Col xs=6>
                                <ButtonWrapper>
                                    <Button
                                        color=ButtonColor::Primary
                                        disabled=save_disabled
                                        on_press=move |_| { trigger_save(); }
                                    >
                                        "Speichern"
                                    </Button>
                                    <Button
                                        color=ButtonColor::Primary
                                        disabled=save_disabled
                                        on_press=move |_| { trigger_save_and_return(); }
                                    >
                                        "Speichern und zurück"
                                    </Button>
                                    <Button
                                        color=ButtonColor::Primary
                                        disabled=save_disabled
                                        on_press=move |_| { trigger_save_and_new(); }
                                    >
                                        "Speichern und neu"
                                    </Button>
                                    <Button
                                        color=ButtonColor::Danger
                                        disabled=delete_disabled
                                        on_press=move |_| { trigger_delete(); }
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
                                    <Button color=ButtonColor::Secondary on_press=move |_| request_leave()>
                                        <span style="text-decoration: underline;">{"L"}</span>
                                        {"istenansicht"}
                                    </Button>
                                </ButtonWrapper>
                            </Col>
                        </Row>
                    </Grid>

                    <CrudFields
                        custom_fields=custom_fields
                        field_config=field_config
                        elements=elements
                        signals=signals
                        mode=FieldMode::Editable
                        current_view=CrudSimpleView::Edit
                        value_changed=value_changed.clone()
                        // active_tab={ctx.props().config.active_tab.clone()}
                        on_tab_selection=on_tab_selected.clone()
                    />
                }.into_any()
            }
            (Err(no_data), _) => {
                view! {
                    <Grid gap=Size::Em(0.6) attr:class="crud-nav">
                        <Row>
                            <Col h_align=ColAlign::End>
                                <ButtonWrapper>
                                    <Button color=ButtonColor::Secondary on_press=move |_| force_leave.run(())>
                                        <span style="text-decoration: underline;">{"L"}</span>
                                        {"istenansicht"}
                                    </Button>
                                </ButtonWrapper>
                            </Col>
                        </Row>
                    </Grid>
                    <NoDataAvailable no_data/>
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
                force_leave.run(());
            }
        />
    }
}

#[component]
fn NoDataAvailable(no_data: NoDataAvailable) -> impl IntoView {
    view! { <div>{format!("Daten nicht verfügbar: {:?}", no_data)}</div> }
}
