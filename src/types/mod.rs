pub mod files;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

#[derive(ThisError, Clone, Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum RequestError {
    /// 400
    #[error("BadRequest")]
    BadRequest(String),

    /// 401
    #[error("Unauthorized")]
    Unauthorized(String),

    /// 403
    #[error("Forbidden")]
    Forbidden(String),

    /// 404
    #[error("Not Found")]
    NotFound(String),

    /// 422
    #[error("Unprocessable Entity: {0:?}")]
    UnprocessableEntity(ErrorInfo),

    /// 500
    #[error("Internal Server Error")]
    InternalServerError(String),

    /// serde deserialize error
    #[error("Deserialize Error")]
    Deserialize(String),

    /// request error
    #[error("Http Request Error")]
    Request(String),
}

/// Conduit api error info for Unprocessable Entity error
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ErrorInfo {
    pub errors: HashMap<String, Vec<String>>,
}
