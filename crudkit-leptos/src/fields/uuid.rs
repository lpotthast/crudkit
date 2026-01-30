use crate::fields::optional::OptionalInput;
use crate::fields::render_label;
use crudkit_core::Value;
use crudkit_web::{FieldMode, FieldOptions};
use leptonic::components::input::TextInput;
use leptos::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

/// UUID field component that handles both required and optional UUID fields.
/// The signal always holds Option<Uuid> - Some(value) for values, None for null.
#[component]
pub fn CrudUuidField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<Uuid>>,
    is_optional: bool,
    #[allow(unused_variables)]
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    let uuid_string = Signal::derive(move || match value.get() {
        Some(uuid) => uuid.to_string(),
        None => "-".to_string(),
    });

    match field_mode {
        FieldMode::Display => {
            move || match value.get() {
                Some(uuid) => view! { {uuid.to_string()} }.into_any(),
                None => view! { "-" }.into_any(),
            }
        }
        .into_any(),

        // UUIDs are never editable (auto-generated), but we show the null toggle for optional fields.
        FieldMode::Readable | FieldMode::Editable if is_optional => view! {
            {render_label(field_options.label.clone())}
            <OptionalInput
                get=value
                set={move |_: Option<Uuid>| {}}
                disabled=true
                default_provider={move || Uuid::new_v4()}
                input_renderer={move |_disabled_or_null| view! {
                    <TextInput
                        attr:id=id.clone()
                        attr:class="crud-input-field"
                        disabled=true
                        get=uuid_string
                    />
                }}
            />
        }
        .into_any(),

        FieldMode::Readable | FieldMode::Editable => view! {
            {render_label(field_options.label.clone())}
            <TextInput
                attr:id=id.clone()
                attr:class="crud-input-field"
                disabled=true
                get=uuid_string
            />
        }
        .into_any(),
    }
}
