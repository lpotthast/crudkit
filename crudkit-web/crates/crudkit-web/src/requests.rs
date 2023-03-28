use std::{fmt::{Debug, Display}, convert::Infallible};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error as ThisError;

use crate::error::ErrorInfo;

#[derive(ThisError, Clone, Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum RequestError {
    /// Auth provider
    #[error("Auth data could not be provided: {0}")]
    AuthProvider(String),

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

#[derive(Debug, Clone, PartialEq)]
pub enum AuthMethod {
    Bearer { token: String },
}

pub trait AuthProvider {
    type Error: Debug + Display;
    fn provide() -> Result<Option<AuthMethod>, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoAuthProvider {}

impl AuthProvider for NoAuthProvider {
    type Error = Infallible;

    fn provide() -> Result<Option<AuthMethod>, Self::Error> {
        Ok(None)
    }
}

/// build all kinds of http requests: post/get/delete etc.
pub async fn request<B, T>(
    method: reqwest::Method,
    url: String,
    auth: Option<AuthMethod>,
    body: B,
) -> Result<T, RequestError>
where
    T: DeserializeOwned + std::fmt::Debug,
    B: Serialize + std::fmt::Debug,
{
    // ASSUMPTION: The given url is complete, meaning nothing hast to be added to it to work!
    let allow_body = method == reqwest::Method::POST || method == reqwest::Method::PUT;

    let mut builder = request_builder(method, url, auth);
    builder = builder.header("Content-Type", "application/json");

    if allow_body {
        builder = builder.json(&body);
    }

    let result = builder.send().await;

    process_json_response(result).await
}

fn request_builder<U>(
    method: reqwest::Method,
    url: U,
    auth: Option<AuthMethod>,
) -> reqwest::RequestBuilder
where
    U: reqwest::IntoUrl,
{
    let mut builder = reqwest::Client::new().request(method, url);
    if let Some(auth) = auth {
        match auth {
            AuthMethod::Bearer { token } => {
                builder = builder.bearer_auth(token);
            }
        }
    }
    builder
}

async fn process_json_response<T>(
    result: Result<reqwest::Response, reqwest::Error>,
) -> Result<T, RequestError>
where
    T: DeserializeOwned + std::fmt::Debug,
{
    match result {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<T>().await {
                    Ok(data) => Ok(data),
                    Err(err) => Err(RequestError::Deserialize(err.to_string())),
                }
            } else {
                Err(error_response_to_request_error(response).await)
            }
        }
        Err(err) => Err(RequestError::Request(err.to_string())),
    }
}

async fn error_response_to_request_error(response: reqwest::Response) -> RequestError {
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

#[allow(dead_code)]
/// Delete request
pub async fn request_delete<T>(url: String, auth: Option<AuthMethod>) -> Result<T, RequestError>
where
    T: DeserializeOwned + std::fmt::Debug,
{
    request(reqwest::Method::DELETE, url, auth, ()).await
}

/// Get request
#[allow(dead_code)]
pub async fn request_get<T>(url: String, auth: Option<AuthMethod>) -> Result<T, RequestError>
where
    T: DeserializeOwned + std::fmt::Debug,
{
    request(reqwest::Method::GET, url, auth, ()).await
}

/// Post request with a body
#[allow(dead_code)]
pub async fn request_post<B, T>(
    url: String,
    auth: Option<AuthMethod>,
    body: B,
) -> Result<T, RequestError>
where
    T: DeserializeOwned + std::fmt::Debug,
    B: Serialize + std::fmt::Debug,
{
    request(reqwest::Method::POST, url, auth, body).await
}

/// Post request with a body
#[allow(dead_code)]
pub async fn request_post_multipart<T>(
    url: String,
    auth: Option<AuthMethod>,
    form: reqwest::multipart::Form,
) -> Result<T, RequestError>
where
    T: DeserializeOwned + std::fmt::Debug,
{
    let builder = request_builder(reqwest::Method::POST, url, auth)
        //.header("Content-Type", "application/json");
        .multipart(form);

    let result = builder.send().await;

    process_json_response(result).await
}

/// Put request with a body
#[allow(dead_code)]
pub async fn request_put<B, T>(
    url: String,
    auth: Option<AuthMethod>,
    body: B,
) -> Result<T, RequestError>
where
    T: DeserializeOwned + std::fmt::Debug,
    B: Serialize + std::fmt::Debug,
{
    request(reqwest::Method::PUT, url, auth, body).await
}
