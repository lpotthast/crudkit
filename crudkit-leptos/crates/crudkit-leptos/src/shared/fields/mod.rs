use crate::shared::crud_field_label::CrudFieldLabelOpt;
use crate::shared::crud_instance_config::SelectConfigTrait;
use crate::shared::fields::boolean::CrudBoolField;
use crate::shared::fields::date_time::{
    CrudOptionalPrimitiveDateTimeField, CrudPrimitiveDateTimeField,
};
use crate::shared::fields::duration::{CrudDurationField, CrudOptionalDurationField};
use crate::shared::fields::json::{CrudJsonField, CrudOptionalJsonField};
use crate::shared::fields::number::{
    CrudF32Field, CrudF64Field, CrudI128Field, CrudI32Field, CrudI64Field, CrudOptionalI128Field,
    CrudOptionalI32Field, CrudOptionalI64Field, CrudOptionalU128Field, CrudOptionalU32Field,
    CrudOptionalU64Field, CrudU128Field, CrudU32Field, CrudU64Field,
};
use crate::shared::fields::select::{CrudOptionalSelectField, CrudSelectField};
use crate::shared::fields::string::{CrudOptionalStringField, CrudStringField};
use crate::shared::fields::uuid::{CrudOptionalUuidField, CrudUuidField};
use crate::shared::fields::validation_status::CrudValidationStatusField;
use crate::ReactiveValue;
use crudkit_web::{FieldMode, FieldOptions, Label, Value};
use leptos::prelude::*;
use std::error::Error;
use std::sync::Arc;

pub mod boolean;
pub mod date_time;
pub mod duration;
pub mod json;
pub mod number;
pub mod select;
pub mod string;
pub mod text;
pub mod uuid;
pub mod validation_status;

pub fn render_label(label: Option<Label>) -> impl IntoView {
    view! { <CrudFieldLabelOpt label=label/> }
}

#[inline(never)]
pub fn render_field(
    value: ReactiveValue,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    field_config: Option<Box<dyn SelectConfigTrait>>,
    value_changed: Callback<Result<Value, Arc<dyn Error>>>,
    // TODO: can this be ViewFnOnce?
    custom_field_renderer: Option<ViewFn>,
) -> AnyView {
    match custom_field_renderer {
        Some(custom_field_renderer) => {
            // This additional closure is required so that each custom field, which may be another
            // crud instance, or, in general, anything that might `provide_context(T)`, have their
            // own context to do so and not override sibling data.
            (move || custom_field_renderer.run()).into_any()
        }
        None => default_field_renderer(
            value,
            id,
            field_options,
            field_mode,
            field_config,
            value_changed,
        )
        .into_any(),
    }
}

fn default_field_renderer(
    value: ReactiveValue,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    field_config: Option<Box<dyn SelectConfigTrait>>,
    value_changed: Callback<Result<Value, Arc<dyn Error>>>,
) -> impl IntoView {
    match value {
        ReactiveValue::String(value) => view! {
            <CrudStringField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::OptionalString(value) => view! {
            <CrudOptionalStringField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        //ReactiveValue::Text(value) => view! {
        //    <CrudTextField
        //        id=id.clone()
        //        field_options=field_options
        //        field_mode=field_mode
        //        value=value
        //        value_changed=value_changed
        //    />
        //}
        //.into_any(),
        ReactiveValue::Json(value) => view! {
            <CrudJsonField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::OptionalJson(value) => view! {
            <CrudOptionalJsonField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::Uuid(value) => view! {
            <CrudUuidField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::OptionalUuid(value) => view! {
            <CrudOptionalUuidField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::I32(value) => view! {
            <CrudI32Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::I64(value) => view! {
            <CrudI64Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::U32(value) => view! {
            <CrudU32Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::U64(value) => view! {
            <CrudU64Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::I128(value) => view! {
            <CrudI128Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::U128(value) => view! {
            <CrudU128Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::OptionalU32(value) => view! {
            <CrudOptionalU32Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::OptionalU64(value) => view! {
            <CrudOptionalU64Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::OptionalI32(value) => view! {
            <CrudOptionalI32Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::OptionalI64(value) => view! {
            <CrudOptionalI64Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::OptionalI128(value) => view! {
            <CrudOptionalI128Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::OptionalU128(value) => view! {
            <CrudOptionalU128Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::F32(value) => view! {
            <CrudF32Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::F64(value) => view! {
            <CrudF64Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::Bool(value) => view! {
            <CrudBoolField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::ValidationStatus(value) => view! {
            <CrudValidationStatusField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value/>
        }
        .into_any(),
        ReactiveValue::PrimitiveDateTime(value) => view! {
            <CrudPrimitiveDateTimeField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::OffsetDateTime(_) => {
            view! { "TODO: Render ReactiveValue::OffsetDateTime" }.into_any()
        }
        ReactiveValue::OptionalPrimitiveDateTime(value) => view! {
            <CrudOptionalPrimitiveDateTimeField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::OptionalOffsetDateTime(_) => {
            view! { "TODO: Render ReactiveValue::OptionalOffsetDateTime" }.into_any()
        }
        ReactiveValue::OneToOneRelation(_) => {
            view! { "TODO: Render ReactiveValue::OneToOneRelation" }.into_any()
        }
        ReactiveValue::Reference(_) => {
            view! { "TODO: Render ReactiveValue::NestedTable" }.into_any()
        }
        ReactiveValue::Custom(_) => panic!("should have had custom field renderer"), // custom_field_renderer(),
        ReactiveValue::Select(value) => view! {
            <CrudSelectField
                id=id.clone()
                field_config=field_config
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::Multiselect(_) => {
            view! { "TODO: Render ReactiveValue::Multiselect" }.into_any()
        }
        ReactiveValue::OptionalSelect(value) => view! {
            <CrudOptionalSelectField
                id=id.clone()
                field_config=field_config
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::OptionalMultiselect(_) => {
            view! { "TODO: Render ReactiveValue::OptionalMultiselect" }.into_any()
        }
        ReactiveValue::Duration(value) => view! {
            <CrudDurationField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::OptionalDuration(value) => view! {
            <CrudOptionalDurationField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
    }
}
