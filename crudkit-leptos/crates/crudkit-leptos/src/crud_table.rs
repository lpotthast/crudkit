use std::{marker::PhantomData, rc::Rc, collections::HashMap};

use crudkit_shared::Order;
use crudkit_web::prelude::*;
use indexmap::IndexMap;
use leptos::*;

// TODO: Add prelude entry for CrudActionTrait
use crate::{
    crud_action::CrudActionTrait,
    crud_list_view::CrudListViewContext,
    prelude::{CrudTableBody, CrudTableFooter, CrudTableHeader}, crud_instance_config::DynSelectConfig,
};

#[derive(Debug, Clone, PartialEq)]
pub enum NoDataAvailable {
    NotYetLoaded,
    RequestFailed(RequestError),
    RequestReturnedNoData(String),
}

#[component]
pub fn CrudTable<T>(
    cx: Scope,
    _phantom: PhantomData<T>,
    #[prop(into)] api_base_url: Signal<String>,
    #[prop(into)] headers: Signal<Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions)>>,
    #[prop(into)] order_by: Signal<IndexMap<<T::ReadModel as CrudDataTrait>::Field, Order>>,
    #[prop(into)] data: Signal<Result<Rc<Vec<T::ReadModel>>, NoDataAvailable>>,
    #[prop(into)] custom_fields: Signal<CustomFields<T::ReadModel, leptos::View>>,
    #[prop(into)] field_config: Signal<HashMap<<T::ReadModel as CrudDataTrait>::Field, DynSelectConfig>>,
    #[prop(into)] read_allowed: Signal<bool>,
    #[prop(into)] edit_allowed: Signal<bool>,
    #[prop(into)] delete_allowed: Signal<bool>,
    #[prop(into)] additional_item_actions: Signal<Vec<Rc<Box<dyn CrudActionTrait>>>>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let list_ctx: CrudListViewContext<T> = expect_context::<CrudListViewContext<T>>(cx);

    let with_actions = Signal::derive(cx, move || {
        !additional_item_actions.get().is_empty()
            || read_allowed.get()
            || edit_allowed.get()
            || delete_allowed.get()
    });

    // TODO: Extract to leptonic
    view! {cx,
        <div class="crud-table-wrapper">
            <table class="crud-table crud-table-bordered crud-table-hoverable">
                <CrudTableHeader
                    _phantom={PhantomData::<T>::default()}
                    headers=headers
                    order_by=order_by
                    with_actions=with_actions
                    with_select_column=list_ctx.has_data
                    all_selected=list_ctx.all_selected
                />

                <CrudTableBody
                    _phantom={PhantomData::<T>::default()}
                    data=data
                    api_base_url=api_base_url
                    headers=headers
                    custom_fields=custom_fields
                    field_config=field_config
                    read_allowed=read_allowed
                    edit_allowed=edit_allowed
                    delete_allowed=delete_allowed
                    additional_item_actions=Signal::derive(cx, move || vec![])
                />

                <CrudTableFooter />
            </table>
        </div>
    }
}
