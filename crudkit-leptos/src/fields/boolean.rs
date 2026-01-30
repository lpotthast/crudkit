use crate::fields::optional::OptionalInput;
use crate::fields::render_label;
use crudkit_core::Value;
use crudkit_web::{FieldMode, FieldOptions};
use leptonic::components::toggle::Toggle;
use leptos::prelude::*;
use std::sync::Arc;

/// Boolean field component that handles both required and optional bool fields.
/// The signal always holds Option<bool> - Some(value) for values, None for null.
#[component]
pub fn CrudBoolField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<bool>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    let toggle_state = Signal::derive(move || value.get().unwrap_or(false));

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
            <div id=id.clone() class="crud-input-field">
                <OptionalInput
                    get=value
                    set={move |_: Option<bool>| {}}
                    disabled=true
                    default_provider={move || false}
                    input_renderer={move |_disabled_or_null| view! {
                        <Toggle
                            state=toggle_state
                            disabled=true
                        />
                    }}
                />
            </div>
        }
        .into_any(),

        FieldMode::Readable => view! {
            {render_label(field_options.label.clone())}
            <div id=id.clone() class="crud-input-field">
                <Toggle state=toggle_state disabled=true/>
            </div>
        }
        .into_any(),

        FieldMode::Editable if is_optional => {
            let disabled = field_options.disabled;
            view! {
                {render_label(field_options.label.clone())}
                <div id=id.clone() class="crud-input-field">
                    <OptionalInput
                        get=value
                        set={move |opt| {
                            let val = match opt {
                                Some(b) => Value::Bool(b),
                                None => Value::Null,
                            };
                            value_changed.run(Ok(val));
                        }}
                        disabled
                        default_provider={move || false}
                        input_renderer={move |disabled_or_null| view! {
                            <Toggle
                                state=toggle_state
                                set_state={move |new| {
                                    value_changed.run(Ok(Value::Bool(new)));
                                }}
                                disabled=disabled_or_null
                            />
                        }}
                    />
                </div>
            }
        }
        .into_any(),

        FieldMode::Editable => view! {
            {render_label(field_options.label.clone())}
            <div id=id.clone() class="crud-input-field">
                <Toggle
                    state=toggle_state
                    set_state=move |new| {
                        value_changed.run(Ok(Value::Bool(new)));
                    }
                    disabled=field_options.disabled
                />
            </div>
        }
        .into_any(),
    }
}
