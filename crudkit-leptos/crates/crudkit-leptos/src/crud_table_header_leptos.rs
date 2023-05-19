use std::marker::PhantomData;

use crudkit_shared::Order;
use crudkit_web::prelude::*;
use leptonic::prelude::*;
use leptos::*;

use crate::{
    crud_instance_leptos::CrudInstanceContext, crud_list_view_leptos::CrudListViewContext,
    prelude::CrudSafeHtmlL,
};

#[component]
pub fn CrudTableHeaderL<T>(
    cx: Scope,
    _phantom: PhantomData<T>,
    #[prop(into)] headers: Signal<
        Vec<(
            <T::ReadModel as CrudDataTrait>::Field,
            HeaderOptions,
            Option<Order>,
        )>,
    >,
    // Whether or not an action column should be displayed.
    #[prop(into)] with_actions: Signal<bool>,
    /// Recommended to be set to false when the table body does not actually display content.
    with_select_column: Signal<bool>,
    // Wether or not all items displayed in the table are selected.
    all_selected: Signal<bool>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    view! {cx,
        <thead class={"crud-table-header"}>
            <tr>
                { move || {
                    if with_select_column.get() {
                        view! {cx,
                            <th class="select min-width">
                                <Checkbox checked=all_selected on_toggle=move || { expect_context::<CrudListViewContext<T>>(cx).toggle_select_all() } />
                            </th>
                        }.into_view(cx)
                    } else {
                        ().into_view(cx)
                    }
                }}

                // TODO: Consider using <For>. But this led to strange behavior last time...
                { move || headers.get().iter().map(|(field, options, order)| {
                    let field_clone = field.clone();
                    // tracing::debug!(?field, ?order, "render header");
                    view! { cx,
                        <th
                            class="crud-column-header"
                            class:crud-column-ordered=order.is_some()
                            class:crud-order-by-trigger=options.ordering_allowed
                            class:min-width=options.min_width
                            on:click=move |_| { expect_context::<CrudInstanceContext<T>>(cx).oder_by(field_clone.clone(), OrderByUpdateOptions { append: false }) }
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
                    }.into_view(cx)
                }).collect::<leptos::View>() }

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
