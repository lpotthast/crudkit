use crate::fields::render_label;
use crudkit_core::Value;
use crudkit_web::{FieldMode, FieldOptions};
use leptonic::components::input::TextInput;
use leptos::prelude::*;
use std::sync::Arc;

#[component]
pub fn CrudStringField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<String>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TextInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=value
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TextInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=value
                    set=move |new| value_changed.run(Ok(Value::String(new)))
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalStringField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<String>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TextInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get().unwrap_or_default())
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TextInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get().unwrap_or_default())
                    set=move |new| value_changed.run(Ok(Value::OptionalString(Some(new))))
                />
            </div>
        }
        .into_any(),
    }
}
