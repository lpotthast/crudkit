use crate::dynamic::crud_action::CrudActionTrait;
use crate::dynamic::crud_list_view::CrudListViewContext;
use crate::dynamic::crud_table_body::CrudTableBody;
use crate::dynamic::crud_table_footer::CrudTableFooter;
use crate::dynamic::crud_table_header::CrudTableHeader;
use crate::dynamic::custom_field::CustomFields;
use crate::shared::crud_instance_config::DynSelectConfig;
use crudkit_shared::Order;
use crudkit_web::dynamic::prelude::*;
use indexmap::IndexMap;
use leptonic::components::table::{Table, TableContainer};
use leptos::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use crate::dynamic::crud_instance_config::Header;

#[derive(Debug, Clone, PartialEq)]
pub enum NoDataAvailable {
    NotYetLoaded,
    RequestFailed(RequestError),
    RequestReturnedNoData(String),
}

#[component]
pub fn CrudTable(
    #[prop(into)] headers: Signal<Vec<Header>>,
    #[prop(into)] order_by: Signal<IndexMap<AnyField, Order>>,     // ReadModel field
    #[prop(into)] data: Signal<Result<Arc<Vec<AnyModel>>, NoDataAvailable>>, // ReadModel
    #[prop(into)] custom_fields: Signal<CustomFields>,             // ReadModel
    #[prop(into)] field_config: Signal<HashMap<AnyField, DynSelectConfig>>, // ReadModel field
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
