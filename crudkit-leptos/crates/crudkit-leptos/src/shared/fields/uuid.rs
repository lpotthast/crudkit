use crate::shared::fields::render_label;
use crudkit_web::{FieldMode, FieldOptions, Value};
use leptonic::components::input::TextInput;
use leptos::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

#[component]
pub fn CrudUuidV4Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Uuid>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get().to_string()}</div> }.into_any(),
        // Never editable, TODO: why though? we could allow editing uuids if we can guarantee their validity.
        FieldMode::Readable | FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TextInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || { value.get().to_string() })
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudUuidV7Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Uuid>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get().to_string()}</div> }.into_any(),
        // Never editable, TODO: why though? we could allow editing uuids if we can guarantee their validity.
        FieldMode::Readable | FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TextInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || { value.get().to_string() })
                />
            </div>
        }
        .into_any(),
    }
}
