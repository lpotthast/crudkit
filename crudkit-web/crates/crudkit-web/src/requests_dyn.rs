use crate::crud_rest_data_provider_dyn::serialize_any_as_json;
use crate::request_error::{error_response_to_request_error, RequestError};
use crate::reqwest_executor::ReqwestExecutor;
use leptos_keycloak_auth::reqwest::Method;
use std::fmt::Debug;
use std::sync::Arc;

/// build all kinds of http requests: post/get/delete etc.
pub async fn request<B>(
    method: Method,
    url: String,
    executor: &(impl ReqwestExecutor + ?Sized),
    body: B,
) -> Result<serde_json::Value, RequestError>
where
    B: erased_serde::Serialize + Debug + Send + Sync + 'static,
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
                        .body(serialize_any_as_json(&body))
                } else {
                    builder
                }
            }),
        )
        .await;

    process_json_response(result).await
}

async fn process_json_response(
    result: Result<reqwest::Response, reqwest::Error>,
) -> Result<serde_json::Value, RequestError> {
    match result {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
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

#[allow(dead_code)]
/// Delete request
pub async fn request_delete(
    url: String,
    executor: &impl ReqwestExecutor,
) -> Result<serde_json::Value, RequestError> {
    request(Method::DELETE, url, executor, ()).await
}

/// Get request
#[allow(dead_code)]
pub async fn request_get(
    url: String,
    executor: &impl ReqwestExecutor,
) -> Result<serde_json::Value, RequestError> {
    request(Method::GET, url, executor, ()).await
}

/// Post request with a body
#[allow(dead_code)]
pub async fn request_post<B>(
    url: String,
    executor: &(impl ReqwestExecutor + ?Sized),
    body: B,
) -> Result<serde_json::Value, RequestError>
where
    B: erased_serde::Serialize + Debug + Send + Sync + 'static,
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
pub async fn request_put<B>(
    url: String,
    auth: &impl ReqwestExecutor,
    body: B,
) -> Result<serde_json::Value, RequestError>
where
    B: erased_serde::Serialize + Debug + Send + Sync + 'static,
{
    request(Method::PUT, url, auth, body).await
}
