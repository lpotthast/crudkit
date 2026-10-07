#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use full_stack::app::{App, shell};
    use full_stack::server;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use std::sync::Arc;
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,sea_orm=warn")),
        )
        .init();

    let db = match server::db::connect_and_prepare().await {
        Ok(db) => Arc::new(db),
        Err(err) => {
            tracing::error!(?err, "Could not prepare the database.");
            std::process::exit(1);
        }
    };

    let conf = match get_configuration(None) {
        Ok(conf) => conf,
        Err(err) => {
            tracing::error!(?err, "Could not read the Leptos configuration.");
            std::process::exit(1);
        }
    };
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options)
        .merge(server::routes::api_router(db));

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(err) => {
            tracing::error!(?err, %addr, "Could not bind the listener.");
            std::process::exit(1);
        }
    };
    tracing::info!("Listening on http://{addr}");
    if let Err(err) = axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
    {
        tracing::error!(?err, "Server error.");
    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // The client-side entry point is `full_stack::hydrate`.
}
