use crate::crud_field_label::CrudFieldLabelOpt;
use crate::fields::boolean::CrudBoolField;
use crate::fields::date_time::CrudPrimitiveDateTimeField;
use crate::fields::duration::CrudDurationField;
use crate::fields::json::CrudJsonField;
use crate::fields::number::{
    CrudF32Field, CrudF64Field, CrudI128Field, CrudI16Field, CrudI32Field, CrudI64Field,
    CrudI8Field, CrudU128Field, CrudU16Field, CrudU32Field, CrudU64Field, CrudU8Field,
};
use crate::fields::string::CrudStringField;
use crate::fields::uuid::CrudUuidField;
use crate::fields::validation_status::CrudValidationStatusField;
use crate::ReactiveField;
use crudkit_core::{Value, ValueKind};
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
        StoredValue<HashMap<F, ReactiveField>>,              // signals
        F,                                                   // field (for metadata access)
        FieldMode,                                           // field_mode
        FieldOptions,                                        // field_options
        ReactiveField,                                       // value
        Callback<Result<Value, Arc<dyn std::error::Error>>>, // value_changed
    )>,
}

impl<F: TypeErasedField> FieldRenderer<F> {
    pub fn new<C: RenderHtml>(
        view_fn: impl Fn(
            StoredValue<HashMap<F, ReactiveField>>,
            F,
            FieldMode,
            FieldOptions,
            ReactiveField,
            Callback<Result<Value, Arc<dyn std::error::Error>>>,
        ) -> C
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            view_cb: ViewCallback::new(Callback::from(move |a1, a2, a3, a4, a5, a6| {
                view! {
                    <div class="crud-field">
                        { view_fn(a1, a2, a3, a4, a5, a6).into_any() }
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
            move |_signals, _field, field_mode, field_options, value, _value_changed| {
                view! {
                    <CrudValidationStatusField
                        id=format!("f{}", Uuid::new_v4())
                        field_options
                        field_mode
                        value=Signal::derive(move || value.value.get().expect_bool())
                    />
                }
            },
        )
    }

    pub fn for_void() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, _field, _field_mode, _field_options, _value, _value_changed| {
                view! {}
            },
        )
    }

    pub fn for_bool() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().as_bool());
                view! {
                    <CrudBoolField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_u8() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().as_u8());
                view! {
                    <CrudU8Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_u16() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().as_u16());
                view! {
                    <CrudU16Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_u32() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().as_u32());
                view! {
                    <CrudU32Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_u64() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().as_u64());
                view! {
                    <CrudU64Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_u128() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().as_u128());
                view! {
                    <CrudU128Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_i8() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().as_i8());
                view! {
                    <CrudI8Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_i16() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().as_i16());
                view! {
                    <CrudI16Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_i32() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().as_i32());
                view! {
                    <CrudI32Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_i64() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().as_i64());
                view! {
                    <CrudI64Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_i128() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().as_i128());
                view! {
                    <CrudI128Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_f32() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().as_f32());
                view! {
                    <CrudF32Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_f64() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().as_f64());
                view! {
                    <CrudF64Field
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_string() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().take_string());
                view! {
                    <CrudStringField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_json() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().as_json().cloned());
                view! {
                    <CrudJsonField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_uuid() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().as_uuid());
                view! {
                    <CrudUuidField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_primitive_date_time() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value =
                    Signal::derive(move || value.value.get().as_primitive_date_time());
                view! {
                    <CrudPrimitiveDateTimeField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_offset_date_time() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, _field, _field_mode, _field_options, _value, _value_changed| {
                view! { "TODO: Render OffsetDateTime" }
            },
        )
    }

    pub fn for_duration() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, field: F, field_mode, field_options, value, value_changed| {
                let typed_value = Signal::derive(move || value.value.get().as_duration().cloned());
                view! {
                    <CrudDurationField
                        id=format!("f{}", Uuid::new_v4())
                        field_options=field_options
                        field_mode=field_mode
                        value=typed_value
                        is_optional=field.is_optional()
                        value_changed=value_changed
                    />
                }
            },
        )
    }

    pub fn for_array() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, _field, _field_mode, _field_options, _value, _value_changed| {
                view! { "TODO: Render Array" }
            },
        )
    }

    pub fn for_other() -> FieldRenderer<F> {
        FieldRenderer::new(
            move |_signals, _field, _field_mode, field_options, value, _value_changed| {
                view! {
                    <Alert variant=AlertVariant::Danger>
                        <AlertTitle slot>"Invalid Configuration"</AlertTitle>
                        <AlertContent slot>
                            <p>
                                "The field labeled '"
                                {format!("{:?}", field_options.label)}
                                "' has a custom type that can only be rendered with a user-specified field renderer. But no renderer was registered for that field in the instance config. You might have forgotten to set the required HashMap entry."
                            </p>
                            <p>"The current value of the field is:"</p>
                            <pre>
                                {move || format!("{:?}", value.get())}
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

/// Returns the default field renderer for a given `ValueKind`.
pub(crate) fn default_field_renderer<F: TypeErasedField>(kind: ValueKind) -> FieldRenderer<F> {
    match kind {
        ValueKind::Null | ValueKind::Void => FieldRenderer::for_void(),
        ValueKind::Bool => FieldRenderer::for_bool(),
        ValueKind::U8 => FieldRenderer::for_u8(),
        ValueKind::U16 => FieldRenderer::for_u16(),
        ValueKind::U32 => FieldRenderer::for_u32(),
        ValueKind::U64 => FieldRenderer::for_u64(),
        ValueKind::U128 => FieldRenderer::for_u128(),
        ValueKind::I8 => FieldRenderer::for_i8(),
        ValueKind::I16 => FieldRenderer::for_i16(),
        ValueKind::I32 => FieldRenderer::for_i32(),
        ValueKind::I64 => FieldRenderer::for_i64(),
        ValueKind::I128 => FieldRenderer::for_i128(),
        ValueKind::F32 => FieldRenderer::for_f32(),
        ValueKind::F64 => FieldRenderer::for_f64(),
        ValueKind::String => FieldRenderer::for_string(),
        ValueKind::Json => FieldRenderer::for_json(),
        ValueKind::Uuid => FieldRenderer::for_uuid(),
        ValueKind::PrimitiveDateTime => FieldRenderer::for_primitive_date_time(),
        ValueKind::OffsetDateTime => FieldRenderer::for_offset_date_time(),
        ValueKind::Duration => FieldRenderer::for_duration(),
        ValueKind::Array => FieldRenderer::for_array(),
        ValueKind::Other => FieldRenderer::for_other(),
    }
}
