use reqwest::Method;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use std::sync::Arc;

use super::error::{RequestError, error_response_to_request_error};
use super::executor::ReqwestExecutor;

/// Posts `body` as JSON to `url` and deserializes the JSON response.
///
/// # Errors
///
/// Returns `RequestError::InvalidRequest` if `url` cannot be parsed, `RequestError::Request` if the
/// HTTP request fails, the status-specific `RequestError` variant (e.g. `Forbidden`, `NotFound`, or
/// `UnprocessableEntity`) if the server responds with a non-success status, or
/// `RequestError::Deserialize` if the response body cannot be deserialized.
pub(crate) async fn post<B, T>(
    url: String,
    executor: &(impl ReqwestExecutor + ?Sized),
    body: B,
) -> Result<T, RequestError>
where
    T: DeserializeOwned + Debug,
    B: Serialize + Debug + Send + Sync + 'static,
{
    let parsed_url = reqwest::Url::parse(&url)
        .map_err(|e| RequestError::InvalidRequest(format!("Invalid URL '{url}': {e}")))?;

    let result = executor
        .request(
            Method::POST,
            parsed_url,
            Arc::new(move |builder| {
                builder
                    .header("Content-Type", "application/json")
                    .json(&body)
            }),
        )
        .await;

    match result {
        Ok(response) if response.status().is_success() => response
            .json::<T>()
            .await
            .map_err(|err| RequestError::Deserialize(err.to_string())),
        Ok(response) => Err(error_response_to_request_error(response).await),
        Err(err) => {
            tracing::error!(?err, "Request failed");
            Err(RequestError::Request(err.to_string()))
        }
    }
}
