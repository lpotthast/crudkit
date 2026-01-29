use std::sync::Arc;

use crate::resource::CrudResource;
use crate::validate::GlobalValidationState;
use crate::validator::{AggregateValidator, EntityValidator};

/// The context is made available to any crud operation.
/// It may contain state relevant to its associated CrudResource.
pub struct CrudContext<R: CrudResource> {
    pub res_context: Arc<R::Context>,
    pub repository: Arc<R::Repository>,
    pub validators: Vec<Arc<dyn EntityValidator<R>>>,
    pub resource_validators: Vec<Arc<dyn AggregateValidator<R>>>,
    pub validation_result_repository: Arc<R::ValidationResultRepository>,
    pub collab_service: Arc<R::CollaborationService>,
    pub global_validation_state: Arc<GlobalValidationState>,
}
