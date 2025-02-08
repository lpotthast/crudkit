use async_trait::async_trait;
use leptos_keycloak_auth::reqwest::Method;
use send_wrapper::SendWrapper;
use std::fmt::Debug;
use std::sync::Arc;

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
