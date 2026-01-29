use crate::fields::render_label;
use crudkit_core::Value;
use crudkit_web::{FieldMode, FieldOptions};
use leptonic::components::input::NumberInput;
use leptos::prelude::*;
use std::sync::Arc;

/// Trait for non-optional numeric types that can be rendered as CRUD fields.
trait NumericValue: Copy + Default + std::fmt::Display + Send + Sync + 'static {
    fn to_f64(self) -> f64;
    fn from_f64(v: f64) -> Self;
    fn into_value(self) -> Value;
}

/// Trait for optional numeric types that can be rendered as CRUD fields.
trait OptionalNumericValue: Clone + Default + Send + Sync + 'static {
    fn to_f64(&self) -> f64;
    fn from_f64_some(v: f64) -> Self;
    fn into_value(self) -> Value;
    fn display_value(&self) -> String;
}

impl NumericValue for u8 {
    fn to_f64(self) -> f64 {
        self as f64
    }
    fn from_f64(v: f64) -> Self {
        v as u8
    }
    fn into_value(self) -> Value {
        Value::U8(self)
    }
}

impl NumericValue for u16 {
    fn to_f64(self) -> f64 {
        self as f64
    }
    fn from_f64(v: f64) -> Self {
        v as u16
    }
    fn into_value(self) -> Value {
        Value::U16(self)
    }
}

impl NumericValue for u32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
    fn from_f64(v: f64) -> Self {
        v as u32
    }
    fn into_value(self) -> Value {
        Value::U32(self)
    }
}

impl NumericValue for u64 {
    fn to_f64(self) -> f64 {
        self as f64
    }
    fn from_f64(v: f64) -> Self {
        v as u64
    }
    fn into_value(self) -> Value {
        Value::U64(self)
    }
}

impl NumericValue for u128 {
    fn to_f64(self) -> f64 {
        self as f64
    }
    fn from_f64(v: f64) -> Self {
        v as u128
    }
    fn into_value(self) -> Value {
        Value::U128(self)
    }
}

impl NumericValue for i8 {
    fn to_f64(self) -> f64 {
        self as f64
    }
    fn from_f64(v: f64) -> Self {
        v as i8
    }
    fn into_value(self) -> Value {
        Value::I8(self)
    }
}

impl NumericValue for i16 {
    fn to_f64(self) -> f64 {
        self as f64
    }
    fn from_f64(v: f64) -> Self {
        v as i16
    }
    fn into_value(self) -> Value {
        Value::I16(self)
    }
}

impl NumericValue for i32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
    fn from_f64(v: f64) -> Self {
        v as i32
    }
    fn into_value(self) -> Value {
        Value::I32(self)
    }
}

impl NumericValue for i64 {
    fn to_f64(self) -> f64 {
        self as f64
    }
    fn from_f64(v: f64) -> Self {
        v as i64
    }
    fn into_value(self) -> Value {
        Value::I64(self)
    }
}

impl NumericValue for i128 {
    fn to_f64(self) -> f64 {
        self as f64
    }
    fn from_f64(v: f64) -> Self {
        v as i128
    }
    fn into_value(self) -> Value {
        Value::I128(self)
    }
}

impl NumericValue for f32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
    fn from_f64(v: f64) -> Self {
        v as f32
    }
    fn into_value(self) -> Value {
        Value::F32(self)
    }
}

impl NumericValue for f64 {
    fn to_f64(self) -> f64 {
        self
    }
    fn from_f64(v: f64) -> Self {
        v
    }
    fn into_value(self) -> Value {
        Value::F64(self)
    }
}

impl OptionalNumericValue for Option<u8> {
    fn to_f64(&self) -> f64 {
        self.unwrap_or_default() as f64
    }
    fn from_f64_some(v: f64) -> Self {
        Some(v as u8)
    }
    fn into_value(self) -> Value {
        Value::OptionalU8(self)
    }
    fn display_value(&self) -> String {
        self.map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

impl OptionalNumericValue for Option<u16> {
    fn to_f64(&self) -> f64 {
        self.unwrap_or_default() as f64
    }
    fn from_f64_some(v: f64) -> Self {
        Some(v as u16)
    }
    fn into_value(self) -> Value {
        Value::OptionalU16(self)
    }
    fn display_value(&self) -> String {
        self.map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

impl OptionalNumericValue for Option<u32> {
    fn to_f64(&self) -> f64 {
        self.unwrap_or_default() as f64
    }
    fn from_f64_some(v: f64) -> Self {
        Some(v as u32)
    }
    fn into_value(self) -> Value {
        Value::OptionalU32(self)
    }
    fn display_value(&self) -> String {
        self.map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

impl OptionalNumericValue for Option<u64> {
    fn to_f64(&self) -> f64 {
        self.unwrap_or_default() as f64
    }
    fn from_f64_some(v: f64) -> Self {
        Some(v as u64)
    }
    fn into_value(self) -> Value {
        Value::OptionalU64(self)
    }
    fn display_value(&self) -> String {
        self.map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

impl OptionalNumericValue for Option<u128> {
    fn to_f64(&self) -> f64 {
        self.unwrap_or_default() as f64
    }
    fn from_f64_some(v: f64) -> Self {
        Some(v as u128)
    }
    fn into_value(self) -> Value {
        Value::OptionalU128(self)
    }
    fn display_value(&self) -> String {
        self.map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

impl OptionalNumericValue for Option<i8> {
    fn to_f64(&self) -> f64 {
        self.unwrap_or_default() as f64
    }
    fn from_f64_some(v: f64) -> Self {
        Some(v as i8)
    }
    fn into_value(self) -> Value {
        Value::OptionalI8(self)
    }
    fn display_value(&self) -> String {
        self.map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

impl OptionalNumericValue for Option<i16> {
    fn to_f64(&self) -> f64 {
        self.unwrap_or_default() as f64
    }
    fn from_f64_some(v: f64) -> Self {
        Some(v as i16)
    }
    fn into_value(self) -> Value {
        Value::OptionalI16(self)
    }
    fn display_value(&self) -> String {
        self.map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

impl OptionalNumericValue for Option<i32> {
    fn to_f64(&self) -> f64 {
        self.unwrap_or_default() as f64
    }
    fn from_f64_some(v: f64) -> Self {
        Some(v as i32)
    }
    fn into_value(self) -> Value {
        Value::OptionalI32(self)
    }
    fn display_value(&self) -> String {
        self.map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

impl OptionalNumericValue for Option<i64> {
    fn to_f64(&self) -> f64 {
        self.unwrap_or_default() as f64
    }
    fn from_f64_some(v: f64) -> Self {
        Some(v as i64)
    }
    fn into_value(self) -> Value {
        Value::OptionalI64(self)
    }
    fn display_value(&self) -> String {
        self.map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

impl OptionalNumericValue for Option<i128> {
    fn to_f64(&self) -> f64 {
        self.unwrap_or_default() as f64
    }
    fn from_f64_some(v: f64) -> Self {
        Some(v as i128)
    }
    fn into_value(self) -> Value {
        Value::OptionalI128(self)
    }
    fn display_value(&self) -> String {
        self.map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

impl OptionalNumericValue for Option<f32> {
    fn to_f64(&self) -> f64 {
        self.unwrap_or_default() as f64
    }
    fn from_f64_some(v: f64) -> Self {
        Some(v as f32)
    }
    fn into_value(self) -> Value {
        Value::OptionalF32(self)
    }
    fn display_value(&self) -> String {
        self.map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

impl OptionalNumericValue for Option<f64> {
    fn to_f64(&self) -> f64 {
        self.unwrap_or_default()
    }
    fn from_f64_some(v: f64) -> Self {
        Some(v)
    }
    fn into_value(self) -> Value {
        Value::OptionalF64(self)
    }
    fn display_value(&self) -> String {
        self.map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

fn render_number_field<T: NumericValue>(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: Signal<T>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { {move || value.get().to_string()} }.into_any(),
        FieldMode::Readable => view! {
            {render_label(field_options.label.clone())}
            <NumberInput
                attr:id=id.clone()
                attr:class="crud-input-field"
                disabled=true
                get=Signal::derive(move || value.get().to_f64())
            />
        }
        .into_any(),
        FieldMode::Editable => view! {
            {render_label(field_options.label.clone())}
            <NumberInput
                attr:id=id.clone()
                attr:class="crud-input-field"
                disabled=field_options.disabled
                get=Signal::derive(move || value.get().to_f64())
                set=move |new: f64| { value_changed.run(Ok(T::from_f64(new).into_value())) }
            />
        }
        .into_any(),
    }
}

fn render_optional_number_field<T: OptionalNumericValue>(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: Signal<T>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { {move || value.get().display_value()} }.into_any(),
        FieldMode::Readable => view! {
            {render_label(field_options.label.clone())}
            <NumberInput
                attr:id=id.clone()
                attr:class="crud-input-field"
                disabled=true
                get=Signal::derive(move || value.get().to_f64())
            />
        }
        .into_any(),
        FieldMode::Editable => view! {
            {render_label(field_options.label.clone())}
            <NumberInput
                attr:id=id.clone()
                attr:class="crud-input-field"
                disabled=field_options.disabled
                get=Signal::derive(move || value.get().to_f64())
                set=move |new: f64| { value_changed.run(Ok(T::from_f64_some(new).into_value())) }
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
    #[prop(into)] value: Signal<u8>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudU16Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<u16>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudU32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<u32>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudU64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<u64>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudU128Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<u128>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudI8Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<i8>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudI16Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<i16>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudI32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<i32>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudI64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<i64>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudI128Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<i128>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudF32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<f32>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudF64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<f64>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudOptionalU8Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u8>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_optional_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudOptionalU16Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u16>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_optional_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudOptionalU32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u32>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_optional_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudOptionalU64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u64>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_optional_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudOptionalU128Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u128>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_optional_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudOptionalI8Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i8>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_optional_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudOptionalI16Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i16>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_optional_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudOptionalI32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i32>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_optional_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudOptionalI64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i64>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_optional_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudOptionalI128Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i128>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_optional_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudOptionalF32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<f32>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_optional_number_field(id, field_options, field_mode, value, value_changed)
}

#[component]
pub fn CrudOptionalF64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<f64>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    render_optional_number_field(id, field_options, field_mode, value, value_changed)
}
