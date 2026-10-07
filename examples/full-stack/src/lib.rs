#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![recursion_limit = "512"]

pub mod api;
pub mod app;
pub mod marauder;
pub mod layout;
pub mod models;
pub mod pages;
pub mod resources;

#[cfg(feature = "ssr")]
pub mod server;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default_with_config(
        tracing_wasm::WASMLayerConfigBuilder::default()
            .set_max_level(tracing::Level::DEBUG)
            .build(),
    );
    leptos::mount::hydrate_body(app::App);
}
