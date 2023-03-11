use std::sync::Arc;

use crate::resource::CrudResource;

/// The context is made available to any crud operation.
/// It may contain state relevant to its associated CrudResource.
pub struct CrudContext<R: CrudResource> {
    pub repository: Arc<R::Repository>,
    pub validator: R::Validator,
    pub validation_result_repository: Arc<R::ValidationResultRepository>,
    pub ws_controller: Arc<R::WebsocketController>,
}

/// Every crud resource needs its own resource context in which any data imaginable can be presented.
/// This context is used in some operations specific to this contexts resource.
pub trait CrudResourceContext {}
