//! CrudKit's built-in field renderers, rendering `crudkit-*` markup on Leptonic's field atoms.

use crate::components::inputs::{
    crud_number_input, crud_text_input, crud_toggle_input, crud_validation_status,
};
use crate::config::FieldRenderer;
use crate::hooks::field::CrudFieldState;
use crate::hooks::inputs::{CrudInputKind, CrudNumberKind, CrudTextCodec, CrudValueFormatter};
use crudkit_core::{Value, ValueKind};
use crudkit_web::prelude::*;
use crudkit_web::value_format::format_date_time;
use leptos::prelude::*;
use std::sync::Arc;

impl<F: TypeErasedField> FieldRenderer<F> {
    /// Returns CrudKit's default renderer for fields of `kind`, following [`CrudInputKind::of`].
    #[must_use]
    pub fn default_for(kind: ValueKind) -> Self {
        match CrudInputKind::of(kind) {
            CrudInputKind::Nothing => Self::for_void(),
            CrudInputKind::Toggle => Self::for_bool(),
            CrudInputKind::Number(number) => match number {
                CrudNumberKind::U8 => Self::for_number::<u8>(),
                CrudNumberKind::U16 => Self::for_number::<u16>(),
                CrudNumberKind::U32 => Self::for_number::<u32>(),
                CrudNumberKind::U64 => Self::for_number::<u64>(),
                CrudNumberKind::U128 => Self::for_number::<u128>(),
                CrudNumberKind::I8 => Self::for_number::<i8>(),
                CrudNumberKind::I16 => Self::for_number::<i16>(),
                CrudNumberKind::I32 => Self::for_number::<i32>(),
                CrudNumberKind::I64 => Self::for_number::<i64>(),
                CrudNumberKind::I128 => Self::for_number::<i128>(),
                CrudNumberKind::F32 => Self::for_number::<f32>(),
                CrudNumberKind::F64 => Self::for_number::<f64>(),
            },
            // Date-times display in the layout's format.
            CrudInputKind::Text { .. } if kind == ValueKind::PrimitiveDateTime => {
                Self::for_primitive_date_time()
            }
            CrudInputKind::Text { codec, multiline } => {
                Self::new(move |_| crud_text_input::<F>(codec.clone(), multiline, None))
            }
            CrudInputKind::Custom if kind == ValueKind::Array => Self::for_array(),
            CrudInputKind::Custom => Self::for_other(),
        }
    }

    /// Renders a `bool` field meaning "validation errors exist" as a warning or check icon.
    ///
    /// ```ignore
    /// read_field_renderer: FieldRendererRegistry::builder()
    ///     .register(ReadUser::HasValidationErrors, FieldRenderer::for_validation_status())
    ///     .build(),
    /// ```
    #[must_use]
    pub fn for_validation_status() -> Self {
        Self::new(|_| crud_validation_status::<F>())
    }

    /// Renders nothing.
    #[must_use]
    pub fn for_void() -> Self {
        Self::new(|_| ())
    }

    /// Renders a checkbox.
    #[must_use]
    pub fn for_bool() -> Self {
        Self::new(|_| crud_toggle_input::<F>())
    }

    /// Renders a number field for numbers of type `T`.
    #[must_use]
    pub fn for_number<T: crate::hooks::inputs::CrudNumber + std::fmt::Display>() -> Self {
        Self::new(|_| crud_number_input::<F, T>())
    }

    /// Renders a text field using `codec`.
    #[must_use]
    pub fn for_text(codec: CrudTextCodec) -> Self {
        Self::new(move |_| crud_text_input::<F>(codec.clone(), false, None))
    }

    /// Renders a single-line text field.
    #[must_use]
    pub fn for_string() -> Self {
        Self::for_text(CrudTextCodec::string())
    }

    /// Renders a text area with pretty-printed JSON.
    #[must_use]
    pub fn for_json() -> Self {
        let codec = CrudTextCodec::json();
        Self::new(move |_| crud_text_input::<F>(codec.clone(), true, None))
    }

    /// Renders a text field for a UUID.
    #[must_use]
    pub fn for_uuid() -> Self {
        Self::for_text(CrudTextCodec::uuid())
    }

    /// Renders a `YYYY-MM-DDTHH:MM:SS` text field for a date-time without offset. Display mode uses
    /// the layout's `date_time_display`.
    #[must_use]
    pub fn for_primitive_date_time() -> Self {
        let codec = CrudTextCodec::primitive_date_time();
        Self::new(move |state: CrudFieldState<F>| {
            let date_time_display = state.options.date_time_display;
            let display: CrudValueFormatter = Arc::new(move |value: &Value| {
                value
                    .as_primitive_date_time()
                    .map(|it| format_date_time(it, date_time_display))
                    .unwrap_or_default()
            });
            crud_text_input::<F>(codec.clone(), false, Some(display))
        })
    }

    /// Renders an RFC 3339 text field for a date-time with offset.
    #[must_use]
    pub fn for_offset_date_time() -> Self {
        Self::for_text(CrudTextCodec::offset_date_time())
    }

    /// Renders a `HH:MM:SS.cc` text field for a duration.
    #[must_use]
    pub fn for_duration() -> Self {
        Self::for_text(CrudTextCodec::duration())
    }

    /// Renders a placeholder; arrays need an application renderer.
    // TODO: Render arrays.
    #[must_use]
    pub fn for_array() -> Self {
        Self::new(|_| view! { <span class="crudkit-value">"TODO: Render Array"</span> })
    }

    /// Renders a visible configuration error: custom types need an application renderer.
    #[must_use]
    pub fn for_other() -> Self {
        Self::new(|state: CrudFieldState<F>| {
            let name = state.field.name().to_string();
            view! {
                <div class="crudkit-field-error" role="alert">
                    {format!(
                        "The field '{name}' has a custom type that needs a field renderer registered in the instance configuration.",
                    )}
                </div>
            }
        })
    }
}
