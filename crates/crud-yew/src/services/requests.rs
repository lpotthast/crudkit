use gloo::storage::{LocalStorage, Storage};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{de::DeserializeOwned, Serialize};

use crate::types::{ErrorInfo, RequestError};

const TOKEN_KEY: &str = "yew.token";

/// Jwt token read from local storage.
pub static TOKEN: Lazy<RwLock<Option<String>>> = Lazy::new(|| {
    if let Ok(token) = LocalStorage::get(TOKEN_KEY) {
        RwLock::new(Some(token))
    } else {
        RwLock::new(None)
    }
});

/// Set jwt token to local storage.
pub fn set_token(token: Option<String>) {
    if let Some(t) = token.clone() {
        LocalStorage::set(TOKEN_KEY, t).expect("failed to set");
    } else {
        LocalStorage::delete(TOKEN_KEY);
    }
    let mut token_lock = TOKEN.write();
    *token_lock = token;
}

/// Get jwt token from lazy static.
pub fn get_token() -> Option<String> {
    let token_lock = TOKEN.read();
    token_lock.clone()
}

/// build all kinds of http requests: post/get/delete etc.
pub async fn request<B, T>(method: reqwest::Method, url: String, body: B) -> Result<T, RequestError>
where
    T: DeserializeOwned + std::fmt::Debug,
    B: Serialize + std::fmt::Debug,
{
    // ASSUMPTION: The given url is complete, meaning nothing hast to be added to it to work!
    let allow_body = method == reqwest::Method::POST || method == reqwest::Method::PUT;

    let mut builder = request_builder(method, url);
    builder = builder.header("Content-Type", "application/json");

    if allow_body {
        builder = builder.json(&body);
    }

    let result = builder.send().await;

    process_json_response(result).await
}

fn request_builder<U>(method: reqwest::Method, url: U) -> reqwest::RequestBuilder
where
    U: reqwest::IntoUrl,
{
    let mut builder = reqwest::Client::new().request(method, url);
    if let Some(token) = get_token() {
        builder = builder.bearer_auth(token);
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
pub async fn request_delete<T>(url: String) -> Result<T, RequestError>
where
    T: DeserializeOwned + std::fmt::Debug,
{
    request(reqwest::Method::DELETE, url, ()).await
}

/// Get request
#[allow(dead_code)]
pub async fn request_get<T>(url: String) -> Result<T, RequestError>
where
    T: DeserializeOwned + std::fmt::Debug,
{
    request(reqwest::Method::GET, url, ()).await
}

/// Post request with a body
#[allow(dead_code)]
pub async fn request_post<B, T>(url: String, body: B) -> Result<T, RequestError>
where
    T: DeserializeOwned + std::fmt::Debug,
    B: Serialize + std::fmt::Debug,
{
    request(reqwest::Method::POST, url, body).await
}

/// Post request with a body
#[allow(dead_code)]
pub async fn request_post_multipart<T>(
    url: String,
    form: reqwest::multipart::Form,
) -> Result<T, RequestError>
where
    T: DeserializeOwned + std::fmt::Debug,
{
    let builder = request_builder(reqwest::Method::POST, url)
        //.header("Content-Type", "application/json");
        .multipart(form);

    let result = builder.send().await;

    process_json_response(result).await
}

/// Put request with a body
#[allow(dead_code)]
pub async fn request_put<B, T>(url: String, body: B) -> Result<T, RequestError>
where
    T: DeserializeOwned + std::fmt::Debug,
    B: Serialize + std::fmt::Debug,
{
    request(reqwest::Method::PUT, url, body).await
}
