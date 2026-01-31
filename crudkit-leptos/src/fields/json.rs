use crate::fields::optional::OptionalInput;
use crate::fields::render_label;
use crudkit_core::Value;
use crudkit_web::{FieldMode, FieldOptions};
use leptonic::components::prelude::*;
use leptonic::prelude::TiptapContent;
use leptos::prelude::*;
use std::sync::Arc;

/// JSON field component that handles both required and optional JSON fields.
/// The signal always holds Option<serde_json::Value> - Some(value) for values, None for null.
#[component]
pub fn CrudJsonField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<serde_json::Value>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    let json_string = Signal::derive(move || {
        value.get()
            .map(|v| serde_json::to_string(&v).unwrap_or_default())
            .unwrap_or_default()
    });

    match field_mode {
        FieldMode::Display => {
            move || match value.get() {
                Some(v) => view! { {serde_json::to_string(&v).unwrap_or_default()} }.into_any(),
                None => view! { "-" }.into_any(),
            }
        }
        .into_any(),

        FieldMode::Readable if is_optional => view! {
            {render_label(field_options.label.clone())}
            <OptionalInput
                get=value
                set={move |_: Option<serde_json::Value>| {}}
                disabled=true
                default_provider={move || serde_json::Value::Null}
                input_renderer={move |_disabled_or_null| view! {
                    <TiptapEditor
                        attr:id=id.clone()
                        attr:class="crud-input-field"
                        value=json_string
                        disabled=true
                    />
                }}
            />
        }
        .into_any(),

        FieldMode::Readable => view! {
            {render_label(field_options.label.clone())}
            <TiptapEditor
                attr:id=id.clone()
                attr:class="crud-input-field"
                value=json_string
                disabled=true
            />
        }
        .into_any(),

        FieldMode::Editable if is_optional => {
            let disabled = field_options.disabled;
            view! {
                {render_label(field_options.label.clone())}
                <OptionalInput
                    get=value
                    set={move |opt| {
                        let val = match opt {
                            Some(json) => Value::Json(json),
                            None => Value::Null,
                        };
                        value_changed.run(Ok(val));
                    }}
                    disabled
                    default_provider={move || serde_json::Value::Null}
                    input_renderer={move |disabled_or_null| view! {
                        <TiptapEditor
                            attr:id=id.clone()
                            attr:class="crud-input-field"
                            value=json_string
                            set_value={move |new| {
                                value_changed.run(
                                    match new {
                                        TiptapContent::Html(content) => serde_json::from_str(&content),
                                        TiptapContent::Json(content) => serde_json::from_str(&content),
                                    }
                                        .map(Value::Json)
                                        .map_err(|err| Arc::new(err) as Arc<dyn std::error::Error>)
                                );
                            }}
                            disabled=disabled_or_null
                        />
                    }}
                />
            }
        }
        .into_any(),

        FieldMode::Editable => view! {
            {render_label(field_options.label.clone())}
            <TiptapEditor
                attr:id=id.clone()
                attr:class="crud-input-field"
                value=json_string
                set_value=move |new| {
                    value_changed.run(
                        match new {
                            TiptapContent::Html(content) => serde_json::from_str(&content),
                            TiptapContent::Json(content) => serde_json::from_str(&content),
                        }
                            .map(Value::Json)
                            .map_err(|err| Arc::new(err) as Arc<dyn std::error::Error>)
                    );
                }

                disabled=field_options.disabled
            />
        }
        .into_any(),
    }
}
