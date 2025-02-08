use std::borrow::Cow;
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
    let select_all = Callback::from(move || {
        list_ctx.toggle_select_all();
    });
    view! {
        <TableHeader>
            <TableRow>
                {move || {
                    with_select_column.get().then(move || view! { <SelectAll all_selected select_all/> })
                }}
                <For
                    each=move || headers.get()
                    key=|(field, _options)| field.get_name()
                    children=move |(field, options)| {
                        move || {
                            let field_clone = field.clone();
                            let update_order = Callback::from(move || { update_order_of_field(field_clone.clone()); });
                            view! {
                                <Header name=options.display_name.clone() order=order_by.read().get(&field).cloned() ordering_allowed=options.ordering_allowed update_order apply_min_width_class=options.min_width/>
                            }
                        }
                    }
                />
                {move || { with_actions.get().then(|| view! { <TableHeaderCell attr:class="actions fit-content">"Aktionen"</TableHeaderCell> }) }}
            </TableRow>
        </TableHeader>
    }
}

#[component]
fn SelectAll(all_selected: Signal<bool>, select_all: Callback<()>) -> impl IntoView {
    view! {
        <TableHeaderCell attr:class="select fit-content">
            <Checkbox checked=all_selected set_checked=move |checked| {
                if checked != all_selected.get_untracked() {
                    select_all.run(())
                }
            }/>
        </TableHeaderCell>
    }
}

#[component]
fn Header(
    name: Cow<'static, str>,
    order: Option<Order>,
    ordering_allowed: bool,
    update_order: Callback<()>,
    apply_min_width_class: bool,
) -> impl IntoView {
    view! {
        <TableHeaderCell
            class:crud-column-ordered=order.is_some()
            class:crud-order-by-trigger=ordering_allowed
            class:min-width=apply_min_width_class
            on:click=move |_| {
                if ordering_allowed {
                    update_order.run(())
                }
            }
        >
            { name }
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
