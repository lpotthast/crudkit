//! Application-wide view registry.

use crate::config::CrudViewRegistry;
use leptos::prelude::*;

/// Makes `registry` the view registry of all instances below whose configuration sets none, e.g.
/// to render every instance of an application with its own views.
pub fn provide_crud_view_registry(registry: CrudViewRegistry) {
    provide_context(registry);
}
