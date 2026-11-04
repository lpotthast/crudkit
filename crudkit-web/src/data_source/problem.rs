//! The problems with which a data source refuses an entity, field by field.

/// A problem with one field of an entity, e.g. an entry of the `errors` of problem details
/// (RFC 9457) that names the field by a JSON pointer.
///
/// A data source refuses an entity with its problems in
/// [`RequestError::InvalidFields`](crate::http::RequestError::InvalidFields). A form shows each
/// problem at the field it names, and the problems of fields that it does not show with the
/// refusal's message.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CrudFieldProblem {
    /// The name of the field, as the model's field enum names it (`Named::name`), e.g. `name`. A
    /// name that the model does not have is shown with the refusal's message.
    pub field: String,
    /// What is wrong with the field, and what to do.
    pub detail: String,
}

impl CrudFieldProblem {
    /// A problem with `field`, which `detail` describes.
    pub fn new(field: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            detail: detail.into(),
        }
    }
}
