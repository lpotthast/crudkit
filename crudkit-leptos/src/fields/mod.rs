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
use leptos::prelude::*;
use std::collections::HashMap;
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
pub struct FieldRenderer<F: TypeErasedField> {
    pub(crate) view_cb: ViewCallback<(
        StoredValue<HashMap<F, ReactiveValue>>,              // signals
        FieldMode,                                           // field_mode
        FieldOptions,                                        // field_options
        ReactiveValue,                                       // value
        Callback<Result<Value, Arc<dyn std::error::Error>>>, // value_changed
    )>,
}

impl<F: TypeErasedField> FieldRenderer<F> {
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
                view! {
                    <div class="crud-field">
                        { view_fn(a1, a2, a3, a4, a5).into_any() }
                    </div>
                }
                .into_any()
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
    ///         FieldRenderer::for_validation_status(),
    ///     )
    ///     .build(),
    /// ```
    ///
    /// # Panics
    ///
    /// When used on any field not of type `bool`!
    pub fn for_validation_status() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, _value_changed| {
                view! {
                    <CrudValidationStatusField
                        id=format!("f{}", Uuid::new_v4())
                        field_options
                        field_mode
                        value=value.expect_bool()
                    />
                }
            },
        )
    }

    pub fn for_void() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, _field_mode, _field_options, _value, _value_changed| {
                view! {}
            },
        )
    }

    pub fn for_bool() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudBoolField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_bool()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_bool() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalBoolField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_bool()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_u8() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudU8Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_u8()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_u16() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudU16Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_u16()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_u32() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudU32Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_u32()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_u64() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudU64Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_u64()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_u128() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudU128Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_u128()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_u8() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalU8Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_u8()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_u16() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalU16Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_u16()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_u32() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalU32Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_u32()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_u64() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalU64Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_u64()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_u128() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalU128Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_u128()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_i8() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudI8Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_i8()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_i16() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudI16Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_i16()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_i32() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudI32Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_i32()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_i64() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudI64Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_i64()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_i128() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudI128Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_i128()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_i8() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalI8Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_i8()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_i16() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalI16Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_i16()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_i32() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalI32Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_i32()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_i64() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalI64Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_i64()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_i128() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalI128Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_i128()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_f32() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudF32Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_f32()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_f64() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudF64Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_f64()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_f32() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalF32Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_f32()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_f64() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalF64Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_f64()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_string() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudStringField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_string()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_string() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalStringField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_string()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_json() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudJsonField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_json()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_json() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalJsonField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_json()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_uuid() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudUuidField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_uuid()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_uuid() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalUuidField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_uuid()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_primitive_date_time() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudPrimitiveDateTimeField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_primitive_date_time()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_primitive_date_time() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalPrimitiveDateTimeField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_primitive_date_time()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_offset_date_time() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, _field_mode, _field_options, _value, _value_changed| {
                view! { "TODO: Render ReactiveValue::OffsetDateTime" }
            },
        )
    }

    pub fn for_optional_offset_date_time() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, _field_mode, _field_options, _value, _value_changed| {
                view! { "TODO: Render ReactiveValue::OptionalOffsetDateTime" }
            },
        )
    }

    pub fn for_duration() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudDurationField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_duration()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_optional_duration() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field_mode, field_options, value, value_changed| {
                view! {
                    <CrudOptionalDurationField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=value.expect_optional_duration()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_other() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, _field_mode, field_options, value, _value_changed| {
                let field_value = value.expect_custom();
                view! {
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
                                {move || format!("{:?}", field_value.get())}
                            </pre>
                        </AlertContent>
                    </Alert>
                }
            },
        )
    }
}

pub fn render_label(label: Option<Label>) -> impl IntoView {
    view! { <CrudFieldLabelOpt label=label/> }
}

pub(crate) fn default_field_renderer<F: TypeErasedField>(value: ReactiveValue) -> FieldRenderer<F> {
    match value {
        ReactiveValue::Void(_) => FieldRenderer::for_void(),
        ReactiveValue::Bool(_) => FieldRenderer::for_bool(),
        ReactiveValue::OptionalBool(_) => FieldRenderer::for_optional_bool(),
        ReactiveValue::U8(_) => FieldRenderer::for_u8(),
        ReactiveValue::U16(_) => FieldRenderer::for_u16(),
        ReactiveValue::U32(_) => FieldRenderer::for_u32(),
        ReactiveValue::U64(_) => FieldRenderer::for_u64(),
        ReactiveValue::U128(_) => FieldRenderer::for_u128(),
        ReactiveValue::OptionalU8(_) => FieldRenderer::for_optional_u8(),
        ReactiveValue::OptionalU16(_) => FieldRenderer::for_optional_u16(),
        ReactiveValue::OptionalU32(_) => FieldRenderer::for_optional_u32(),
        ReactiveValue::OptionalU64(_) => FieldRenderer::for_optional_u64(),
        ReactiveValue::OptionalU128(_) => FieldRenderer::for_optional_u128(),
        ReactiveValue::I8(_) => FieldRenderer::for_i8(),
        ReactiveValue::I16(_) => FieldRenderer::for_i16(),
        ReactiveValue::I32(_) => FieldRenderer::for_i32(),
        ReactiveValue::I64(_) => FieldRenderer::for_i64(),
        ReactiveValue::I128(_) => FieldRenderer::for_i128(),
        ReactiveValue::OptionalI8(_) => FieldRenderer::for_optional_i8(),
        ReactiveValue::OptionalI16(_) => FieldRenderer::for_optional_i16(),
        ReactiveValue::OptionalI32(_) => FieldRenderer::for_optional_i32(),
        ReactiveValue::OptionalI64(_) => FieldRenderer::for_optional_i64(),
        ReactiveValue::OptionalI128(_) => FieldRenderer::for_optional_i128(),
        ReactiveValue::F32(_) => FieldRenderer::for_f32(),
        ReactiveValue::F64(_) => FieldRenderer::for_f64(),
        ReactiveValue::OptionalF32(_) => FieldRenderer::for_optional_f32(),
        ReactiveValue::OptionalF64(_) => FieldRenderer::for_optional_f64(),
        ReactiveValue::String(_) => FieldRenderer::for_string(),
        ReactiveValue::OptionalString(_) => FieldRenderer::for_optional_string(),
        ReactiveValue::Json(_) => FieldRenderer::for_json(),
        ReactiveValue::OptionalJson(_) => FieldRenderer::for_optional_json(),
        ReactiveValue::Uuid(_) => FieldRenderer::for_uuid(),
        ReactiveValue::OptionalUuid(_) => FieldRenderer::for_optional_uuid(),
        ReactiveValue::PrimitiveDateTime(_) => FieldRenderer::for_primitive_date_time(),
        ReactiveValue::OffsetDateTime(_) => FieldRenderer::for_offset_date_time(),
        ReactiveValue::Duration(_) => FieldRenderer::for_duration(),
        ReactiveValue::OptionalPrimitiveDateTime(_) => {
            FieldRenderer::for_optional_primitive_date_time()
        }
        ReactiveValue::OptionalOffsetDateTime(_) => FieldRenderer::for_optional_offset_date_time(),
        ReactiveValue::OptionalDuration(_) => FieldRenderer::for_optional_duration(),
        ReactiveValue::Other(_) => FieldRenderer::for_other(),
    }
}
