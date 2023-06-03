use std::{collections::HashMap, marker::PhantomData};

use crudkit_condition::IntoAllEqualCondition;
use crudkit_id::{Id, IdField};
use crudkit_shared::{SaveResult, Saved};
use crudkit_web::{
    prelude::{CrudRestDataProvider, CustomUpdateFields, ReadOne, UpdateOne},
    requests::RequestError,
    CrudDataTrait, CrudFieldValueTrait, CrudMainTrait, CrudSimpleView, DeletableModel, Elem,
    FieldMode, Value,
};
use leptonic::prelude::*;
use leptos::*;
use uuid::Uuid;

use crate::{
    crud_action::{CrudEntityAction, EntityModalGeneration, States},
    crud_action_context::CrudActionContext,
    crud_fields::CrudFields,
    crud_instance::CrudInstanceContext,
    crud_leave_modal::CrudLeaveModal,
    crud_table::NoDataAvailable,
};

#[derive(Debug, Clone, PartialEq)]
struct EntityReq<T: CrudMainTrait + 'static> {
    reload: Uuid,
    id: T::UpdateModelId,
    data_provider: CrudRestDataProvider<T>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Then {
    DoNothing,
    OpenListView,
    OpenCreateView,
}

// TODO: CrudEditView tracks changes, but CrudCreateView does not. Consolidate this logic into a shared component.

#[component]
pub fn CrudEditView<T>(
    cx: Scope,
    _phantom: PhantomData<T>,
    #[prop(into)] api_base_url: Signal<String>,
    /// The ID of the entity being edited.
    #[prop(into)]
    id: Signal<T::UpdateModelId>,
    #[prop(into)] data_provider: Signal<CrudRestDataProvider<T>>,
    #[prop(into)] actions: Signal<Vec<CrudEntityAction<T>>>,
    #[prop(into)] elements: Signal<Vec<Elem<T::UpdateModel>>>,
    #[prop(into)] custom_fields: Signal<CustomUpdateFields<T, leptos::View>>,

    on_entity_updated: Callback<Saved<T::UpdateModel>>,
    on_entity_update_aborted: Callback<String>,
    on_entity_not_updated_critical_errors: Callback<()>,
    on_entity_update_failed: Callback<RequestError>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let instance_ctx = expect_context::<CrudInstanceContext<T>>(cx);

    // The input is `None`, if the `entity` was not yet loaded. After the entity is loaded for the first time,
    // the this signal becomes a copy of the current (loaded) entity state.
    // We cannot use a `Default` value. The UpdateModel type may contain fields for which no default is available.
    // All modifications made through the UI are stored in this signal.
    let (input, set_input) = create_signal(cx, Option::<T::UpdateModel>::None);

    let entity_resource = create_local_resource(
        cx,
        move || {
            tracing::debug!("entity_req");
            EntityReq {
                reload: instance_ctx.reload.get(),
                id: id.get(),
                data_provider: data_provider.get(),
            }
        },
        move |req| async move {
            req.data_provider
            .read_one(ReadOne {
                skip: None,
                order_by: None,
                condition: Some(<T as CrudMainTrait>::UpdateModelId::fields_iter(&req.id) // TODO: This is complex and requires several use statements. Should be made easier.
                .map(|field| (field.name().to_owned(), field.to_value()))
                .into_all_equal_condition()),
            })
            .await
        },
    );

    // Stores the current state of the entity or an error, if no entity could be fetched.
    // Until the initial fetch request is completed, this is in the `Err(NoDataAvailable::NotYetLoaded` state!
    let (entity, set_entity) = create_signal(
        cx,
        Result::<StoredValue<T::UpdateModel>, NoDataAvailable>::Err(NoDataAvailable::NotYetLoaded),
    );

    // Update the `entity` signal whenever we fetched a new version of the edited entity.
    create_effect(cx, move |_prev| {
        set_entity.set(match entity_resource.read(cx) {
            Some(result) => {
                tracing::info!("loaded entity data");
                match result {
                    Ok(data) => match data {
                        Some(data) => {
                            let update_model = data.into();
                            // Copying the loaded entity data to be our current input, so that input fields can work on the data.
                            // TODO: Call something like into_update_model(), to make this into more readable.
                            set_input.set(Some(update_model.clone()));
                            Ok(store_value(cx, update_model))
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

    let input_changed = Signal::derive(cx, move || match (input.get(), entity.get()) {
        (Some(input), Ok(entity)) => entity.with_value(|entity| input != *entity),
        _ => false,
    });

    // The state of the `input` signal should be considered to be erroneous if at least one field is contained in this error list.
    let (input_errors, set_input_errors) = create_signal(
        cx,
        HashMap::<<T::UpdateModel as CrudDataTrait>::Field, String>::new(),
    );

    let (user_wants_to_leave, set_user_wants_to_leave) = create_signal(cx, false);
    let (show_leave_modal, set_show_leave_modal) = create_signal(cx, false);

    let force_leave = move || instance_ctx.list();
    let request_leave = move || set_user_wants_to_leave.set(true);

    create_effect(cx, move |_prev| {
        match (user_wants_to_leave.get(), input_changed.get()) {
            (true, true) => set_show_leave_modal.set(true),
            (true, false) => force_leave(),
            (false, _) => {}
        }
    });

    let save_action = create_action(cx, move |(entity, and_then): &(T::UpdateModel, Then)| {
        let entity: <T as CrudMainTrait>::UpdateModel = entity.clone();
        let and_then = and_then.clone();
        async move {
            (
                data_provider
                    .get() // TODO: This does not track!!
                    .update_one(UpdateOne {
                        entity: entity.clone(),
                        condition: Some(<T as CrudMainTrait>::UpdateModelId::fields_iter(&id.get()) // TODO: Simplify this!
                        .map(|field| (field.name().to_owned(), field.to_value()))
                        .into_all_equal_condition()),
                    })
                    .await,
                and_then
            )
        }
    });

    let save_disabled = Signal::derive(cx, move || {
        save_action.pending().get() || !input_changed.get()
    });

    let delete_disabled = Signal::derive(cx, move || {
        save_action.pending().get() || input.get().is_none()
    });

    let save_action_value = save_action.value();
    create_effect(cx, move |_prev| {
        if let Some((result, and_then)) = save_action_value.get() {
            match result {
                Ok(save_result) => match save_result {
                    SaveResult::Saved(saved) => {
                        set_entity.set(Ok(store_value(cx, saved.entity.clone())));
                        on_entity_updated.call(saved);
                        match and_then {
                            Then::DoNothing => {}
                            Then::OpenListView => instance_ctx.list(),
                            Then::OpenCreateView => instance_ctx.create(),
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

    let action_ctx = CrudActionContext::<T>::new(cx);

    let value_changed = Callback::<(
        <T::UpdateModel as CrudDataTrait>::Field,
        Result<Value, String>,
    )>::new(cx, move |(field, result)| {
        tracing::info!(?field, ?result, "value changed");
        match result {
            Ok(value) => {
                set_input.update(|input| match input {
                    Some(input) => field.set_value(input, value),
                    None => {}
                });
                set_input_errors.update(|errors| {
                    errors.remove(&field);
                });
            }
            Err(err) => {
                set_input_errors.update(|errors| {
                    errors.insert(field, err);
                });
            }
        }
    });

    view! {cx,
        { move || match entity.get() {
            Ok(entity) => view! {cx,
                { move || {
                    view! {cx,
                        <Grid spacing=6 class="crud-nav">
                            <Row>
                                <Col>
                                    <ButtonWrapper>
                                        <Button color=ButtonColor::Primary disabled=save_disabled on_click=move |_| trigger_save() variations=view!{cx,
                                            <Button color=ButtonColor::Primary disabled=save_disabled on_click=move |_| trigger_save_and_return()>
                                                "Speichern und zurück"
                                            </Button>
                                            <Button color=ButtonColor::Primary disabled=save_disabled on_click=move |_| trigger_save_and_new()>
                                                "Speichern und neu"
                                            </Button>
                                        }.into_view(cx)>
                                            "Speichern"
                                        </Button>
                                        <Button color=ButtonColor::Danger disabled=delete_disabled on_click=move |_| trigger_delete()>
                                            "Löschen"
                                        </Button>

                                        <For
                                            each=move || actions.get()
                                            key=|action| match action {
                                                CrudEntityAction::Custom {id, name: _, icon: _, button_color: _, valid_in: _, action: _, modal: _} => *id
                                            }
                                            view=move |cx, action| match action {
                                                CrudEntityAction::Custom {id, name, icon, button_color, valid_in, action, modal} => {
                                                    valid_in.contains(&States::Update).then(|| {
                                                        if let Some(modal_generator) = modal {
                                                            view! {cx,
                                                                <Button
                                                                    color=button_color
                                                                    disabled=Signal::derive(cx, move || action_ctx.is_action_executing(id))
                                                                    on_click=move |_| action_ctx.request_action(id)
                                                                >
                                                                    { icon.map(|icon| view! {cx, <Icon icon=icon/>}) }
                                                                    { name.clone() }
                                                                </Button>
                                                                {
                                                                    modal_generator.call((cx, EntityModalGeneration {
                                                                        show_when: Signal::derive(cx, move || action_ctx.is_action_requested(id)),
                                                                        state: input.into(),
                                                                        cancel: Callback::new(cx, move |_| action_ctx.cancel_action(id)),
                                                                        execute: Callback::new(cx, move |action_payload| action_ctx.trigger_entity_action(cx, id, input.get().unwrap(), action_payload, action)),
                                                                    }))
                                                                }
                                                            }.into_view(cx)
                                                        } else {
                                                            view! {cx,
                                                                <Button
                                                                    color=button_color
                                                                    disabled=Signal::derive(cx, move || action_ctx.is_action_executing(id))
                                                                    on_click=move |_| action_ctx.trigger_entity_action(cx, id, input.get().unwrap(), None, action)
                                                                >
                                                                    { icon.map(|icon| view! {cx, <Icon icon=icon/>}) }
                                                                    { name.clone() }
                                                                </Button>
                                                            }.into_view(cx)
                                                        }
                                                    })
                                                }
                                            }
                                        />
                                    </ButtonWrapper>
                                </Col>

                                <Col h_align=ColAlign::End>
                                    <ButtonWrapper>
                                        <Button color=ButtonColor::Secondary on_click=move |_| request_leave()>
                                            <span style="text-decoration: underline;">{"L"}</span>{"istenansicht"}
                                        </Button>
                                    </ButtonWrapper>
                                </Col>
                            </Row>
                        </Grid>
                    }
                } }

                <CrudFields
                    // children={ChildrenRenderer::new(ctx.props().children.iter().filter(|it| match it {
                    //     Item::NestedInstance(_) => true,
                    //     Item::Relation(_) => true,
                    //     Item::Select(select) => select.props.for_model == crate::crud_reset_field::Model::Update,
                    // }).collect::<Vec<Item>>())}
                    custom_fields=custom_fields
                    api_base_url=api_base_url
                    elements=elements
                    entity=entity
                    mode=FieldMode::Editable
                    current_view=CrudSimpleView::Edit
                    value_changed=value_changed
                    //     active_tab={ctx.props().config.active_tab.clone()}
                    // TODO: Propagate tab selections: ctx.props().on_tab_selected.emit(label);
                    //     on_tab_selection={ctx.link().callback(|label| Msg::TabSelected(label))}
                />
            }.into_view(cx),
            Err(no_data) => view! {cx,
                <Grid spacing=6 class="crud-nav">
                    <Row>
                        <Col h_align=ColAlign::End>
                            <ButtonWrapper>
                                <Button color=ButtonColor::Secondary on_click=move |_| force_leave()>
                                    <span style="text-decoration: underline;">{"L"}</span>{"istenansicht"}
                                </Button>
                            </ButtonWrapper>
                        </Col>
                    </Row>
                </Grid>
                <div>
                    {format!("Daten nicht verfügbar: {:?}", no_data)}
                </div>
            }.into_view(cx),
        } }

        <CrudLeaveModal
            show_when=show_leave_modal
            on_cancel=Callback::new(cx, move |()| {
                set_show_leave_modal.set(false);
                set_user_wants_to_leave.set(false);
            })
            on_accept=Callback::new(cx, move |()| {
                set_show_leave_modal.set(false);
                force_leave();
            })
        />
    }
}
