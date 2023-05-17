use crate::prelude::*;
use crudkit_web::prelude::*;
use crudkit_shared::Order;
use std::marker::PhantomData;
use yew::prelude::*;

pub enum Msg<T: CrudDataTrait> {
    OrderBy((T::Field, HeaderOptions)),
    SelectAll(bool),
}

#[derive(Properties, PartialEq)]
pub struct Props<T>
where
    T: CrudDataTrait,
{
    pub headers: Vec<(T::Field, HeaderOptions, Option<Order>)>,
    pub on_order_by: Callback<(T::Field, OrderByUpdateOptions)>,
    pub with_actions: bool,
    /// Should be true if all entities are selected.
    pub with_select_column: bool,
    pub all_selected: bool,
    pub on_select_all: Callback<bool>,
}

pub struct CrudTableHeader<T> {
    phantom: PhantomData<T>,
}

impl<T: 'static + CrudDataTrait> Component for CrudTableHeader<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            phantom: PhantomData {},
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::OrderBy((field, options)) => {
                if options.ordering_allowed {
                    ctx.props()
                        .on_order_by
                        .emit((field, OrderByUpdateOptions { append: false }));
                }
                false
            }
            Msg::SelectAll(state) => {
                ctx.props().on_select_all.emit(state);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <thead class={"crud-table-header"}>
                <tr>
                    if ctx.props().with_select_column {
                        <th class={"select min-width"}>
                            <CrudCheckbox state={ctx.props().all_selected} on_toggle={ctx.link().callback(Msg::SelectAll)}/>
                        </th>
                    }

                    {
                        ctx.props().headers.iter().map(|(field, options, order)| {
                            let mut classes = classes!("crud-column-header");
                            if order.is_some() {
                                classes.push("crud-column-ordered");
                            }
                            if options.ordering_allowed {
                                classes.push("crud-order-by-trigger");
                            }
                            if options.min_width {
                                classes.push("min-width");
                            }

                            let field_clone = field.clone();
                            let options_clone = options.clone();
                            html! {
                                <th
                                    class={classes}
                                    onclick={ctx.link().callback(move |_| Msg::OrderBy((field_clone.clone(), options_clone.clone())))}
                                >
                                    <div class={"crud-row"}>
                                        <div class={"crud-col crud-col-flex-start crud-col-flex-top crud-column-header-main-row"}>
                                            {options.display_name.clone()}

                                            <span class={classes!("crud-order-by-sign", order.is_some().then(|| "active"))}>
                                                <CrudSafeHtml html={
                                                    match order {
                                                        Some(order) => match order {
                                                            Order::Asc => "&uarr;",
                                                            Order::Desc => "&darr;",
                                                        },
                                                        None => "&uarr;",
                                                    }
                                                } />
                                            </span>

                                        </div>
                                    </div>
                                </th>
                            }
                        }).collect::<Html>()
                    }
                    if ctx.props().with_actions {
                        <th class={"actions min-width"}>
                            {"Aktionen"}
                        </th>
                    }
                </tr>
            </thead>
        }
    }
}
