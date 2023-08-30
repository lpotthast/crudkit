use crudkit_web::Label;
use leptos::*;

// TODO: Extract into leptonic
#[component]
pub fn CrudFieldLabel(cx: Scope, label: Label) -> impl IntoView {
    view! {cx,
        <span class="crud-field-label">
            {label.name.clone()}
        </span>
    }
}

// TODO: Extract into leptonic
#[component]
pub fn CrudFieldLabelOpt(cx: Scope, label: Option<Label>) -> impl IntoView {
    match label {
        Some(label) => view! {cx, <CrudFieldLabel label=label/>}.into_view(cx),
        None => ().into_view(cx),
    }
}
