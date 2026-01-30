use crate::fields::optional::OptionalInput;
use crate::fields::render_label;
use crudkit_core::Value;
use crudkit_web::{FieldMode, FieldOptions};
use leptonic::components::input::TextInput;
use leptos::prelude::*;
use std::sync::Arc;

/// String field component that handles both required and optional string fields.
/// The signal always holds Option<String> - Some(value) for values, None for null.
#[component]
pub fn CrudStringField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<String>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    let text_value = Signal::derive(move || value.get().unwrap_or_default());

    match field_mode {
        FieldMode::Display => {
            move || match value.get() {
                Some(v) => view! { {v} }.into_any(),
                None => view! { "-" }.into_any(),
            }
        }
        .into_any(),

        FieldMode::Readable if is_optional => view! {
            {render_label(field_options.label.clone())}
            <OptionalInput
                get=value
                set={move |_: Option<String>| {}}
                disabled=true
                default_provider={move || String::new()}
                input_renderer={move |_disabled_or_null| view! {
                    <TextInput
                        attr:id=id.clone()
                        attr:class="crud-input-field"
                        disabled=true
                        get=text_value
                    />
                }}
            />
        }
        .into_any(),

        FieldMode::Readable => view! {
            {render_label(field_options.label.clone())}
            <TextInput
                attr:id=id.clone()
                attr:class="crud-input-field"
                disabled=true
                get=text_value
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
                            Some(s) => Value::String(s),
                            None => Value::Null,
                        };
                        value_changed.run(Ok(val));
                    }}
                    disabled
                    default_provider={move || String::new()}
                    input_renderer={move |disabled_or_null| view! {
                        <TextInput
                            attr:id=id.clone()
                            attr:class="crud-input-field"
                            disabled=disabled_or_null
                            get=text_value
                            set={move |new| value_changed.run(Ok(Value::String(new)))}
                        />
                    }}
                />
            }
        }
        .into_any(),

        FieldMode::Editable => view! {
            {render_label(field_options.label.clone())}
            <TextInput
                attr:id=id.clone()
                attr:class="crud-input-field"
                disabled=field_options.disabled
                get=text_value
                set=move |new| value_changed.run(Ok(Value::String(new)))
            />
        }
        .into_any(),
    }
}
