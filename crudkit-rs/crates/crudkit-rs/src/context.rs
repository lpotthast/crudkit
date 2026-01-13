use std::sync::Arc;

use crate::resource::CrudResource;

/// The context is made available to any crud operation.
/// It may contain state relevant to its associated CrudResource.
pub struct CrudContext<R: CrudResource> {
    pub res_context: Arc<R::Context>,
    pub repository: Arc<R::Repository>,
    pub validator: R::Validator,
    pub validation_result_repository: Arc<R::ValidationResultRepository>,
    pub ws_controller: Arc<R::WebsocketService>,
}
