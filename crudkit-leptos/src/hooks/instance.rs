//! Access to the surrounding CrudKit instance.

use crate::instance::{CrudInstanceContext, CrudNavigation};
use leptos::prelude::*;

/// Returns the context of the surrounding [`CrudInstance`](crate::instance::CrudInstance).
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

/// Returns the navigation of the surrounding view, or of the instance outside of a rendered view.
///
/// Navigating with it opens another view of the instance, e.g. `navigate(CrudView::create())`, and
/// asks for confirmation while a form of the view has unsaved changes.
///
/// # Panics
///
/// Panics when called outside of a CrudKit instance.
pub fn use_crud_navigation() -> CrudNavigation {
    use_crud_instance().navigation
}
