use crate::repository::RepositoryError;
use crudkit_condition::IntoAllEqualConditionError;
use crudkit_validation::PartialSerializableAggregateViolations;
use snafu::Snafu;
use std::sync::Arc;

/// Error type for CRUD operations.
///
/// This enum represents all possible errors that can occur during CRUD operations.
/// Each variant maps to a specific HTTP status code for API responses.
///
/// # Logging vs HTTP Responses
///
/// - Use `{error:?}` (Debug) for logging - shows full internal details
/// - Convert to `AxumCrudError` for HTTP responses - exposes only minimal user-safe messages
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum CrudError {
    // =========================================================================
    // Permission/Authorization Errors (HTTP 403)
    // =========================================================================
    /// Permission or authorization denied by lifecycle hook.
    ///
    /// Mapped to HTTP status 403 Forbidden.
    #[snafu(display("Forbidden: {reason}"))]
    Forbidden { reason: String },

    // =========================================================================
    // Business Logic/Validation Errors (HTTP 422)
    // =========================================================================
    /// Business logic rejection by lifecycle hook.
    ///
    /// Mapped to HTTP status 422 Unprocessable Entity.
    #[snafu(display("Unprocessable entity: {reason}"))]
    UnprocessableEntity { reason: String },

    /// Critical validation errors prevent the operation.
    ///
    /// Mapped to HTTP status 422 Unprocessable Entity.
    #[snafu(display("Validation failed with critical errors"))]
    CriticalValidationErrors {
        violations: PartialSerializableAggregateViolations,
    },

    // =========================================================================
    // Not Found Errors (HTTP 404)
    // =========================================================================
    /// Entity not found.
    ///
    /// Mapped to HTTP status 404 Not Found.
    #[snafu(display("Entity not found"))]
    NotFound,

    // =========================================================================
    // Client Errors (HTTP 400)
    // =========================================================================
    /// Could not convert to condition (invalid query parameters).
    ///
    /// Mapped to HTTP status 400 Bad Request.
    #[snafu(display("Invalid query parameters: {source}"))]
    IntoCondition { source: IntoAllEqualConditionError },

    // =========================================================================
    // Server Errors (HTTP 500)
    // =========================================================================
    /// Repository/database error.
    ///
    /// Mapped to HTTP status 500 Internal Server Error.
    #[snafu(display("Repository error: {reason:?}"))]
    Repository { reason: Arc<dyn RepositoryError> },

    /// Unexpected lifecycle hook error.
    ///
    /// Mapped to HTTP status 500 Internal Server Error.
    #[snafu(display("Lifecycle hook error: {reason}"))]
    LifecycleHookError { reason: String },

    /// Could not save validation results.
    ///
    /// Mapped to HTTP status 500 Internal Server Error.
    #[snafu(display("Could not save validations: {reason:?}"))]
    SaveValidations { reason: Arc<dyn RepositoryError> },

    /// Could not delete validation results.
    ///
    /// Mapped to HTTP status 500 Internal Server Error.
    #[snafu(display("Could not delete validations: {reason:?}"))]
    DeleteValidations { reason: Arc<dyn RepositoryError> },
}
