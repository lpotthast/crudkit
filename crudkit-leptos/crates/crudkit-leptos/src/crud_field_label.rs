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
