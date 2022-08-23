use crud_shared_types::Order;
use std::{marker::PhantomData, rc::Rc};
use yew::{prelude::*, html::ChildrenRenderer};
use yewbi::Bi;

use crate::crud_instance::Item;

use super::prelude::*;

pub enum Msg<T: CrudDataTrait> {
    OrderBy((T::Field, OrderByUpdateOptions)),
    Read(T),
    Edit(T),
    Delete(T),
    ActionTriggered((Rc<Box<dyn CrudActionTrait>>, T)),
}

#[derive(Properties, PartialEq)]
pub struct Props<T>
where
    T: CrudDataTrait,
{
    pub children: ChildrenRenderer<Item>,
    pub api_base_url: String,
    pub data: Option<Rc<Vec<T>>>,
    pub no_data: Option<NoData>,
    pub headers: Vec<(T::Field, HeaderOptions, Option<Order>)>,
    pub on_order_by: Callback<(T::Field, OrderByUpdateOptions)>,
    pub read_allowed: bool,
    pub edit_allowed: bool,
    pub delete_allowed: bool,
    pub on_read: Callback<T>,
    pub on_edit: Callback<T>,
    pub on_delete: Callback<T>,
    pub additional_item_actions: Vec<Rc<Box<dyn CrudActionTrait>>>,
    pub on_additional_item_action: Callback<(Rc<Box<dyn CrudActionTrait>>, T)>,
}

impl<T: 'static + CrudDataTrait> Props<T> {
    pub fn has_actions(&self) -> bool {
        !self.additional_item_actions.is_empty()
            || self.read_allowed
            || self.edit_allowed
            || self.delete_allowed
    }
}

pub struct CrudTable<T> {
    phantom: PhantomData<T>,
}

impl<T: 'static + CrudDataTrait> CrudTable<T> {
    pub fn create_read_callback<A>(
        entity: T,
    ) -> impl Fn(A) -> <CrudTable<T> as Component>::Message {
        move |_| -> Msg<T> { Msg::Read(entity.clone()) }
    }
    pub fn create_edit_callback<A>(
        entity: T,
    ) -> impl Fn(A) -> <CrudTable<T> as Component>::Message {
        move |_| -> Msg<T> { Msg::Edit(entity.clone()) }
    }
    pub fn create_delete_callback<A>(
        entity: T,
    ) -> impl Fn(A) -> <CrudTable<T> as Component>::Message {
        move |_| -> Msg<T> { Msg::Delete(entity.clone()) }
    }
}

impl<T> Component for CrudTable<T>
where
    T: 'static + CrudDataTrait,
{
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            phantom: PhantomData {},
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ActionTriggered((action, entity)) => {
                ctx.props().on_additional_item_action.emit((action, entity));
                false
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
            Msg::OrderBy(field) => {
                ctx.props().on_order_by.emit(field);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let has_actions = ctx.props().has_actions();
        html! {
            <div class={"crud-table-wrapper"}>
                <table class={"crud-table crud-table-bordered crud-table-hoverable"}>
                    <CrudTableHeader<T>
                        headers={ctx.props().headers.clone()}
                        on_order_by={ctx.link().callback(Msg::OrderBy)}
                        with_actions={has_actions}
                    />
                    <tbody>
                        {
                            if let Some(data) = &ctx.props().data {
                                match data.len() {
                                    0 => html! {
                                        <tr>
                                            <td colspan={"100%"} class={"no-data"}>
                                                {"Keine Daten"}
                                            </td>
                                        </tr>
                                    },
                                    _ => data.iter().map(|entity| {
                                        let cloned_entity = entity.clone();
                                        html! {
                                            <tr class={"interactable"}
                                                onclick={link.callback(move |_| Msg::Edit(cloned_entity.clone()))}
                                            >
                                                {
                                                    ctx.props().headers.iter().map(|(field, options, _order)| {
                                                        html! {
                                                            <td>
                                                                <CrudField<T>
                                                                    children={ctx.props().children.clone()}
                                                                    api_base_url={ctx.props().api_base_url.clone()}
                                                                    current_view={CrudView::List}
                                                                    field_type={field.clone()}
                                                                    field_options={FieldOptions { disabled: false, label: None, date_time_display: options.date_time_display }}
                                                                    entity={entity.clone()}
                                                                    field_mode={FieldMode::Display}
                                                                    value_changed={|_| {}}
                                                                />
                                                            </td>
                                                        }
                                                    }).collect::<Html>()
                                                }
                                                if has_actions {
                                                    <td>
                                                        <div class={"action-icons"}>
                                                            if ctx.props().read_allowed {
                                                                <div
                                                                    class={"action-icon"}
                                                                    onclick={link.callback(CrudTable::<T>::create_read_callback(entity.clone()))}
                                                                >
                                                                    <CrudIcon variant={Bi::Eye}/>
                                                                </div>
                                                            }
                                                            if ctx.props().edit_allowed {
                                                                <div
                                                                    class={"action-icon"}
                                                                    onclick={link.callback(CrudTable::<T>::create_edit_callback(entity.clone()))}
                                                                >
                                                                    <CrudIcon variant={Bi::Pencil}/>
                                                                </div>
                                                            }
                                                            if ctx.props().delete_allowed {
                                                                <div
                                                                    class={"action-icon"}
                                                                    onclick={link.callback(CrudTable::<T>::create_delete_callback(entity.clone()))}
                                                                >
                                                                    <CrudIcon variant={Bi::Trash}/>
                                                                </div>
                                                            }
                                                        {
                                                            ctx.props().additional_item_actions.iter().map(|action| {
                                                                // TODO: can we eliminate some clone()'s?
                                                                let cloned_action = action.clone();
                                                                let cloned_entity = entity.clone();
                                                                html! {
                                                                    <div
                                                                        class={"action-icon"}
                                                                        onclick={link.callback(move |_| Msg::ActionTriggered((cloned_action.clone(), cloned_entity.clone())))}>
                                                                        <CrudIcon variant={action.get_icon().unwrap_or(Bi::Question)}/>
                                                                    </div>
                                                                }
                                                            }).collect::<Html>()
                                                        }
                                                        </div>
                                                    </td>
                                                }
                                            </tr>
                                        }
                                    }).collect::<Html>()
                                }
                            }
                            else if let Some(no_data) = &ctx.props().no_data {
                                html! {
                                    <tr>
                                        <td colspan={"100%"}>
                                            {format!("No data available: {:?}", no_data)}
                                        </td>
                                    </tr>
                                }
                            }
                            else {
                                html! { "Component misconfigured: Either pass some data or an error, not both." }
                            }
                        }
                    </tbody>
                    <CrudTableFooter />
                </table>
            </div>
        }
    }
}
