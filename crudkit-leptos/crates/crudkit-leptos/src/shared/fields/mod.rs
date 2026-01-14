use crate::ReactiveValue;
use crate::shared::crud_field_label::CrudFieldLabelOpt;
use crate::shared::fields::boolean::{CrudBoolField, CrudOptionalBoolField};
use crate::shared::fields::date_time::{
    CrudOptionalPrimitiveDateTimeField, CrudPrimitiveDateTimeField,
};
use crate::shared::fields::duration::{CrudDurationField, CrudOptionalDurationField};
use crate::shared::fields::json::{CrudJsonField, CrudOptionalJsonField};
use crate::shared::fields::number::{
    CrudF32Field, CrudF64Field, CrudI8Field, CrudI16Field, CrudI32Field, CrudI64Field,
    CrudI128Field, CrudOptionalF32Field, CrudOptionalF64Field, CrudOptionalI8Field,
    CrudOptionalI16Field, CrudOptionalI32Field, CrudOptionalI64Field, CrudOptionalI128Field,
    CrudOptionalU8Field, CrudOptionalU16Field, CrudOptionalU32Field, CrudOptionalU64Field,
    CrudOptionalU128Field, CrudU8Field, CrudU16Field, CrudU32Field, CrudU64Field, CrudU128Field,
};
use crate::shared::fields::string::{CrudOptionalStringField, CrudStringField};
use crate::shared::fields::uuid::{CrudOptionalUuidField, CrudUuidField};
use crudkit_core::Value;
use crudkit_web::dynamic::prelude::*;
use crudkit_web::{FieldMode, FieldOptions, Label};
use leptonic::components::prelude::{Alert, AlertContent, AlertTitle, AlertVariant};
use leptonic::prelude::ViewCallback;
use leptos::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::sync::Arc;

pub mod boolean;
pub mod date_time;
pub mod duration;
pub mod json;
pub mod number;
pub mod optional;
pub mod select;
pub mod string;
pub mod text;
pub mod uuid;
pub mod validation_status;

#[derive(Debug, Clone)]
pub struct FieldRenderer<F: DynField> {
    pub(crate) view_cb: ViewCallback<(
        StoredValue<HashMap<F, ReactiveValue>>,              // signals
        FieldMode,                                           // field_mode
        FieldOptions,                                        // field_options
        ReactiveValue,                                       // value
        Callback<Result<Value, Arc<dyn std::error::Error>>>, // value_changed
    )>,
}

impl<F: DynField> FieldRenderer<F> {
    pub fn new<C: RenderHtml>(
        view_fn: impl Fn(
            StoredValue<HashMap<F, ReactiveValue>>,
            FieldMode,
            FieldOptions,
            ReactiveValue,
            Callback<Result<Value, Arc<dyn std::error::Error>>>,
        ) -> C
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            view_cb: ViewCallback::new(Callback::from(move |a1, a2, a3, a4, a5| {
                view_fn(a1, a2, a3, a4, a5).into_any()
            })),
        }
    }
}

pub fn render_label(label: Option<Label>) -> impl IntoView {
    view! { <CrudFieldLabelOpt label=label/> }
}

pub(crate) fn default_field_renderer(
    value: ReactiveValue,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value_changed: Callback<Result<Value, Arc<dyn Error>>>,
) -> impl IntoView {
    match value {
        ReactiveValue::Void(_) => view! {}.into_any(),

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
        ReactiveValue::OptionalBool(value) => view! {
            <CrudOptionalBoolField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),

        ReactiveValue::U8(value) => view! {
            <CrudU8Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::U16(value) => view! {
            <CrudU16Field
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
        ReactiveValue::OptionalU8(value) => view! {
            <CrudOptionalU8Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::OptionalU16(value) => view! {
            <CrudOptionalU16Field
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

        ReactiveValue::I8(value) => view! {
            <CrudI8Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::I16(value) => view! {
            <CrudI16Field
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

        ReactiveValue::OptionalI8(value) => view! {
            <CrudOptionalI8Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::OptionalI16(value) => view! {
            <CrudOptionalI16Field
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
        ReactiveValue::OptionalF32(value) => view! {
            <CrudOptionalF32Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),
        ReactiveValue::OptionalF64(value) => view! {
            <CrudOptionalF64Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any(),

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

        // Ecosystem support.
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

        // Extension support.
        ReactiveValue::Other(field_value) => view! {
            <Alert variant=AlertVariant::Danger>
                <AlertTitle slot>"Invalid Configuration"</AlertTitle>
                <AlertContent slot>
                    <p>
                        "The field labeled '"
                        {format!("{:?}", field_options.label)}
                        "' has a custom type that can only be rendered wit a user-specified field renderer. But no renderer was registered for that field in the instance config. You might have forgotten to set the required HashMap entry."
                    </p>
                    <p>"The current value of the field is:"</p>
                    <pre>
                        {format!("{:?}", field_value.get())}
                    </pre>
                </AlertContent>
            </Alert>
        }
            .into_any(),
    }
}
