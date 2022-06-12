use super::prelude::*;
use crud_shared_types::Order;
use std::marker::PhantomData;
use yew::prelude::*;

pub enum Msg<T: CrudDataTrait> {
    OrderBy((T::FieldType, HeaderOptions)),
}

#[derive(Properties, PartialEq)]
pub struct Props<T>
where
    T: CrudDataTrait,
{
    pub headers: Vec<(T::FieldType, HeaderOptions, Option<Order>)>,
    pub on_order_by: Callback<(T::FieldType, OrderByUpdateOptions)>,
    pub with_actions: bool,
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <thead class={"crud-table-header"}>
                <tr>
                    {
                        ctx.props().headers.iter().map(|(field, options, order)| {
                            let mut classes = classes!("crud-column-header");
                            if order.is_some() {
                                classes.push("crud-column-ordered");
                            }
                            if options.ordering_allowed {
                                classes.push("crud-order-by-trigger");
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
                                            {match order {
                                                Some(order) => html! {
                                                    <span class="crud-order-by-sign">
                                                        {match order {
                                                            Order::Asc => html!{ <CrudSafeHtml html={"&uarr;"} /> },
                                                            Order::Desc => html!{ <CrudSafeHtml html={"&darr;"} /> },
                                                        }}
                                                    </span>
                                                },
                                                None => html!{},
                                            }}
                                            {options.display_name.clone()}
                                        </div>
                                    </div>
                                </th>
                            }
                        }).collect::<Html>()
                    }
                    if ctx.props().with_actions {
                        <th class={"actions"}>
                            {"Aktionen"}
                        </th>
                    }
                </tr>
            </thead>
        }
    }
}
