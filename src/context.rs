use crate::resource::CrudResource;

/// The context is made available to any crud operation.
/// It may contain state relevant to its associated CrudResource.
pub struct CrudContext<R: CrudResource> {
    pub validator: R::Validator,
    pub validation_result_repository: R::ValidationResultRepository,
}
