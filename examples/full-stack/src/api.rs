//! How the frontend reaches the CrudKit REST API.

use crudkit_leptos::prelude::ReqwestExecutor;
use std::sync::Arc;

/// Absolute base URL of the CrudKit API, e.g. `http://127.0.0.1:3000/api`.
///
/// `reqwest` requires absolute URLs. In the browser, the page origin is used, so the app works under any host name.
/// CrudKit only fetches data in the browser, but the configuration is also built while server-side rendering.
pub fn api_base_url() -> String {
    #[cfg(feature = "ssr")]
    {
        let site_addr =
            std::env::var("LEPTOS_SITE_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_owned());
        format!("http://{site_addr}/api")
    }
    #[cfg(not(feature = "ssr"))]
    {
        let origin = leptos::prelude::window()
            .location()
            .origin()
            .unwrap_or_else(|_| "http://127.0.0.1:3000".to_owned());
        format!("{origin}/api")
    }
}

/// Unauthenticated HTTP client used by every CrudKit instance.
pub fn executor() -> Arc<dyn ReqwestExecutor> {
    Arc::new(reqwest::Client::new())
}
