use crate::fields::render_label;
use crudkit_core::Value;
use crudkit_web::{FieldMode, FieldOptions};
use leptonic::components::input::TextInput;
use leptos::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

#[component]
pub fn CrudUuidField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Uuid>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => (move || value.get().to_string()).into_any(),
        // Never editable, TODO: why though? we could allow editing uuids if we can guarantee their validity.
        FieldMode::Readable | FieldMode::Editable => view! {
            {render_label(field_options.label.clone())}
            <TextInput
                attr:id=id.clone()
                attr:class="crud-input-field"
                disabled=true
                get=Signal::derive(move || { value.get().to_string() })
            />
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalUuidField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<Uuid>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => (move || match value.get() {
            Some(uuid) => uuid.to_string(),
            None => "-".to_string(),
        })
        .into_any(),
        // Never editable, TODO: why though? we could allow editing uuids if we can guarantee their validity.
        FieldMode::Readable | FieldMode::Editable => view! {
            {render_label(field_options.label.clone())}
            <TextInput
                attr:id=id.clone()
                attr:class="crud-input-field"
                disabled=true
                get=Signal::derive(move || match value.get() {
                    Some(uuid) => uuid.to_string(),
                    None => "-".to_string(),
                })
            />
        }
        .into_any(),
    }
}
