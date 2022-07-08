pub mod files;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

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

/// Conduit api error info for Unprocessable Entity error
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ErrorInfo {
    pub errors: HashMap<String, Vec<String>>,
}
