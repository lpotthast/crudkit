use std::marker::PhantomData;

use crudkit_shared::Order;
use crudkit_web::prelude::*;
use leptonic::prelude::*;
use leptos::*;
use leptos_icons::BsIcon;

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
    view! {cx,
        <thead class={"crud-table-header"}>
            <tr>

            { move || {
                if with_actions.get() {
                    view! {cx,
                        <th class={"actions min-width"}>
                            {"Aktionen"}
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
