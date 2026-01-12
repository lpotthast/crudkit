use crate::shared::fields::optional::OptionalInput;
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
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
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
                        set_state=move |new| { value_changed.run(Ok(Value::Bool(new))) }
                        disabled=field_options.disabled
                    />
                </div>
            </div>
        }.into_any(),
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
                Some(value) => view! { <div>{value}</div> }.into_any(),
                None => view! { <div>"-"</div> }.into_any(),
            }
        }
            .into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
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
            </div>
        }.into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
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
            </div>
        }.into_any(),
    }
}
