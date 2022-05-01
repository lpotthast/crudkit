use gloo::storage::{LocalStorage, Storage};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use serde::{de::DeserializeOwned, Serialize};

use crate::types::{ErrorInfo, RequestError};

// TODO: Make this read an environment variable at runtime.
//const API_ROOT: &str = dotenv!("API_ROOT");
const API_ROOT: &str = "http://127.0.0.1:4000/api";
const TOKEN_KEY: &str = "yew.token";

lazy_static! {
    /// Jwt token read from local storage.
    pub static ref TOKEN: RwLock<Option<String>> = {
        if let Ok(token) = LocalStorage::get(TOKEN_KEY) {
            RwLock::new(Some(token))
        } else {
            RwLock::new(None)
        }
    };
}

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

/// build all kinds of http request: post/get/delete etc.
pub async fn request<S, B, T>(method: reqwest::Method, url: S, body: B) -> Result<T, RequestError>
where
    S: AsRef<str>,
    T: DeserializeOwned + std::fmt::Debug,
    B: Serialize + std::fmt::Debug,
{
    let allow_body = method == reqwest::Method::POST || method == reqwest::Method::PUT;
    let url = format!("{}{}", API_ROOT, url.as_ref());
    let mut builder = reqwest::Client::new()
        .request(method, url)
        .header("Content-Type", "application/json");
    if let Some(token) = get_token() {
        builder = builder.bearer_auth(token);
    }

    if allow_body {
        builder = builder.json(&body);
    }

    let response = builder.send().await;

    if let Ok(data) = response {
        if data.status().is_success() {
            let data: Result<T, _> = data.json::<T>().await;
            if let Ok(data) = data {
                //log::debug!("Response: {:?}", data);
                Ok(data)
            } else {
                Err(RequestError::Deserialize)
            }
        } else {
            match data.status().as_u16() {
                401 => Err(RequestError::Unauthorized),
                403 => Err(RequestError::Forbidden),
                404 => Err(RequestError::NotFound),
                500 => Err(RequestError::InternalServerError),
                422 => {
                    let data: Result<ErrorInfo, _> = data.json::<ErrorInfo>().await;
                    if let Ok(data) = data {
                        Err(RequestError::UnprocessableEntity(data))
                    } else {
                        Err(RequestError::Deserialize)
                    }
                }
                _ => Err(RequestError::Request),
            }
        }
    } else {
        Err(RequestError::Request)
    }
}

#[allow(dead_code)]
/// Delete request
pub async fn request_delete<S, T>(url: S) -> Result<T, RequestError>
where
    S: AsRef<str>,
    T: DeserializeOwned + std::fmt::Debug,
{
    request(reqwest::Method::DELETE, url, ()).await
}

/// Get request
#[allow(dead_code)]
pub async fn request_get<S, T>(url: S) -> Result<T, RequestError>
where
    S: AsRef<str>,
    T: DeserializeOwned + std::fmt::Debug,
{
    request(reqwest::Method::GET, url, ()).await
}

/// Post request with a body
#[allow(dead_code)]
pub async fn request_post<S, B, T>(url: S, body: B) -> Result<T, RequestError>
where
    S: AsRef<str>,
    T: DeserializeOwned + std::fmt::Debug,
    B: Serialize + std::fmt::Debug,
{
    request(reqwest::Method::POST, url, body).await
}

/// Put request with a body
#[allow(dead_code)]
pub async fn request_put<S, B, T>(url: S, body: B) -> Result<T, RequestError>
where
    S: AsRef<str>,
    T: DeserializeOwned + std::fmt::Debug,
    B: Serialize + std::fmt::Debug,
{
    request(reqwest::Method::PUT, url, body).await
}
