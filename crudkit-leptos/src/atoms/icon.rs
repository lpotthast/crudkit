//! Decorative icons.

use leptonic::utils::classes::{Classes, MergeStrategy};
use leptonic::utils::styles::Styles;
use leptos::prelude::*;

/// Renders an `icondata` icon as an inline, decorative SVG.
///
/// The icon is hidden from assistive technology. Give the surrounding control an accessible name.
///
/// Default class: `crudkit-Icon`.
#[component]
pub fn CrudIcon(
    /// The icon to render.
    icon: icondata_core::Icon,
    #[prop(into, optional)] classes: Classes,
    /// Styles of the `<svg>` element, applied after the icon's own.
    #[prop(into, optional)]
    styles: Styles,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-Icon")
        .merge(classes, MergeStrategy::UnionConditions);
    // Icon sets style some icons inline; the caller's styles come after them.
    let icon_styles = icon
        .style
        .and_then(|css| Styles::parse_css(css).ok())
        .unwrap_or_default();
    view! {
        <svg
            class=classes
            aria-hidden="true"
            focusable="false"
            width="1em"
            height="1em"
            viewBox=icon.view_box
            style=icon_styles.merge(styles)
            fill=icon.fill.unwrap_or("currentColor")
            stroke=icon.stroke
            stroke-width=icon.stroke_width
            stroke-linecap=icon.stroke_linecap
            stroke-linejoin=icon.stroke_linejoin
            inner_html=icon.data
        ></svg>
    }
}
