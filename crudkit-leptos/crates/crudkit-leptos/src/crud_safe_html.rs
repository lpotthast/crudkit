use std::borrow::Cow;

use leptos::{html::span, *};

#[component]
pub fn CrudSafeHtml(cx: Scope, #[prop(into)] html: Cow<'static, str>) -> impl IntoView {
    // TODO: Sanitize input?
    span(cx).inner_html(html)
}
