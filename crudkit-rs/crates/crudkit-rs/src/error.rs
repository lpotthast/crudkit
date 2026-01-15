use crate::repository::RepositoryError;
use crudkit_condition::IntoAllEqualConditionError;
use snafu::Snafu;
use std::sync::Arc;

// TODO: We should ensure that the error printed and/or serialized down to any frontend does not disclose secret information!

/// Error type for CRUD operations.
///
/// This enum represents all possible errors that can occur during CRUD operations.
/// Each variant maps to a specific HTTP status code for API responses.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum CrudError {
    // =========================================================================
    // Permission/Authorization Errors (HTTP 403)
    // =========================================================================
    /// Permission or authorization denied by lifecycle hook.
    ///
    /// Mapped to HTTP status 403 Forbidden.
    #[snafu(display("CrudError: Forbidden. {reason}"))]
    Forbidden { reason: String },

    // =========================================================================
    // Business Logic/Validation Errors (HTTP 422)
    // =========================================================================
    /// Business logic rejection by lifecycle hook.
    ///
    /// Mapped to HTTP status 422 Unprocessable Entity.
    #[snafu(display("CrudError: Unprocessable entity. {reason}"))]
    UnprocessableEntity { reason: String },

    /// Critical validation errors prevent the operation.
    ///
    /// Mapped to HTTP status 422 Unprocessable Entity.
    #[snafu(display("CrudError: Validation failed with critical errors."))]
    ValidationFailed,

    // =========================================================================
    // Not Found Errors (HTTP 404)
    // =========================================================================
    /// Entity not found.
    ///
    /// Mapped to HTTP status 404 Not Found.
    #[snafu(display("CrudError: Entity not found."))]
    NotFound,

    // =========================================================================
    // Client Errors (HTTP 400)
    // =========================================================================
    /// Could not convert to condition (invalid query parameters).
    ///
    /// Mapped to HTTP status 400 Bad Request.
    #[snafu(display("CrudError: Could not convert to condition."))]
    IntoCondition { source: IntoAllEqualConditionError },

    // =========================================================================
    // Server Errors (HTTP 500)
    // =========================================================================
    /// Repository/database error.
    ///
    /// Mapped to HTTP status 500 Internal Server Error.
    #[snafu(display("CrudError: Repository error.\n\nCaused by:\n{reason:?}"))]
    Repository { reason: Arc<dyn RepositoryError> },

    /// Lifecycle hook internal error.
    ///
    /// Mapped to HTTP status 500 Internal Server Error.
    #[snafu(display("CrudError: Lifecycle hook error. {reason}"))]
    LifecycleError { reason: String },

    /// Could not save validation results.
    ///
    /// Mapped to HTTP status 500 Internal Server Error.
    #[snafu(display("CrudError: Could not save validations.\n\nCaused by:\n{reason:?}"))]
    SaveValidations { reason: Arc<dyn RepositoryError> },

    /// Could not delete validation results.
    ///
    /// Mapped to HTTP status 500 Internal Server Error.
    #[snafu(display("CrudError: Could not delete validations.\n\nCaused by:\n{reason:?}"))]
    DeleteValidations { reason: Arc<dyn RepositoryError> },
}
