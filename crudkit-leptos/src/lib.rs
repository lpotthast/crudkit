#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(missing_docs)]
#![recursion_limit = "512"]

//! CrudKit's Leptos frontend.
//!
//! Layering, following Leptonic's hooks branch:
//!
//! - [`hooks`]: state hooks exposing CrudKit's behavior, and interaction hooks binding it to
//!   Leptonic's accessible input and table hooks;
//! - [`atoms`]: headless single-element components where CrudKit adds behavior;
//! - [`components`]: the built-in, composed UI with `crudkit-*` classes, available with the
//!   `components` feature (enabled by default).
//!
//! Next to these, [`config`] holds what applications declare, and [`instance`] the runtime of
//! mounted instances.

pub mod atoms;
#[cfg(feature = "components")]
pub mod components;
pub mod config;
pub mod hooks;
pub mod instance;
#[cfg(all(test, not(target_family = "wasm")))]
mod test_support;

/*
* Reexport common modules.
* This allows the user to only
*
* - `use crudkit_leptos::prelude::*` and
* - derive all common proc macros
*
* without the need to add more use declaration or
* to manually depend on other crud crates such as "crudkit_id",
* which are required for many derive macro implementations.
*/
pub use crudkit_core;
pub use crudkit_web;

/// Common imports of CrudKit applications: the shared contracts, derive macros, and the public
/// types, hooks, atoms, and components of this crate.
pub mod prelude {
    pub use crudkit_core;
    pub use crudkit_core::collaboration;
    pub use crudkit_core::condition;
    pub use crudkit_core::id;
    pub use crudkit_core::id::*;
    pub use crudkit_core::validation;
    pub use crudkit_core::*;
    pub use crudkit_web;
    pub use crudkit_web::prelude::*;

    // Explicitly re-export Model from crudkit_web to resolve ambiguity
    // (both crudkit_core and crudkit_web export Model).
    pub use crudkit_web::model::Model;

    pub use crudkit_core_macros::CkId;
    pub use crudkit_web_macros::{CkActionPayload, CkField, CkResource};

    pub use crudkit_web::http::ReqwestExecutor;
    pub use crudkit_web::view::{CREATE_VIEW, EDIT_VIEW, READ_VIEW, TABLE_VIEW};

    pub use super::atoms::prelude::*;
    #[cfg(feature = "components")]
    pub use super::components::prelude::*;
    pub use super::config::*;
    pub use super::hooks::*;
    pub use super::instance::*;
}
