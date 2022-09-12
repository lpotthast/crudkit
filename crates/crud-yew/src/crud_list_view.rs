use crud_shared_types::Order;
use std::rc::Rc;
use yew::{prelude::*, html::ChildrenRenderer};
use yewbi::Bi;

use crate::{crud_instance::Item, services::crud_rest_data_provider::{CrudRestDataProvider, ReadMany, ReadCount}};

use super::{
    prelude::*,
    types::RequestError,
};

pub enum Msg<T: CrudMainTrait> {
    ComponentCreated,
    PageSelected(u64),
    PageLoaded(Result<Vec<T::ReadModel>, RequestError>),
    CountRead(Result<usize, RequestError>),
    ToggleFilter,
    OrderBy((<T::ReadModel as CrudDataTrait>::Field, OrderByUpdateOptions)),
    Create,
    Read(T::ReadModel),
    Edit(T::ReadModel),
    Delete(T::ReadModel),
    ActionTriggered((Rc<Box<dyn CrudActionTrait>>, T::ReadModel)),
}

#[derive(Properties, PartialEq)]
pub struct Props<T: CrudMainTrait> {
    pub children: ChildrenRenderer<Item>,
    pub data_provider: CrudRestDataProvider<T>,
    pub config: CrudInstanceConfig<T>,
    pub on_create: Callback<()>,
    pub on_read: Callback<T::UpdateModel>,
    pub on_edit: Callback<T::UpdateModel>,
    pub on_delete: Callback<T::UpdateModel>,
    pub on_order_by: Callback<(<T::ReadModel as CrudDataTrait>::Field, OrderByUpdateOptions)>,
    pub on_page_selected: Callback<u64>,
    pub on_action: Callback<(Rc<Box<dyn CrudActionTrait>>, T::ReadModel)>,
}

pub struct CrudListView<T: 'static + CrudMainTrait> {
    data: Result<Rc<Vec<T::ReadModel>>, NoData>,
    filter: Option<()>,
    item_count: Result<u64, NoData>,
}

impl<T: CrudMainTrait> CrudListView<T> {
    fn load_page(&self, ctx: &Context<CrudListView<T>>) {
        let order_by = ctx.props().config.order_by.clone();
        let page = ctx.props().config.page as u64;
        let items_per_page = ctx.props().config.items_per_page as u64;
        let data_provider = ctx.props().data_provider.clone();
        ctx.link().send_future(async move {
            Msg::PageLoaded(
                data_provider.read_many(ReadMany {
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
            Msg::CountRead(data_provider.read_count(ReadCount { condition: None }).await)
        });
    }

    fn get_data(&self) -> Option<Rc<Vec<T::ReadModel>>> {
        match &self.data {
            Ok(data) => Some(data.clone()),
            Err(_) => None,
        }
    }

    fn get_data_error(&self) -> Option<NoData> {
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
        ctx.link().send_future(async move { Msg::ComponentCreated });
        Self {
            data: Err(NoData::NotYetLoaded),
            filter: None,
            item_count: Err(NoData::NotYetLoaded),
        }
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
            Msg::PageLoaded(data) => {
                self.data = data.map(Rc::new).map_err(NoData::FetchFailed);
                true
            }
            Msg::CountRead(data) => {
                self.item_count = data.map_err(NoData::FetchFailed).map(|val| val as u64);
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
            Msg::Read(entity) => {
                ctx.props().on_read.emit(entity.into());
                false
            }
            Msg::Edit(entity) => {
                ctx.props().on_edit.emit(entity.into());
                false
            }
            Msg::Delete(entity) => {
                ctx.props().on_delete.emit(entity.into());
                false
            }
            Msg::ActionTriggered((action, entity)) => {
                ctx.props().on_action.emit((action, entity));
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
                        </CrudBtnWrapper>
                    </div>

                    <div class={"crud-col crud-col-flex-end"}>
                        <CrudBtnWrapper>
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
                    on_read={ctx.link().callback(Msg::Read)}
                    on_edit={ctx.link().callback(Msg::Edit)}
                    on_delete={ctx.link().callback(Msg::Delete)}
                    additional_item_actions={vec![]}
                    on_additional_item_action={ctx.link().callback(Msg::ActionTriggered)}
                />

                {
                    match &self.item_count {
                        Ok(count) => html! {
                            <CrudPagination
                                current_page={ctx.props().config.page}
                                item_count={*count}
                                items_per_page={ctx.props().config.items_per_page}
                                on_page_select={ctx.link().callback(|page| Msg::PageSelected(page))}
                            />
                        },
                        Err(no_data) => html! {
                            <div>{format!("Keine Daten verfügbar: {:?}", no_data)}</div>
                        },
                    }
                }
            </>
        }
    }
}
