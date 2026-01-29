use crate::fields::optional::OptionalInput;
use crate::fields::render_label;
use crudkit_core::Value;
use crudkit_web::{FieldMode, FieldOptions};
use leptonic::components::toggle::Toggle;
use leptos::prelude::*;
use std::sync::Arc;

#[component]
pub fn CrudBoolField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<bool>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => (move || value.get()).into_any(),
        FieldMode::Readable => view! {
            {render_label(field_options.label.clone())}
            <div id=id.clone() class="crud-input-field">
                <Toggle state=value disabled=true/>
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            {render_label(field_options.label.clone())}
            <div id=id.clone() class="crud-input-field">
                <Toggle
                    state=value
                    set_state=move |new| { value_changed.run(Ok(Value::Bool(new))) }
                    disabled=field_options.disabled
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalBoolField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<bool>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => {
            move || match value.get() {
                Some(value) => view! { {value} }.into_any(),
                None => view! { "-" }.into_any(),
            }
        }
        .into_any(),
        FieldMode::Readable => view! {
            {render_label(field_options.label.clone())}
            <div id=id.clone() class="crud-input-field">
                <OptionalInput
                    get=value
                    set=move |new| {
                        value_changed.run(Ok(Value::OptionalBool(new)));
                    }
                    disabled=field_options.disabled
                    default_provider=move || false
                    input_renderer=move |disabled_or_null| view! {
                        <Toggle
                            state=Signal::derive(move || value.get().unwrap_or_default())
                            set_state=move |new| {
                                value_changed.run(Ok(Value::OptionalBool(Some(new))));
                            }
                            disabled=disabled_or_null
                        />
                    }
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            {render_label(field_options.label.clone())}
            <div id=id.clone() class="crud-input-field">
                <OptionalInput
                    get=value
                    set=move |new| {
                        value_changed.run(Ok(Value::OptionalBool(new)));
                    }
                    disabled=field_options.disabled
                    default_provider=move || false
                    input_renderer=move |disabled_or_null| view! {
                        <Toggle
                            state=Signal::derive(move || value.get().unwrap_or_default())
                            set_state=move |new| {
                                value_changed.run(Ok(Value::OptionalBool(Some(new))));
                            }
                            disabled=disabled_or_null
                        />
                    }
                />
            </div>
        }
        .into_any(),
    }
}
