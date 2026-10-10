//! Default content of atoms.

use leptos::prelude::*;

/// The content of an atom: its `children`, else its `default` content.
pub(crate) fn content_or_default(
    children: Option<Children>,
    default: impl IntoView + 'static,
) -> AnyView {
    match children {
        Some(children) => children().into_any(),
        None => default.into_any(),
    }
}
