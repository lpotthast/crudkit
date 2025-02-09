use crate::shared::fields::render_label;
use crudkit_web::{FieldMode, FieldOptions, Value};
use leptonic::components::toggle::Toggle;
use leptos::prelude::*;
use std::sync::Arc;

#[component]
pub fn CrudBoolField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<bool>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
                    <Toggle state=value disabled=true/>
                </div>
            </div>
        }.into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
                    <Toggle
                        state=value
                        set_state=move |new| { value_changed.run((Ok(Value::Bool(new)),)) }
                        disabled=field_options.disabled
                    />
                </div>
            </div>
        }.into_any(),
    }
}
