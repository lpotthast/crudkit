use std::marker::PhantomData;

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
    crud_action::{Callable, Callback, CrudEntityAction, EntityModalGeneration, States},
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
    on_list: Callback<()>,
    on_create: Callback<()>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let instance_ctx = expect_context::<CrudInstanceContext<T>>(cx);

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

    let entity: Memo<Result<StoredValue<T::UpdateModel>, NoDataAvailable>> =
        create_memo(cx, move |_prev| match entity_resource.read(cx) {
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
        });

    let (user_wants_to_leave, set_user_wants_to_leave) = create_signal(cx, false);

    // Only allow the user to save if the input diverges from the initial data of the entity.
    let save_disabled = Signal::derive(cx, move || match (input.get(), entity.get()) {
        (Some(input), Ok(entity)) => entity.with_value(|entity| input == *entity),
        _ => false,
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
    let save_action_value = save_action.value();
    create_effect(cx, move |_prev| {
        if let Some((result, and_then)) = save_action_value.get() {
            // TODO: Impl this
            // self.set_entity_from_save_result(result.clone(), SetFrom::Update, ctx);

            match result {
                Ok(save_result) => match save_result {
                    SaveResult::Saved(saved) => {
                        on_entity_updated.call_with(saved);
                        match and_then {
                            Then::DoNothing => {}
                            Then::OpenListView => on_list.call_with(()),
                            Then::OpenCreateView => on_create.call_with(()),
                        }
                    }
                    SaveResult::Aborted { reason } => {
                        on_entity_update_aborted.call_with(reason);
                    }
                    SaveResult::CriticalValidationErrors => {
                        tracing::info!("Entity was not updated due to critical validation errors.");
                        on_entity_not_updated_critical_errors.call_with(());
                    }
                },
                Err(err) => {
                    warn!(
                        "Could not update entity due to RequestError: {}",
                        err.to_string()
                    );
                    on_entity_update_failed.call_with(err);
                }
            }
        }
    });

    // TODO: Should probably be derived. Only allow saves when changes were made...
    let (delete_disabled, set_delete_disabled) = create_signal(cx, false);

    let force_leave = move || {
        instance_ctx.list();
    };
    let request_leave = move || {};
    let trigger_save = move || save_action.dispatch((input.get().unwrap(), Then::DoNothing));
    let trigger_save_and_return = move || save_action.dispatch((input.get().unwrap(), Then::OpenListView));
    let trigger_save_and_new = move || save_action.dispatch((input.get().unwrap(), Then::OpenCreateView));
    let trigger_delete = move || {
        instance_ctx.request_deletion_of(DeletableModel::Update(
            input.get().expect("Entity to be already loaded"),
        ));
    };

    let action_ctx = CrudActionContext::<T>::new(cx);

    let action_row = move || {
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
                                                        modal_generator.call_with((cx, EntityModalGeneration {
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
                            <Button color=ButtonColor::Secondary on_click=move |_| force_leave()>
                                <span style="text-decoration: underline;">{"L"}</span>{"istenansicht"}
                            </Button>
                        </ButtonWrapper>
                    </Col>
                </Row>
            </Grid>
        }
    };

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
            }
            Err(err) => {}
        }
    });

    view! {cx,
        { move || match entity.get() {
            Ok(entity) => view! {cx,
                { action_row }

                <CrudFields
                //     children={ChildrenRenderer::new(ctx.props().children.iter().filter(|it| match it {
                //         Item::NestedInstance(_) => true,
                //         Item::Relation(_) => true,
                //         Item::Select(select) => select.props.for_model == crate::crud_reset_field::Model::Update,
                //     }).collect::<Vec<Item>>())}

                    custom_fields=custom_fields
                    api_base_url=api_base_url
                    elements=elements
                    entity=entity
                    mode=FieldMode::Editable
                    current_view=CrudSimpleView::Edit
                    value_changed=value_changed
                //     active_tab={ctx.props().config.active_tab.clone()}
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
            show_when=user_wants_to_leave
            on_cancel=move || set_user_wants_to_leave.set(false)
            on_accept=move || set_user_wants_to_leave.set(false)
        />
    }
}
