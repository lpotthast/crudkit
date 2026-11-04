//! The live region shared by the status atoms.

use crate::config::CrudText;
use crate::hooks::texts::CrudTexts;
use crudkit_web::http::RequestError;
use leptonic::leptos_classes::Classes;
use leptonic::leptos_styles::Styles;
use leptos::prelude::*;

/// A `<div>` that is a polite live region (`role="status"`) whose content explains a status.
///
/// The region stays rendered, empty while there is nothing to explain, because screen readers
/// announce changes of a live region's content, but rarely the content of a newly inserted one.
#[component]
pub(crate) fn StatusRegion(
    /// The `data-status` value.
    data_status: Signal<&'static str>,
    classes: Classes,
    styles: Styles,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <div class=classes style=styles role="status" data-status=data_status>
            {children()}
        </div>
    }
}

/// The default content for a status, or `content` if given.
pub(crate) fn status_content(content: Option<&ViewFn>, text: String) -> AnyView {
    match content {
        Some(content) => content.run(),
        None => text.into_any(),
    }
}

/// The default content for the status of a list or an entity that could not be loaded because of
/// `error` ([`CrudText::StatusFailed`]).
pub(crate) fn failed_status_text(texts: CrudTexts, error: &RequestError) -> String {
    let error = texts.request_error(error);
    texts.get(CrudText::StatusFailed { error: &error })
}
