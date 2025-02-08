use crate::error::ErrorInfo;
use thiserror::Error as ThisError;

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
