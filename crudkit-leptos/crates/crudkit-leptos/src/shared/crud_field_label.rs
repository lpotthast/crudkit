use crudkit_web::Label;
use leptos::prelude::*;

// TODO: Extract into leptonic
#[component]
pub fn CrudFieldLabel(label: Label) -> impl IntoView {
    view! { <span class="crud-field-label">{label.name.clone()}</span> }
}

// TODO: Extract into leptonic
#[component]
pub fn CrudFieldLabelOpt(label: Option<Label>) -> impl IntoView {
    label.map(|label| view! { <CrudFieldLabel label=label/> })
}
