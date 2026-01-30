use crate::fields::optional::OptionalInput;
use crate::fields::render_label;
use crudkit_core::Value;
use crudkit_web::{FieldMode, FieldOptions};
use leptonic::components::input::NumberInput;
use leptos::prelude::*;
use std::sync::Arc;

/// Helper function to render a numeric field.
fn render_number_field<T, ToF64, IntoValue, IntoOptValue>(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: Signal<Option<T>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
    to_f64: ToF64,
    into_value: IntoValue,
    into_opt_value: IntoOptValue,
) -> impl IntoView
where
    T: Copy + Default + std::fmt::Display + Send + Sync + 'static,
    ToF64: Fn(T) -> f64 + Copy + Send + Sync + 'static,
    IntoValue: Fn(f64) -> Value + Copy + Send + Sync + 'static,
    IntoOptValue: Fn(Option<T>) -> Value + Copy + Send + Sync + 'static,
{
    let number_value = Signal::derive(move || value.get().map(to_f64).unwrap_or(0.0));

    match field_mode {
        FieldMode::Display => {
            move || match value.get() {
                Some(v) => view! { {v.to_string()} }.into_any(),
                None => view! { "-" }.into_any(),
            }
        }
        .into_any(),

        FieldMode::Readable if is_optional => view! {
            {render_label(field_options.label.clone())}
            <OptionalInput
                get=value
                set={move |_: Option<T>| {}}
                disabled=true
                default_provider={move || T::default()}
                input_renderer={move |_disabled_or_null| view! {
                    <NumberInput
                        attr:id=id.clone()
                        attr:class="crud-input-field"
                        disabled=true
                        get=number_value
                    />
                }}
            />
        }
        .into_any(),

        FieldMode::Readable => view! {
            {render_label(field_options.label.clone())}
            <NumberInput
                attr:id=id.clone()
                attr:class="crud-input-field"
                disabled=true
                get=number_value
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
                        value_changed.run(Ok(into_opt_value(opt)));
                    }}
                    disabled
                    default_provider={move || T::default()}
                    input_renderer={move |disabled_or_null| view! {
                        <NumberInput
                            attr:id=id.clone()
                            attr:class="crud-input-field"
                            disabled=disabled_or_null
                            get=number_value
                            set={move |new: f64| { value_changed.run(Ok(into_value(new))) }}
                        />
                    }}
                />
            }
        }
        .into_any(),

        FieldMode::Editable => view! {
            {render_label(field_options.label.clone())}
            <NumberInput
                attr:id=id.clone()
                attr:class="crud-input-field"
                disabled=field_options.disabled
                get=number_value
                set=move |new: f64| { value_changed.run(Ok(into_value(new))) }
            />
        }
        .into_any(),
    }
}

#[component]
pub fn CrudU8Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u8>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(
        id, field_options, field_mode, value, is_optional, value_changed,
        |v| v as f64,
        |f| Value::U8(f as u8),
        |opt| opt.map(Value::U8).unwrap_or(Value::Null),
    )
}

#[component]
pub fn CrudU16Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u16>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(
        id, field_options, field_mode, value, is_optional, value_changed,
        |v| v as f64,
        |f| Value::U16(f as u16),
        |opt| opt.map(Value::U16).unwrap_or(Value::Null),
    )
}

#[component]
pub fn CrudU32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u32>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(
        id, field_options, field_mode, value, is_optional, value_changed,
        |v| v as f64,
        |f| Value::U32(f as u32),
        |opt| opt.map(Value::U32).unwrap_or(Value::Null),
    )
}

#[component]
pub fn CrudU64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u64>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(
        id, field_options, field_mode, value, is_optional, value_changed,
        |v| v as f64,
        |f| Value::U64(f as u64),
        |opt| opt.map(Value::U64).unwrap_or(Value::Null),
    )
}

#[component]
pub fn CrudU128Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u128>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(
        id, field_options, field_mode, value, is_optional, value_changed,
        |v| v as f64,
        |f| Value::U128(f as u128),
        |opt| opt.map(Value::U128).unwrap_or(Value::Null),
    )
}

#[component]
pub fn CrudI8Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i8>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(
        id, field_options, field_mode, value, is_optional, value_changed,
        |v| v as f64,
        |f| Value::I8(f as i8),
        |opt| opt.map(Value::I8).unwrap_or(Value::Null),
    )
}

#[component]
pub fn CrudI16Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i16>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(
        id, field_options, field_mode, value, is_optional, value_changed,
        |v| v as f64,
        |f| Value::I16(f as i16),
        |opt| opt.map(Value::I16).unwrap_or(Value::Null),
    )
}

#[component]
pub fn CrudI32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i32>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(
        id, field_options, field_mode, value, is_optional, value_changed,
        |v| v as f64,
        |f| Value::I32(f as i32),
        |opt| opt.map(Value::I32).unwrap_or(Value::Null),
    )
}

#[component]
pub fn CrudI64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i64>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(
        id, field_options, field_mode, value, is_optional, value_changed,
        |v| v as f64,
        |f| Value::I64(f as i64),
        |opt| opt.map(Value::I64).unwrap_or(Value::Null),
    )
}

#[component]
pub fn CrudI128Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i128>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(
        id, field_options, field_mode, value, is_optional, value_changed,
        |v| v as f64,
        |f| Value::I128(f as i128),
        |opt| opt.map(Value::I128).unwrap_or(Value::Null),
    )
}

#[component]
pub fn CrudF32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<f32>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(
        id, field_options, field_mode, value, is_optional, value_changed,
        |v| v as f64,
        |f| Value::F32(f as f32),
        |opt| opt.map(Value::F32).unwrap_or(Value::Null),
    )
}

#[component]
pub fn CrudF64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<f64>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(
        id, field_options, field_mode, value, is_optional, value_changed,
        |v| v,
        Value::F64,
        |opt| opt.map(Value::F64).unwrap_or(Value::Null),
    )
}
