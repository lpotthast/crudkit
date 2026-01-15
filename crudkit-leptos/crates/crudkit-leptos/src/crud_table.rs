use crate::crud_action::CrudActionTrait;
use crate::crud_instance_config::{FieldRendererRegistry, Header};
use crate::crud_list_view::CrudListViewContext;
use crate::crud_table_body::CrudTableBody;
use crate::crud_table_footer::CrudTableFooter;
use crate::crud_table_header::CrudTableHeader;
use crudkit_core::Order;
use crudkit_web::prelude::{AnyReadField, AnyReadModel};
use crudkit_web::request_error::RequestError;
use indexmap::IndexMap;
use leptonic::components::table::{Table, TableContainer};
use leptos::prelude::*;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum NoDataAvailable {
    NotYetLoaded,
    RequestFailed(RequestError),
    RequestReturnedNoData(String),
}

#[component]
pub fn CrudTable(
    #[prop(into)] headers: Signal<Vec<Header>>,
    #[prop(into)] order_by: Signal<IndexMap<AnyReadField, Order>>,
    #[prop(into)] data: Signal<Result<Arc<Vec<AnyReadModel>>, NoDataAvailable>>,
    #[prop(into)] field_renderer_registry: Signal<FieldRendererRegistry<AnyReadField>>,
    #[prop(into)] read_allowed: Signal<bool>,
    #[prop(into)] edit_allowed: Signal<bool>,
    #[prop(into)] delete_allowed: Signal<bool>,
    #[prop(into)] additional_item_actions: Signal<Vec<Arc<Box<dyn CrudActionTrait>>>>, // TODO: Use AnyAction
) -> impl IntoView {
    let list_ctx = expect_context::<CrudListViewContext>();

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
                    headers=headers
                    order_by=order_by
                    with_actions=with_actions
                    with_select_column=list_ctx.has_data
                    all_selected=list_ctx.all_selected
                />

                <CrudTableBody
                    data=data
                    headers=headers
                    field_renderer_registry=field_renderer_registry
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
