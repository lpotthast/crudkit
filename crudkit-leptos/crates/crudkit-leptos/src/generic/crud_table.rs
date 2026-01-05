use crate::generic::crud_action::CrudActionTrait;
use crate::generic::crud_list_view::CrudListViewContext;
use crate::generic::crud_table_body::CrudTableBody;
use crate::generic::crud_table_footer::CrudTableFooter;
use crate::generic::crud_table_header::CrudTableHeader;
use crate::generic::custom_field::CustomFields;
use crate::shared::crud_instance_config::DynSelectConfig;
use crudkit_shared::Order;
use crudkit_web::generic::prelude::*;
use indexmap::IndexMap;
use leptonic::components::table::{Table, TableContainer};
use leptos::prelude::*;
use std::sync::Arc;
use std::{collections::HashMap, marker::PhantomData};

#[derive(Debug, Clone, PartialEq)]
pub enum NoDataAvailable {
    NotYetLoaded,
    RequestFailed(RequestError),
    RequestReturnedNoData(String),
}

#[component]
pub fn CrudTable<T>(
    _phantom: PhantomData<T>,
    #[prop(into)] headers: Signal<Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions)>>,
    #[prop(into)] order_by: Signal<IndexMap<<T::ReadModel as CrudDataTrait>::Field, Order>>,
    #[prop(into)] data: Signal<Result<Arc<Vec<T::ReadModel>>, NoDataAvailable>>,
    #[prop(into)] custom_fields: Signal<CustomFields<T::ReadModel>>,
    #[prop(into)] field_config: Signal<
        HashMap<<T::ReadModel as CrudDataTrait>::Field, DynSelectConfig>,
    >,
    #[prop(into)] read_allowed: Signal<bool>,
    #[prop(into)] edit_allowed: Signal<bool>,
    #[prop(into)] delete_allowed: Signal<bool>,
    #[prop(into)] additional_item_actions: Signal<Vec<Arc<Box<dyn CrudActionTrait>>>>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let list_ctx = expect_context::<CrudListViewContext<T>>();

    let with_actions = Signal::derive(move || {
        !additional_item_actions.get().is_empty()
            || read_allowed.get()
            || edit_allowed.get()
            || delete_allowed.get()
    });

    // TODO: Extract to leptonic
    view! {
        <TableContainer>
            <Table bordered=true hoverable=true>
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
                    headers=headers
                    custom_fields=custom_fields
                    field_config=field_config
                    read_allowed=read_allowed
                    edit_allowed=edit_allowed
                    delete_allowed=delete_allowed
                    additional_item_actions=Signal::derive(move || vec![])
                />

                <CrudTableFooter/>
            </Table>
        </TableContainer>
    }
}
