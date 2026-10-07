//! Decorative icons.

use leptonic::utils::classes::Classes;
use leptos::prelude::*;

/// Renders an `icondata` icon as an inline, decorative SVG.
///
/// The icon is hidden from assistive technology. Give the surrounding control an accessible name.
#[component]
pub fn CrudIcon(
    /// The icon to render.
    icon: icondata::Icon,
    /// Classes of the `<svg>` element.
    #[prop(into, optional)]
    classes: Classes,
) -> impl IntoView {
    view! {
        <svg
            class=classes
            aria-hidden="true"
            focusable="false"
            width="1em"
            height="1em"
            viewBox=icon.view_box
            style=icon.style
            fill=icon.fill.unwrap_or("currentColor")
            stroke=icon.stroke
            stroke-width=icon.stroke_width
            stroke-linecap=icon.stroke_linecap
            stroke-linejoin=icon.stroke_linejoin
            inner_html=icon.data
        ></svg>
    }
}
