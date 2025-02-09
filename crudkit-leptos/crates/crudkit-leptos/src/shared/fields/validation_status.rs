use crate::shared::fields::render_label;
use crudkit_web::{FieldMode, FieldOptions};
use leptonic::components::icon::Icon;
use leptonic::prelude::icondata;
use leptos::prelude::*;

#[component]
pub fn CrudValidationStatusField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<bool>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {
            <div>
                {move || match value.get() {
                    true => view! { <Icon icon=icondata::BsExclamationTriangleFill/> },
                    false => view! { <Icon icon=icondata::BsCheck/> },
                }}

            </div>
        }.into_any(),
        FieldMode::Readable | FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
                    {move || match value.get() {
                        true => view! { <Icon icon=icondata::BsExclamationTriangleFill/> },
                        false => view! { <Icon icon=icondata::BsCheck/> },
                    }}
                </div>
            </div>
        }.into_any(),
    }
}
