use crate::error::ErrorInfo;
use thiserror::Error as ThisError;

/// Type-safe error for CRUD operations.
///
/// This error type provides a unified way to handle all CRUD operation failures
/// in the frontend. Use this type in failure callbacks to handle errors appropriately.
///
/// # HTTP Status Mapping
///
/// | Error Variant | HTTP Status |
/// |--------------|-------------|
/// | `Forbidden` | 403 Forbidden |
/// | `UnprocessableEntity` | 422 Unprocessable Entity |
/// | `NotFound` | 404 Not Found |
/// | `ServerError` | 500 Internal Server Error |
/// | `Unauthorized` | 401 Unauthorized |
/// | `BadRequest` | 400 Bad Request |
/// | `NetworkError` | N/A (client-side error) |
///
/// # Example
///
/// ```ignore
/// on_entity_creation_failed=move |error| {
///     match error {
///         CrudOperationError::Forbidden { reason } => {
///             show_toast(ToastVariant::Error, "Permission Denied", reason)
///         }
///         CrudOperationError::UnprocessableEntity { reason } => {
///             show_toast(ToastVariant::Warn, "Cannot Proceed", reason)
///         }
///         CrudOperationError::ServerError { message } => {
///             show_toast(ToastVariant::Error, "Server Error", message)
///         }
///         // ...
///     }
/// }
/// ```
#[derive(ThisError, Clone, Debug, PartialEq)]
pub enum CrudOperationError {
    /// Permission denied (HTTP 403).
    ///
    /// The user lacks permission to perform this operation.
    #[error("Permission denied: {reason}")]
    Forbidden { reason: String },

    /// Business logic/validation rejection (HTTP 422).
    ///
    /// The operation was rejected due to business rules or validation errors.
    #[error("Unprocessable entity: {reason}")]
    UnprocessableEntity { reason: String },

    /// Entity not found (HTTP 404).
    ///
    /// The requested entity does not exist.
    #[error("Not found: {message}")]
    NotFound { message: String },

    /// Server error (HTTP 500).
    ///
    /// An internal server error occurred.
    #[error("Server error: {message}")]
    ServerError { message: String },

    /// Authentication required (HTTP 401).
    ///
    /// The user needs to authenticate to perform this operation.
    #[error("Unauthorized: {message}")]
    Unauthorized { message: String },

    /// Bad request (HTTP 400).
    ///
    /// The request was malformed or invalid.
    #[error("Bad request: {message}")]
    BadRequest { message: String },

    /// Network/deserialization error.
    ///
    /// A client-side error occurred (network failure, deserialization error).
    #[error("Network error: {message}")]
    NetworkError { message: String },
}

// TODO: This mapping only works because we know that we use the inverse mapping in crudkit-rs. Should we use a tagged enum (serialization) instead to make this unambiguous? We could even share the type between frontend and backend impls.
impl From<RequestError> for CrudOperationError {
    fn from(err: RequestError) -> Self {
        match err {
            RequestError::Forbidden(s) => Self::Forbidden { reason: s },
            RequestError::UnprocessableEntity(info) => Self::UnprocessableEntity {
                reason: format!("{:?}", info.errors),
            },
            RequestError::NotFound(s) => Self::NotFound { message: s },
            RequestError::InternalServerError(s) => Self::ServerError { message: s },
            RequestError::Unauthorized(s) => Self::Unauthorized { message: s },
            RequestError::BadRequest(s) => Self::BadRequest { message: s },
            RequestError::Request(s) | RequestError::Deserialize(s) => {
                Self::NetworkError { message: s }
            }
        }
    }
}

// TODO: All our other libraries use `snafu` for error handling. We should probably switch to that and remove the thiserror dependency.
#[derive(ThisError, Clone, Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum RequestError {
    /// 400
    #[error("BadRequest: {0}")]
    BadRequest(String),

    /// 401
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// 403
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// 404
    #[error("Not Found: {0}")]
    NotFound(String),

    /// 422
    #[error("Unprocessable Entity: {0:?}")]
    UnprocessableEntity(ErrorInfo),

    /// 500
    #[error("InternalServerError: {0}")]
    InternalServerError(String),

    /// serde deserialize error
    #[error("DeserializeError: {0}")]
    Deserialize(String),

    /// request error
    #[error("HttpRequestError: {0}")]
    Request(String),
}

pub async fn error_response_to_request_error(response: reqwest::Response) -> RequestError {
    let status = response.status().as_u16();
    assert!(status != 200);
    match status {
        400 => RequestError::BadRequest(
            response
                .text()
                .await
                .unwrap_or_else(|error| error.to_string()),
        ),
        401 => RequestError::Unauthorized(
            response
                .text()
                .await
                .unwrap_or_else(|error| error.to_string()),
        ),
        403 => RequestError::Forbidden(
            response
                .text()
                .await
                .unwrap_or_else(|error| error.to_string()),
        ),
        404 => RequestError::NotFound(
            response
                .text()
                .await
                .unwrap_or_else(|error| error.to_string()),
        ),
        500 => RequestError::InternalServerError(
            response
                .text()
                .await
                .unwrap_or_else(|error| error.to_string()),
        ),
        422 => {
            let data = response.json::<ErrorInfo>().await;
            match data {
                Ok(error_info) => RequestError::UnprocessableEntity(error_info),
                Err(err) => RequestError::Deserialize(err.to_string()),
            }
        }
        code => RequestError::Request(format!(
            "Code: {}, Text: {}",
            code,
            response
                .text()
                .await
                .unwrap_or_else(|error| error.to_string())
        )),
    }
}
