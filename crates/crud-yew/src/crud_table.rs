use chrono_utc_date_time::UtcDateTime;
use crud_shared_types::Order;
use gloo::timers::callback::Interval;
use std::{marker::PhantomData, rc::Rc};
use yew::{html::ChildrenRenderer, prelude::*};
use yewbi::Bi;

use crate::crud_instance::Item;

use super::prelude::*;

const MILLIS_UNTIL_ERROR_IS_SHOWN: u32 = 1000;

pub enum Msg<T: CrudDataTrait> {
    OrderBy((T::Field, OrderByUpdateOptions)),
    Read(T),
    Edit(T),
    Delete(T),
    ActionTriggered((Rc<Box<dyn CrudActionTrait>>, T)),
    SetError(NoData),
}

#[derive(Properties, PartialEq)]
pub struct Props<T>
where
    T: CrudDataTrait,
{
    pub children: ChildrenRenderer<Item>,
    pub api_base_url: String,
    pub data: Option<Rc<Vec<T>>>,
    pub no_data: Option<(NoData, UtcDateTime)>,
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
    error: Option<NoData>,
    clock_handle: Option<Interval>,
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
            error: None,
            clock_handle: None,
            phantom: PhantomData {},
        }
    }

    /// Checks whether or not the "no_data" property changed. If that is the case:
    /// And data is present: Creates a new clock, which waits `MILLIS_UNTIL_ERROR_IS_SHOWN` milliseconds and displays the error.
    /// And data is not present: Removes the error and any leftover clock.
    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        if old_props.no_data != ctx.props().no_data {
            match &ctx.props().no_data {
                Some((no_data, _since)) => {
                    let clock_handle = {
                        let link = ctx.link().clone();
                        let no_data = no_data.clone();
                        Interval::new(MILLIS_UNTIL_ERROR_IS_SHOWN, move || link.send_message(Msg::SetError(no_data.clone())))
                    };
                    self.clock_handle = Some(clock_handle);
                },
                None => {
                    self.error = None;
                    self.clock_handle = None;
                },
            }
        }
        true
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
            Msg::SetError(no_data) => {
                self.error = Some(no_data);
                self.clock_handle = None;
                true
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
                                                                    current_view={CrudSimpleView::List}
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
                            else if let Some((_reason, _since)) = &ctx.props().no_data {
                                if self.error.is_none() {
                                    // Error is not yet set! We just display a single empty row.
                                    html! {
                                        <tr>
                                            <td colspan={"100%"}>
                                                {"\u{00a0}"} // nbsp, see https://doc.rust-lang.org/std/primitive.char.html
                                            </td>
                                        </tr>
                                    }
                                } else {
                                    // Error is present but handled below!
                                    html! {}
                                }
                            }
                            else {
                                html! { "Component misconfigured: Either pass some data or an error, not both." }
                            }
                        }
                        {
                            if let Some(reason) = &self.error {
                                html! {
                                    <tr>
                                        <td colspan={"100%"}>
                                            {format!("No data available: {reason:?}")}
                                        </td>
                                    </tr>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </tbody>
                    <CrudTableFooter />
                </table>
            </div>
        }
    }
}
