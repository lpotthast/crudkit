use leptos_keycloak_auth::reqwest::Method;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use std::sync::Arc;

use crate::request_error::{RequestError, error_response_to_request_error};
use crate::reqwest_executor::ReqwestExecutor;

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
        Err(err) => {
            tracing::error!(?err, "Request failed");
            Err(RequestError::Request(err.to_string()))
        }
    }
}

#[allow(dead_code)]
/// Delete request
pub async fn delete<T>(url: String, executor: &impl ReqwestExecutor) -> Result<T, RequestError>
where
    T: DeserializeOwned + Debug,
{
    request::<(), T>(Method::DELETE, url, executor, ()).await
}

/// Get request
#[allow(dead_code)]
pub async fn get<T>(url: String, executor: &impl ReqwestExecutor) -> Result<T, RequestError>
where
    T: DeserializeOwned + Debug,
{
    request::<(), T>(Method::GET, url, executor, ()).await
}

/// Post request with a body
#[allow(dead_code)]
pub async fn post<B, T>(
    url: String,
    executor: &(impl ReqwestExecutor + ?Sized),
    body: B,
) -> Result<T, RequestError>
where
    T: DeserializeOwned + Debug,
    B: Serialize + Debug + Send + Sync + 'static,
{
    request::<B, T>(Method::POST, url, executor, body).await
}

/// Post request returning raw JSON (for dynamic/type-erased usage)
#[allow(dead_code)]
pub async fn post_json<B>(
    url: String,
    executor: &(impl ReqwestExecutor + ?Sized),
    body: B,
) -> Result<serde_json::Value, RequestError>
where
    B: Serialize + Debug + Send + Sync + 'static,
{
    request::<B, serde_json::Value>(Method::POST, url, executor, body).await
}

/// Put request with a body
#[allow(dead_code)]
pub async fn put<B, T>(url: String, auth: &impl ReqwestExecutor, body: B) -> Result<T, RequestError>
where
    T: DeserializeOwned + Debug,
    B: Serialize + Debug + Send + Sync + 'static,
{
    request::<B, T>(Method::PUT, url, auth, body).await
}
