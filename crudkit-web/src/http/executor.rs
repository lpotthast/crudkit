use async_trait::async_trait;
use reqwest::Method;
use send_wrapper::SendWrapper;
use std::fmt::Debug;
use std::sync::Arc;

// `async_trait` marks the boxed futures it returns `#[must_use]`, which they already are.
#[allow(clippy::double_must_use)]
#[async_trait]
pub trait ReqwestExecutor: Debug + Send + Sync {
    async fn request(
        &self,
        method: Method,
        url: reqwest::Url,
        with: Arc<dyn Fn(reqwest::RequestBuilder) -> reqwest::RequestBuilder + Send + Sync>,
    ) -> Result<reqwest::Response, reqwest::Error>;
}

#[cfg(feature = "keycloak-auth")]
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
