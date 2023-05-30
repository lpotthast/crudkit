use std::marker::PhantomData;

use crudkit_condition::IntoAllEqualCondition;
use crudkit_id::{Id, IdField};
use crudkit_web::{
    prelude::{CrudRestDataProvider, ReadOne},
    requests::RequestError,
    CrudMainTrait, DeletableModel,
};
use leptonic::prelude::*;
use leptos::*;
use uuid::Uuid;

use crate::{
    crud_action::{Callback, CrudEntityAction, EntityModalGeneration, States},
    crud_action_context::CrudActionContext,
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

async fn load_entity<T: CrudMainTrait + 'static>(
    req: EntityReq<T>,
) -> Result<Option<T::ReadModel>, RequestError> {
    // TODO: This is complex and requires several use statements. Should be made easier.
    let condition = <T as CrudMainTrait>::UpdateModelId::fields_iter(&req.id)
        .map(|field| (field.name().to_owned(), field.to_value()))
        .into_all_equal_condition();

    req.data_provider
        .read_one(ReadOne {
            skip: None,
            order_by: None,
            condition: Some(condition),
        })
        .await
}

#[component]
pub fn CrudEditView<T>(
    cx: Scope,
    _phantom: PhantomData<T>,
    /// The ID of the entity being edited.
    #[prop(into)]
    id: MaybeSignal<T::UpdateModelId>,
    #[prop(into)] data_provider: Signal<CrudRestDataProvider<T>>,
    #[prop(into)] actions: Signal<Vec<CrudEntityAction<T>>>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let instance_ctx = expect_context::<CrudInstanceContext<T>>(cx);

    let (input, set_input) = create_signal(cx, Option::<T::UpdateModel>::None);

    // Whenever this signal returns a new/different value, the data of the currently viewed entity is re-fetched.
    let entity_req = Signal::derive(cx, move || {
        tracing::debug!("entity_req");
        // Every server-provided resource must be reloaded when a general reload is requested!
        let reload = instance_ctx.reload.get();
        let id = id.get();
        let data_provider = data_provider.get();
        EntityReq {
            reload,
            id,
            data_provider,
        }
    });

    let entity_res = create_local_resource(cx, move || entity_req.get(), load_entity);

    // TODO: create_memo or Signal::derive??? We only want this once..
    let data = create_memo(cx, move |_prev| match entity_res.read(cx) {
        Some(result) => {
            tracing::info!("loaded entity data");
            match result {
                Ok(data) => {
                    // Copying the loaded entity data to be our current input, so that input fields can work on the data.
                    // TODO: Call something like into_update_model(), to make this into more readable.
                    set_input.set(data.clone().map(|it| it.into()));
                    Ok(data)
                }
                Err(reason) => Err(NoDataAvailable::FetchFailed(reason)),
            }
        }
        None => Err(NoDataAvailable::NotYetLoaded),
    });

    let (user_wants_to_leave, set_user_wants_to_leave) = create_signal(cx, false);

    // TODO: Should probably be derived.
    let (save_disabled, set_save_disabled) = create_signal(cx, true);
    let (delete_disabled, set_delete_disabled) = create_signal(cx, false);

    let force_leave = move || {
        instance_ctx.list();
    };
    let request_leave = move || {};
    let trigger_save = move || {};
    let trigger_save_and_return = move || {};
    let trigger_save_and_new = move || {};
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

                            {
                                move || actions.get().into_iter().map(|action| match action {
                                    CrudEntityAction::Custom {id, name, icon, button_color, valid_in, action, modal} => {
                                        valid_in.contains(&States::Update).then(|| {
                                            let action_id: &'static str = id; // TODO: remove
                                            let action = action.clone();

                                            if let Some(modal) = modal {
                                                view! {cx,
                                                    <Button
                                                        color=button_color
                                                        disabled=Signal::derive(cx, move || action_ctx.is_action_executing(action_id))
                                                        on_click=move |_| action_ctx.request_action(action_id)
                                                    >
                                                        { icon.map(|icon| view! {cx, <Icon icon=icon/>}) }
                                                        { name.clone() }
                                                    </Button>
                                                    {
                                                        modal.0((cx, EntityModalGeneration {
                                                            show_when: Signal::derive(cx, move || action_ctx.is_action_requested(action_id)),
                                                            state: input.into(),
                                                            cancel: Callback::of(move |_| action_ctx.cancel_action(action_id.clone())),
                                                            execute: Callback::of(move |action_payload| action_ctx.trigger_entity_action(cx, action_id, input.get().unwrap(), action_payload, action.clone())),
                                                        }))
                                                    }
                                                }.into_view(cx)
                                            } else {
                                                view! {cx,
                                                    <Button
                                                        color=button_color
                                                        disabled=Signal::derive(cx, move || action_ctx.is_action_executing(action_id))
                                                        on_click=move |_| action_ctx.trigger_entity_action(cx ,action_id, input.get().unwrap(), None, action.clone())
                                                    >
                                                        { icon.map(|icon| view! {cx, <Icon icon=icon/>}) }
                                                        { name.clone() }
                                                    </Button>
                                                }.into_view(cx)
                                            }
                                        })
                                    }
                                }).collect_view(cx)
                            }
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

    view! {cx,
        { move || match data.get() {
            Ok(data) => view! {cx,
                { action_row }

                // <CrudFields<T::UpdateModel>
                //     api_base_url={ctx.props().config.api_base_url.clone()}
                //     children={ChildrenRenderer::new(ctx.props().children.iter().filter(|it| match it {
                //         Item::NestedInstance(_) => true,
                //         Item::Relation(_) => true,
                //         Item::Select(select) => select.props.for_model == crate::crud_reset_field::Model::Update,
                //     }).collect::<Vec<Item>>())}
                //     custom_fields={ctx.props().custom_fields.clone()}
                //     elements={ctx.props().config.elements.clone()}
                //     entity={self.input.clone()}
                //     mode={FieldMode::Editable}
                //     current_view={CrudSimpleView::Edit}
                //     value_changed={ctx.link().callback(Msg::ValueChanged)}
                //     active_tab={ctx.props().config.active_tab.clone()}
                //     on_tab_selection={ctx.link().callback(|label| Msg::TabSelected(label))}
                // />
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
