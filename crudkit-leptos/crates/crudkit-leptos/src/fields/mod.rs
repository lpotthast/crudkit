use crate::crud_field_label::CrudFieldLabelOpt;
use crate::fields::boolean::{CrudBoolField, CrudOptionalBoolField};
use crate::fields::date_time::{CrudOptionalPrimitiveDateTimeField, CrudPrimitiveDateTimeField};
use crate::fields::duration::{CrudDurationField, CrudOptionalDurationField};
use crate::fields::json::{CrudJsonField, CrudOptionalJsonField};
use crate::fields::number::{
    CrudF32Field, CrudF64Field, CrudI128Field, CrudI16Field, CrudI32Field, CrudI64Field,
    CrudI8Field, CrudOptionalF32Field, CrudOptionalF64Field, CrudOptionalI128Field,
    CrudOptionalI16Field, CrudOptionalI32Field, CrudOptionalI64Field, CrudOptionalI8Field,
    CrudOptionalU128Field, CrudOptionalU16Field, CrudOptionalU32Field, CrudOptionalU64Field,
    CrudOptionalU8Field, CrudU128Field, CrudU16Field, CrudU32Field, CrudU64Field, CrudU8Field,
};
use crate::fields::string::{CrudOptionalStringField, CrudStringField};
use crate::fields::uuid::{CrudOptionalUuidField, CrudUuidField};
use crate::fields::validation_status::CrudValidationStatusField;
use crate::ReactiveValue;
use crudkit_core::Value;
use crudkit_web::prelude::*;
use crudkit_web::{FieldMode, FieldOptions, Label};
use leptonic::components::prelude::{Alert, AlertContent, AlertTitle, AlertVariant};
use leptonic::prelude::ViewCallback;
use leptos::either::Either;
use leptos::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::sync::Arc;
use ::uuid::Uuid;

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

    /// Creates a `FieldRenderer` that uses the predefined `CrudValidationStatusField` component
    /// to render a boolean field indicating "validation errors exist" as either a checkmark
    /// when `false`, as no errors exist, or an exclamation mark when `true`, as errors do exist.
    ///
    /// # Usage
    ///
    /// In a crud instance configuration, use it like this:
    /// ```ignore
    /// read_field_renderer: FieldRendererRegistry::builder()
    ///     .register(
    ///         ReadUser::HasValidationErrors,
    ///         FieldRenderer::render_validation_status(),
    ///     )
    ///     .build(),
    /// ```
    ///
    /// # Panics
    ///
    /// When used on any field not of type `bool`!
    pub fn render_validation_status() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, _value_changed| {
                view! {
                    <CrudValidationStatusField
                        id=format!("f{}", Uuid::new_v4())
                        field_options
                        field_mode
                        value=move || value.expect_bool().get()
                    />
                }
            },
        )
    }

    pub fn render_bool() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudBoolField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=move || value.expect_bool().get()
                        value_changed=value_changed
                    />
                }
            },
        )
    }
}

pub fn render_label(label: Option<Label>) -> impl IntoView {
    view! { <CrudFieldLabelOpt label=label/> }
}

pub(crate) fn default_field_renderer<F: DynField>(
    value: ReactiveValue,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value_changed: Callback<Result<Value, Arc<dyn Error>>>,
) -> Either<FieldRenderer<F>, impl IntoView> {
    match value {
        ReactiveValue::Void(_) => Either::Right(view! {}.into_any()),
        ReactiveValue::Bool(value) => Either::Left(FieldRenderer::render_bool()),
        ReactiveValue::OptionalBool(value) => Either::Right(view! {
            <CrudOptionalBoolField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::U8(value) => Either::Right(view! {
            <CrudU8Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::U16(value) => Either::Right(view! {
            <CrudU16Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::U32(value) => Either::Right(view! {
            <CrudU32Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::U64(value) => Either::Right(view! {
            <CrudU64Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::U128(value) => Either::Right(view! {
            <CrudU128Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OptionalU8(value) => Either::Right(view! {
            <CrudOptionalU8Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OptionalU16(value) => Either::Right(view! {
            <CrudOptionalU16Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OptionalU32(value) => Either::Right(view! {
            <CrudOptionalU32Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OptionalU64(value) => Either::Right(view! {
            <CrudOptionalU64Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OptionalU128(value) => Either::Right(view! {
            <CrudOptionalU128Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),

        ReactiveValue::I8(value) => Either::Right(view! {
            <CrudI8Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::I16(value) => Either::Right(view! {
            <CrudI16Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::I32(value) => Either::Right(view! {
            <CrudI32Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::I64(value) => Either::Right(view! {
            <CrudI64Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::I128(value) => Either::Right(view! {
            <CrudI128Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),

        ReactiveValue::OptionalI8(value) => Either::Right(view! {
            <CrudOptionalI8Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OptionalI16(value) => Either::Right(view! {
            <CrudOptionalI16Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OptionalI32(value) => Either::Right(view! {
            <CrudOptionalI32Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OptionalI64(value) => Either::Right(view! {
            <CrudOptionalI64Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OptionalI128(value) => Either::Right(view! {
            <CrudOptionalI128Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),

        ReactiveValue::F32(value) => Either::Right(view! {
            <CrudF32Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::F64(value) => Either::Right(view! {
            <CrudF64Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OptionalF32(value) => Either::Right(view! {
            <CrudOptionalF32Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OptionalF64(value) => Either::Right(view! {
            <CrudOptionalF64Field
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),

        ReactiveValue::String(value) => Either::Right(view! {
            <CrudStringField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OptionalString(value) => Either::Right(view! {
            <CrudOptionalStringField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),

        // Ecosystem support.
        ReactiveValue::Json(value) => Either::Right(view! {
            <CrudJsonField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OptionalJson(value) => Either::Right(view! {
            <CrudOptionalJsonField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),

        ReactiveValue::Uuid(value) => Either::Right(view! {
            <CrudUuidField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OptionalUuid(value) => Either::Right(view! {
            <CrudOptionalUuidField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),

        ReactiveValue::PrimitiveDateTime(value) => Either::Right(view! {
            <CrudPrimitiveDateTimeField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OffsetDateTime(_) => Either::Right(
            view! { "TODO: Render ReactiveValue::OffsetDateTime" }.into_any()
        ),
        ReactiveValue::Duration(value) => Either::Right(view! {
            <CrudDurationField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OptionalPrimitiveDateTime(value) => Either::Right(view! {
            <CrudOptionalPrimitiveDateTimeField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),
        ReactiveValue::OptionalOffsetDateTime(_) => Either::Right(
            view! { "TODO: Render ReactiveValue::OptionalOffsetDateTime" }.into_any()
        ),
        ReactiveValue::OptionalDuration(value) => Either::Right(view! {
            <CrudOptionalDurationField
                id=id.clone()
                field_options=field_options
                field_mode=field_mode
                value=value
                value_changed=value_changed
            />
        }
        .into_any()),

        // Extension support.
        ReactiveValue::Other(field_value) => Either::Right(view! {
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
            .into_any()),
    }
}
