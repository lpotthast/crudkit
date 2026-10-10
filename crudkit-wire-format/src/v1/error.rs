//! The error body of every failed request.

use super::{PartialViolationsV1, WireFormatVersionV1};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response body of a failed request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponseV1 {
    /// Wire format version of the body.
    pub wire_format_version: WireFormatVersionV1,
    pub error: ErrorV1,
}

/// Why a request failed. Serialized with a `kind` tag, so clients classify an error without
/// relying on the transport's status code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ErrorV1 {
    /// The request is malformed, e.g. it names an unknown field or uses an unsupported condition.
    BadRequest { message: String },
    /// The operation requires authentication.
    Unauthorized { message: String },
    /// The caller may not perform the operation.
    Forbidden { message: String },
    /// The selected entity does not exist.
    NotFound { message: String },
    /// Business rules rejected the operation.
    UnprocessableEntity { message: String },
    /// Critical validation errors prevent the operation.
    CriticalValidationErrors {
        message: String,
        violations: PartialViolationsV1,
    },
    /// The server failed to perform the operation.
    InternalServerError { message: String },
}
