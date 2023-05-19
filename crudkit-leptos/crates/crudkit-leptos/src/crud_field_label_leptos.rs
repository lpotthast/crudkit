use crudkit_web::Label;
use leptos::*;

#[component]
pub fn CrudFieldLabelL(cx: Scope, label: Label) -> impl IntoView {
    view! {cx,
        <span class="crud-field-label">
            {label.name.clone()}
        </span>
    }
}
