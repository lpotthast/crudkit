use std::rc::Rc;

use crud_shared::Order;
use yew::{
    html::{ChildrenRenderer, Scope},
    prelude::*,
};
use yew_bootstrap_icons::Bi;

use crate::{
    crud_action::ModalGeneration,
    crud_instance::Item,
    services::crud_rest_data_provider::{CrudRestDataProvider, ReadCount, ReadMany},
    types::custom_field::CustomReadFields,
};

use super::{prelude::*, types::RequestError};

// TODO: Disable the reset button as long as there is an ongoing request!
// TODO: Disable the reset button when a reset is going on...

pub enum Msg<T: CrudMainTrait> {
    ComponentCreated,
    PageSelected(u64),
    ItemCountSelected(u64),
    PageLoaded(Result<Vec<T::ReadModel>, RequestError>),
    CountRead(Result<usize, RequestError>),
    ToggleFilter,
    OrderBy((<T::ReadModel as CrudDataTrait>::Field, OrderByUpdateOptions)),
    Create,
    EntrySelectionChanged(Vec<T::ReadModel>),
    Read(T::ReadModel),
    Edit(T::ReadModel),
    Delete(T::ReadModel),
    ActionInitialized {
        action_id: &'static str,
    },
    ActionCancelled {
        action_id: &'static str,
    },
    ActionTriggered {
        action_id: &'static str,
        action_payload: Option<T::ActionPayload>,
        action: Callback<(
            Option<T::ActionPayload>,
            Callback<Result<CrudActionAftermath, CrudActionAftermath>>,
        )>,
    },
    ActionExecuted {
        action_id: &'static str,
        result: Result<CrudActionAftermath, CrudActionAftermath>,
    },
    EntityActionTriggered((Rc<Box<dyn CrudActionTrait>>, T::ReadModel)),
    Reset,
    Reload,
}

#[derive(Properties, PartialEq)]
pub struct Props<T: CrudMainTrait + 'static> {
    pub children: ChildrenRenderer<Item>,
    pub custom_fields: CustomReadFields<T>,
    pub data_provider: CrudRestDataProvider<T>,
    pub config: CrudInstanceConfig<T>,
    pub static_config: CrudStaticInstanceConfig<T>,
    pub on_reset: Callback<()>,
    pub on_create: Callback<()>,
    pub on_read: Callback<T::ReadModel>,
    pub on_edit: Callback<T::ReadModel>,
    pub on_delete: Callback<T::ReadModel>,
    pub on_order_by: Callback<(<T::ReadModel as CrudDataTrait>::Field, OrderByUpdateOptions)>,
    pub on_page_selected: Callback<u64>,
    pub on_item_count_selected: Callback<u64>,
    pub on_entity_action: Callback<(Rc<Box<dyn CrudActionTrait>>, T::ReadModel)>,
    pub on_global_action: Callback<CrudActionAftermath>,
    pub on_link: Callback<Option<Scope<CrudListView<T>>>>,
}

pub struct CrudListView<T: 'static + CrudMainTrait> {
    data: Result<Rc<Vec<T::ReadModel>>, (NoData, time::OffsetDateTime)>,
    selected: Vec<T::ReadModel>,
    filter: Option<()>,
    item_count: Result<u64, (NoData, time::OffsetDateTime)>,
    user_wants_to_activate: Vec<String>,
    actions_executing: Vec<&'static str>,
}

impl<T: CrudMainTrait> CrudListView<T> {
    fn load_page(&self, ctx: &Context<CrudListView<T>>) {
        let order_by = ctx.props().config.order_by.clone();
        let page = ctx.props().config.page as u64;
        let items_per_page = ctx.props().config.items_per_page as u64;
        let data_provider = ctx.props().data_provider.clone();
        ctx.link().send_future(async move {
            Msg::PageLoaded(
                data_provider
                    .read_many(ReadMany {
                        limit: Some(items_per_page),
                        skip: Some(items_per_page * (page - 1)),
                        order_by: Some(order_by),
                        condition: None,
                    })
                    .await,
            )
        });
    }

    fn load_count(&self, ctx: &Context<CrudListView<T>>) {
        let data_provider = ctx.props().data_provider.clone();
        ctx.link().send_future(async move {
            Msg::CountRead(
                data_provider
                    .read_count(ReadCount { condition: None })
                    .await,
            )
        });
    }

    fn get_data(&self) -> Option<Rc<Vec<T::ReadModel>>> {
        match &self.data {
            Ok(data) => Some(data.clone()),
            Err(_) => None,
        }
    }

    fn get_data_error(&self) -> Option<(NoData, time::OffsetDateTime)> {
        match &self.data {
            Ok(_) => None,
            Err(err) => Some(err.clone()),
        }
    }
}

impl<T: 'static + CrudMainTrait> Component for CrudListView<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.props().on_link.emit(Some(ctx.link().clone()));
        ctx.link().send_future(async move { Msg::ComponentCreated });
        Self {
            data: Err((NoData::NotYetLoaded, time::OffsetDateTime::now_utc())),
            selected: vec![],
            filter: None,
            item_count: Err((NoData::NotYetLoaded, time::OffsetDateTime::now_utc())),
            user_wants_to_activate: vec![],
            actions_executing: vec![],
        }
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        ctx.props().on_link.emit(None);
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ComponentCreated => {
                self.load_page(ctx);
                self.load_count(ctx);
                false
            }
            Msg::PageSelected(page) => {
                ctx.props().on_page_selected.emit(page);
                //self.data = Err(NoData::NotYetLoaded);
                false
            }
            Msg::ItemCountSelected(page) => {
                ctx.props().on_item_count_selected.emit(page);
                //self.data = Err(NoData::NotYetLoaded);
                false
            }
            Msg::PageLoaded(data) => {
                self.data = data
                    .map(Rc::new)
                    .map_err(|err| (NoData::FetchFailed(err), time::OffsetDateTime::now_utc()));
                true
            }
            Msg::CountRead(data) => {
                self.item_count = data
                    .map_err(|err| (NoData::FetchFailed(err), time::OffsetDateTime::now_utc()))
                    .map(|val| val as u64);
                true
            }
            Msg::Reset => {
                ctx.props().on_reset.emit(());
                true
            }
            Msg::ToggleFilter => todo!(),
            Msg::OrderBy((field, options)) => {
                ctx.props().on_order_by.emit((field, options));
                false
            }
            Msg::Create => {
                ctx.props().on_create.emit(());
                false
            }
            Msg::EntrySelectionChanged(selected) => {
                self.selected = selected;
                // TODO: Show special ui for multi-selection.
                true
            }
            Msg::Read(entity) => {
                ctx.props().on_read.emit(entity);
                false
            }
            Msg::Edit(entity) => {
                ctx.props().on_edit.emit(entity);
                false
            }
            Msg::Delete(entity) => {
                ctx.props().on_delete.emit(entity);
                false
            }
            Msg::ActionInitialized { action_id } => {
                self.user_wants_to_activate.push(action_id.to_string());
                true
            }
            Msg::ActionCancelled { action_id } => {
                if let Some(index) = self
                    .user_wants_to_activate
                    .iter()
                    .position(|it| it.as_str() == action_id)
                {
                    self.user_wants_to_activate.remove(index);
                    true
                } else {
                    false
                }
            }
            Msg::ActionTriggered {
                action_id,
                action_payload,
                action,
            } => {
                if let Some(index) = self
                    .user_wants_to_activate
                    .iter()
                    .position(|it| it.as_str() == action_id)
                {
                    self.user_wants_to_activate.remove(index);
                }
                action.emit((
                    action_payload,
                    ctx.link()
                        .callback(move |result| Msg::ActionExecuted { action_id, result }),
                ));
                if !self.actions_executing.contains(&action_id) {
                    self.actions_executing.push(action_id);
                    true
                } else {
                    false
                }
            }
            Msg::ActionExecuted { action_id, result } => {
                // We currently handle both the success and the error path in the same way. This might need to be changes in the future.
                // But the user should always state in which path we are!
                match result {
                    Ok(aftermath) => ctx.props().on_global_action.emit(aftermath),
                    Err(aftermath) => ctx.props().on_global_action.emit(aftermath),
                }
                if let Some(index) = self
                    .actions_executing
                    .iter()
                    .position(|it| *it == action_id)
                {
                    self.actions_executing.remove(index);
                    true
                } else {
                    false
                }
            }
            Msg::EntityActionTriggered((action, entity)) => {
                ctx.props().on_entity_action.emit((action, entity));
                false
            }
            Msg::Reload => {
                self.load_page(ctx);
                self.load_count(ctx);
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.load_page(ctx);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <div class={"crud-row crud-nav"}>
                    <div class={"crud-col"}>
                        <CrudBtnWrapper>
                            <CrudBtn name={""} variant={Variant::Success} icon={Bi::PlusCircle} onclick={ctx.link().callback(|_| Msg::Create)}>
                                <CrudBtnName>
                                    <span style="text-decoration: underline">{"N"}</span>{"eu"}
                                </CrudBtnName>
                            </CrudBtn>

                            {
                                ctx.props().static_config.actions.iter()
                                    .map(|action| match action {
                                        CrudAction::Custom {id, name, icon, variant, action, modal} => {
                                            let action_id: &str = (&id).clone();
                                            let action = action.clone();

                                            if let Some(modal) = modal {
                                                html! {
                                                    <>
                                                    <CrudBtn
                                                        name={name.clone()}
                                                        variant={variant.clone()}
                                                        icon={icon.clone()}
                                                        disabled={self.actions_executing.contains(&id)}
                                                        onclick={ctx.link().callback(move |_| Msg::ActionInitialized { action_id }) }
                                                    />
                                                    if self.user_wants_to_activate.iter().any(|it| it.as_str() == action_id) {
                                                        <CrudModal>
                                                            {{ modal(ModalGeneration {
                                                                cancel: ctx.link().callback(move |_| Msg::ActionCancelled { action_id }),
                                                                execute: ctx.link().callback(move |action_payload| Msg::ActionTriggered { action_id, action_payload, action: action.clone() }),
                                                            }) }}
                                                        </CrudModal>
                                                    }
                                                    </>
                                                }
                                            } else {
                                                html! {
                                                    <CrudBtn
                                                        name={name.clone()}
                                                        variant={variant.clone()}
                                                        icon={icon.clone()}
                                                        disabled={self.actions_executing.contains(&id)}
                                                        onclick={ctx.link().callback(move |_| Msg::ActionTriggered { action_id, action_payload: None, action: action.clone() }) }
                                                    />
                                                }
                                            }
                                        }
                                    })
                                    .collect::<Html>()
                            }
                        </CrudBtnWrapper>
                    </div>

                    <div class={"crud-col crud-col-flex-end"}>
                        <CrudBtnWrapper>
                            <CrudBtn name={""} variant={Variant::Default} icon={Bi::ArrowRepeat} disabled={false} onclick={ctx.link().callback(|_| Msg::Reset)}>
                                <CrudBtnName>
                                    {"Reset"}
                                </CrudBtnName>
                            </CrudBtn>
                            <CrudBtn name={""} variant={Variant::Primary} icon={Bi::Search} disabled={true} onclick={ctx.link().callback(|_| Msg::ToggleFilter)}>
                                <CrudBtnName>
                                    {"Filter"}
                                    if self.filter.is_some() {
                                        <div style={"font-size: 0.5em; font-weight: bold; margin-left: 0.3em;"}>
                                            {"aktiv"}
                                        </div>
                                    }
                                </CrudBtnName>
                            </CrudBtn>
                        </CrudBtnWrapper>
                    </div>
                </div>

                <CrudTable<T::ReadModel>
                    children={ctx.props().children.clone()}
                    custom_fields={ctx.props().custom_fields.clone()}
                    api_base_url={ctx.props().config.api_base_url.clone()}
                    data={self.get_data()}
                    no_data={self.get_data_error()}
                    headers={ctx.props().config.headers.iter()
                        .map(|(field, options)| (field.clone(), options.clone(), ctx.props().config.order_by.get(field).cloned()))
                        .collect::<Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions, Option<Order>)>>()}
                    on_order_by={ctx.link().callback(Msg::OrderBy)}
                    read_allowed={true}
                    edit_allowed={true}
                    delete_allowed={true}
                    selected={self.selected.clone()}
                    on_selection={ctx.link().callback(Msg::EntrySelectionChanged)}
                    on_read={ctx.link().callback(Msg::Read)}
                    on_edit={ctx.link().callback(Msg::Edit)}
                    on_delete={ctx.link().callback(Msg::Delete)}
                    additional_item_actions={vec![]}
                    on_additional_item_action={ctx.link().callback(Msg::EntityActionTriggered)}
                />

                {
                    match self.selected.len() {
                        0 => html! {},
                        num_selected => html! {
                            <div class={"multiselect-actions"}>
                                <div>
                                    { num_selected } {"servers selected"}
                                </div>
                            </div>
                        },
                    }
                }

                {
                    match &self.item_count {
                        Ok(count) => html! {
                            <CrudPagination
                                current_page={ctx.props().config.page}
                                item_count={*count}
                                items_per_page={ctx.props().config.items_per_page}
                                on_page_select={ctx.link().callback(|page| Msg::PageSelected(page))}
                                on_item_count_select={ctx.link().callback(|page| Msg::ItemCountSelected(page))}
                            />
                        },
                        Err((reason, since)) => if (time::OffsetDateTime::now_utc() - *since).whole_seconds() > 5 {
                            html! {
                                <div>{format!("Keine Daten verfügbar: {reason:?}")}</div>
                            }
                        } else {
                            html! {}
                        },
                    }
                }
            </>
        }
    }
}
