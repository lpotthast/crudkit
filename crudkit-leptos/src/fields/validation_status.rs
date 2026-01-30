use crate::fields::render_label;
use crudkit_web::{FieldMode, FieldOptions};
use leptonic::components::icon::Icon;
use leptonic::prelude::icondata;
use leptos::prelude::*;

/// Validation status field component that displays a checkmark or exclamation mark
/// based on whether validation errors exist.
#[component]
pub fn CrudValidationStatusField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<bool>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => (move || match value.get() {
            true => view! { <Icon icon=icondata::BsExclamationTriangleFill/> },
            false => view! { <Icon icon=icondata::BsCheck/> },
        })
        .into_any(),
        FieldMode::Readable | FieldMode::Editable => view! {
            {render_label(field_options.label.clone())}
            <div id=id.clone() class="crud-input-field">
                {move || match value.get() {
                    true => view! { <Icon icon=icondata::BsExclamationTriangleFill/> },
                    false => view! { <Icon icon=icondata::BsCheck/> },
                }}
            </div>
        }
        .into_any(),
    }
}
