//! Access to the surrounding CrudKit instance.

use crate::instance::CrudInstanceContext;
use leptos::prelude::*;

/// Returns the context of the surrounding [`crate::components::instance::CrudInstance`].
///
/// Inside a rendered view, the returned context carries that view's navigation.
///
/// # Panics
///
/// Panics when called outside of a CrudKit instance. Calling it there is a programming error.
#[must_use]
pub fn use_crud_instance() -> CrudInstanceContext {
    use_context::<CrudInstanceContext>()
        .expect("`use_crud_instance` must be called below a mounted `CrudInstance`")
}
