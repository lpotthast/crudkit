use async_trait::async_trait;
use leptos_keycloak_auth::reqwest::Method;
use send_wrapper::SendWrapper;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error as ThisError;

use crate::error::ErrorInfo;

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

#[async_trait]
pub trait ReqwestExecutor: Debug + Send + Sync {
    async fn request(
        &self,
        method: Method,
        url: reqwest::Url,
        with: Arc<dyn Fn(reqwest::RequestBuilder) -> reqwest::RequestBuilder + Send + Sync>,
    ) -> Result<reqwest::Response, reqwest::Error>;
}

#[async_trait]
impl ReqwestExecutor for leptos_keycloak_auth::AuthenticatedClient {
    async fn request(
        &self,
        method: Method,
        url: reqwest::Url,
        with: Arc<dyn Fn(reqwest::RequestBuilder) -> reqwest::RequestBuilder + Send + Sync>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        SendWrapper::new(self.request(method, url, |builder| with(builder))).await
    }
}

#[async_trait]
impl ReqwestExecutor for reqwest::Client {
    async fn request(
        &self,
        method: Method,
        url: reqwest::Url,
        with: Arc<dyn Fn(reqwest::RequestBuilder) -> reqwest::RequestBuilder + Send + Sync>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        SendWrapper::new(with(self.request(method, url)).send()).await
    }
}

#[derive(Debug)]
pub struct NewClientPerRequestExecutor;

#[async_trait]
impl ReqwestExecutor for NewClientPerRequestExecutor {
    async fn request(
        &self,
        method: Method,
        url: reqwest::Url,
        with: Arc<dyn Fn(reqwest::RequestBuilder) -> reqwest::RequestBuilder + Send + Sync>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        ReqwestExecutor::request(&reqwest::Client::new(), method, url, with).await
    }
}

/// build all kinds of http requests: post/get/delete etc.
pub async fn request<B, T>(
    method: Method,
    url: String,
    executor: &(impl ReqwestExecutor + ?Sized),
    body: B,
) -> Result<T, RequestError>
where
    T: DeserializeOwned + Debug,
    B: Serialize + Debug + Send + Sync + 'static,
{
    // ASSUMPTION: The given url is complete, meaning nothing hast to be added to it to work!
    let allow_body = method == Method::POST || method == Method::PUT;

    let result = executor
        .request(
            method,
            reqwest::Url::parse(&url).unwrap(),
            Arc::new(move |builder| {
                if allow_body {
                    builder
                        .header("Content-Type", "application/json")
                        .json(&body)
                } else {
                    builder
                }
            }),
        )
        .await;

    process_json_response(result).await
}

async fn process_json_response<T>(
    result: Result<reqwest::Response, reqwest::Error>,
) -> Result<T, RequestError>
where
    T: DeserializeOwned + Debug,
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

#[allow(dead_code)]
/// Delete request
pub async fn request_delete<T>(
    url: String,
    executor: &impl ReqwestExecutor,
) -> Result<T, RequestError>
where
    T: DeserializeOwned + Debug,
{
    request(Method::DELETE, url, executor, ()).await
}

/// Get request
#[allow(dead_code)]
pub async fn request_get<T>(url: String, executor: &impl ReqwestExecutor) -> Result<T, RequestError>
where
    T: DeserializeOwned + Debug,
{
    request(Method::GET, url, executor, ()).await
}

/// Post request with a body
#[allow(dead_code)]
pub async fn request_post<B, T>(
    url: String,
    executor: &(impl ReqwestExecutor + ?Sized),
    body: B,
) -> Result<T, RequestError>
where
    T: DeserializeOwned + Debug,
    B: Serialize + Debug + Send + Sync + 'static,
{
    request(Method::POST, url, executor, body).await
}

///// Post request with a body
//#[allow(dead_code)]
//pub async fn request_post_multipart<T>(
//    url: String,
//    executor: &impl ReqwestExecutor,
//    form: reqwest::multipart::Form,
//) -> Result<T, RequestError>
//where
//    T: DeserializeOwned + Debug,
//{
//    let result = executor
//        .request(Method::POST, url, move |builder| {
//            builder
//                //    //.header("Content-Type", "application/json");
//                .multipart(form)
//        })
//        .await;
//
//    process_json_response(result).await
//}

/// Put request with a body
#[allow(dead_code)]
pub async fn request_put<B, T>(
    url: String,
    auth: &impl ReqwestExecutor,
    body: B,
) -> Result<T, RequestError>
where
    T: DeserializeOwned + Debug,
    B: Serialize + Debug + Send + Sync + 'static,
{
    request(Method::PUT, url, auth, body).await
}
