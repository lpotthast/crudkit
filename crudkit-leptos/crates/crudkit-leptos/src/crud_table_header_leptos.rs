use std::marker::PhantomData;

use crudkit_shared::Order;
use crudkit_web::prelude::*;
use leptonic::prelude::*;
use leptos::*;
use leptos_icons::BsIcon;

use crate::prelude::CrudSafeHtmlL;

#[component]
pub fn CrudTableHeaderL<T>(
    cx: Scope,
    phantom: PhantomData<T>,
    #[prop(into)] headers: Signal<Vec<(T::Field, HeaderOptions, Option<Order>)>>,
    // Whether or not an action column should be displayed.
    #[prop(into)] with_actions: Signal<bool>,
    /// Recommended to be set to false when the table body does not actually display content.
    with_select_column: Signal<bool>,
    // Wether or not all items displayed in the table are selected.
    all_selected: Signal<bool>,
    //on_order_by: O,
    //on_select_all: S,
) -> impl IntoView
where
    T: CrudDataTrait + 'static,
    //O: Fn((T::Field, OrderByUpdateOptions)) -> () + 'static, // order by
    //S: Fn(bool) -> () + 'static, // select all
{
    let order_by = move |field: <T as CrudDataTrait>::Field, options: HeaderOptions| {

    };

    // TODO: Implement
    let toggle_select_all = move || {};

    view! {cx,
        <thead class={"crud-table-header"}>
            <tr>
                { move || {
                    if with_select_column.get() {
                        view! {cx,
                            <th class="select min-width">
                                <Checkbox checked=all_selected on_toggle=toggle_select_all />
                            </th>
                        }.into_view(cx)
                    } else {
                        ().into_view(cx)
                    }
                }}

                <For
                    each=headers
                    key=|(field, options, order)| field.get_name()
                    view=move |cx, (field, options, order)| {
                        let field_clone = field.clone();
                        let options_clone = options.clone();

                        view! { cx,
                            <th
                                class="crud-column-header"
                                class:crud-column-ordered=order.is_some()
                                class:crud-order-by-trigger=options.ordering_allowed
                                class:min-width=options.min_width
                                on:click=move |_| order_by(field_clone.clone(), options_clone.clone())
                            >
                                <div class="crud-row">
                                    <div class="crud-col crud-col-flex-start crud-col-flex-top crud-column-header-main-row">
                                        {options.display_name.clone()}

                                        <span class="crud-order-by-sign" class:active=order.is_some()>
                                            <CrudSafeHtmlL html={
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
                    }
                />

                { move || {
                    if with_actions.get() {
                        view! {cx,
                            <th class="actions min-width">
                                "Aktionen"
                            </th>
                        }.into_view(cx)
                    } else {
                        ().into_view(cx)
                    }
                }}
            </tr>
        </thead>
    }
}
