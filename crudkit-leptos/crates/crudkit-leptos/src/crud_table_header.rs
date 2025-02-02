use std::marker::PhantomData;

use crudkit_shared::Order;
use crudkit_web::prelude::*;
use indexmap::IndexMap;
use leptonic::components::prelude::*;
use leptos::prelude::*;

use crate::{crud_instance::CrudInstanceContext, crud_list_view::CrudListViewContext};

#[component]
pub fn CrudTableHeader<T>(
    _phantom: PhantomData<T>,
    #[prop(into)] headers: Signal<Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions)>>,
    #[prop(into)] order_by: Signal<IndexMap<<T::ReadModel as CrudDataTrait>::Field, Order>>,
    // Whether an action column should be displayed.
    #[prop(into)] with_actions: Signal<bool>,
    /// Recommended to be set to false when the table body does not actually display content.
    with_select_column: Signal<bool>,
    // Whether all items displayed in the table are selected.
    all_selected: Signal<bool>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let instance_ctx = expect_context::<CrudInstanceContext<T>>();
    let list_ctx = expect_context::<CrudListViewContext<T>>();

    let update_order_of_field = move |field: <T::ReadModel as CrudDataTrait>::Field| {
        instance_ctx.oder_by(field, OrderByUpdateOptions { append: false })
    };

    view! {
        <TableHeader>
            <TableRow>
                {move || {
                    with_select_column
                        .get()
                        .then(|| {
                            view! {
                                <TableHeaderCell attr:class="select fit-content">
                                    <Checkbox checked=all_selected set_checked=move |checked| {
                                        if checked != all_selected.get_untracked() {
                                            list_ctx.toggle_select_all()
                                        }
                                    }/>
                                </TableHeaderCell>
                            }
                        })
                }}
                <For
                    each=move || headers.get()
                    key=|(field, _options)| field.get_name()
                    children=move |(field, options)| {
                        move || {
                            let field_clone = field.clone();
                            let order_by = order_by.get();
                            let order = order_by.get(&field).cloned();
                            let display_name = options.display_name.clone();
                            tracing::debug!(? field, ? order, "render header");
                            view! {
                                <TableHeaderCell
                                    class:crud-column-ordered=order.is_some()
                                    class:crud-order-by-trigger=options.ordering_allowed
                                    class:min-width=options.min_width
                                    on:click=move |_| {
                                        if options.ordering_allowed {
                                            update_order_of_field(field_clone.clone())
                                        }
                                    }
                                >
                                    {display_name.clone()}
                                    <span class="crud-order-by-sign" class:active=order.is_some()>
                                        <SafeHtml<String> html=match order {
                                            Some(order) => {
                                                match order {
                                                    Order::Asc => "&uarr;",
                                                    Order::Desc => "&darr;",
                                                }
                                            }
                                            None => "&uarr;",
                                        }/>
                                    </span>
                                </TableHeaderCell>
                            }
                        }
                    }
                />
                {move || { with_actions.get().then(|| view! { <TableHeaderCell attr:class="actions fit-content">"Aktionen"</TableHeaderCell> }) }}
            </TableRow>
        </TableHeader>
    }
}
