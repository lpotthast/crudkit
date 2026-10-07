//! Bindings between a rendered field and Leptonic's input atoms.
//!
//! Each hook reads the [`CrudFieldState`] of the field being rendered and returns the value, the
//! setter, and the flags for one Leptonic field atom, named after the atom's props. Input reported
//! through the setter updates the form's draft and the field's reactive value. Optional fields map an empty input to `Value::Null`.

use crate::hooks::field::{CrudFieldState, use_crud_field};
use crudkit_core::{TimeDuration, Value, ValueKind};
use crudkit_web::field::FieldMode;
use crudkit_web::prelude::*;
use crudkit_web::value_format::{DurationParts, format_date_time_input, parse_date_time_input};
use leptonic::utils::NumberValue;
use leptos::prelude::*;
use std::fmt;
use std::sync::Arc;
use time::format_description::well_known::Rfc3339;

/// Flags shared by all CrudKit inputs.
#[derive(Debug, Clone)]
pub struct CrudInputProps {
    /// DOM id for the input element.
    pub id: String,
    /// Label from the layout, if any.
    pub label: Option<String>,
    /// Whether the input is disabled by its layout options.
    pub is_disabled: Signal<bool>,
    /// Whether the input only presents its value.
    pub is_read_only: Signal<bool>,
    /// Whether a value is required.
    pub is_required: Signal<bool>,
    /// Whether the last input was rejected.
    pub is_invalid: Signal<bool>,
    /// Why the last input was rejected.
    pub error: Signal<Option<String>>,
}

fn input_props<F: TypeErasedField>(field: &CrudFieldState<F>) -> CrudInputProps {
    let is_disabled = field.options.disabled;
    let is_read_only = field.mode != FieldMode::Editable;
    let is_required = !field.is_optional();
    let error = field.error;
    CrudInputProps {
        id: field.dom_id.clone(),
        label: field.options.label.as_ref().map(|label| label.name.clone()),
        is_disabled: Signal::stored(is_disabled),
        is_read_only: Signal::stored(is_read_only),
        is_required: Signal::stored(is_required),
        is_invalid: Signal::derive(move || error.read().is_some()),
        error,
    }
}

#[derive(Debug)]
struct InvalidInput(String);

impl fmt::Display for InvalidInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for InvalidInput {}

/// Formats a field's [`Value`] as text.
pub type CrudValueFormatter = Arc<dyn Fn(&Value) -> String + Send + Sync>;

/// Parses text into a field's [`Value`], or returns a message describing why the text is invalid.
pub type CrudValueParser = Arc<dyn Fn(&str) -> Result<Value, String> + Send + Sync>;

/// Converts between a field's [`Value`] and the text shown in a text input.
#[derive(Clone)]
pub struct CrudTextCodec {
    /// Formats a value for display. `Value::Null` must format as an empty string.
    pub format: CrudValueFormatter,
    /// Parses non-empty user input.
    pub parse: CrudValueParser,
}

impl fmt::Debug for CrudTextCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrudTextCodec").finish_non_exhaustive()
    }
}

impl CrudTextCodec {
    /// Codec for `Value::String` fields.
    #[must_use]
    pub fn string() -> Self {
        Self {
            format: Arc::new(|value| value.as_string().cloned().unwrap_or_default()),
            parse: Arc::new(|text| Ok(Value::String(text.to_owned()))),
        }
    }

    /// Codec for `Value::Json` fields, pretty-printing JSON.
    #[must_use]
    pub fn json() -> Self {
        Self {
            format: Arc::new(|value| {
                value
                    .as_json()
                    .map(|json| serde_json::to_string_pretty(json).unwrap_or_default())
                    .unwrap_or_default()
            }),
            parse: Arc::new(|text| {
                serde_json::from_str(text)
                    .map(Value::Json)
                    .map_err(|err| err.to_string())
            }),
        }
    }

    /// Codec for `Value::Uuid` fields.
    #[must_use]
    pub fn uuid() -> Self {
        Self::parsed(Value::Uuid, Value::as_uuid)
    }

    /// Codec for `Value::PrimitiveDateTime` fields in the form `YYYY-MM-DDTHH:MM:SS`.
    #[must_use]
    pub fn primitive_date_time() -> Self {
        Self {
            format: Arc::new(|value| {
                value
                    .as_primitive_date_time()
                    .map(format_date_time_input)
                    .unwrap_or_default()
            }),
            parse: Arc::new(|text| {
                parse_date_time_input(text)
                    .map(Value::PrimitiveDateTime)
                    .map_err(|err| err.to_string())
            }),
        }
    }

    /// Codec for `Value::OffsetDateTime` fields in RFC 3339 form.
    #[must_use]
    pub fn offset_date_time() -> Self {
        Self {
            format: Arc::new(|value| {
                value
                    .as_offset_date_time()
                    .and_then(|it| it.format(&Rfc3339).ok())
                    .unwrap_or_default()
            }),
            parse: Arc::new(|text| {
                time::OffsetDateTime::parse(text.trim(), &Rfc3339)
                    .map(Value::OffsetDateTime)
                    .map_err(|err| err.to_string())
            }),
        }
    }

    /// Codec for `Value::Duration` fields in the form `HH:MM:SS.cc`.
    #[must_use]
    pub fn duration() -> Self {
        Self {
            format: Arc::new(|value| {
                value
                    .as_duration()
                    .map(|it| DurationParts::from_duration(it.0).to_string())
                    .unwrap_or_default()
            }),
            parse: Arc::new(|text| {
                text.parse::<DurationParts>()
                    .map(|parts| Value::Duration(TimeDuration(parts.to_duration())))
                    .map_err(|err| err.to_string())
            }),
        }
    }

    /// Codec for values that format with `Display` and parse with `FromStr`, e.g. UUIDs or
    /// 64-bit and 128-bit integers.
    pub fn parsed<T>(to_value: fn(T) -> Value, from_value: fn(&Value) -> Option<T>) -> Self
    where
        T: fmt::Display + std::str::FromStr + 'static,
        T::Err: fmt::Display,
    {
        Self {
            format: Arc::new(move |value| {
                from_value(value)
                    .map(|it| it.to_string())
                    .unwrap_or_default()
            }),
            parse: Arc::new(move |text| {
                text.trim()
                    .parse::<T>()
                    .map(to_value)
                    .map_err(|err| err.to_string())
            }),
        }
    }
}

/// Output of [`use_crud_text_input`].
///
/// Spread `value` and `set_value` onto Leptonic's `TextField`.
#[derive(Debug, Clone)]
pub struct UseCrudTextInputReturn {
    /// The shown text.
    pub value: Signal<String>,
    /// Receives text the user entered.
    pub set_value: Callback<String>,
    /// Flags of the input.
    pub props: CrudInputProps,
}

/// Binds the rendered field to a text input using `codec`.
///
/// Empty input becomes `Value::Null` for optional fields. Input the codec rejects is reported as an
/// input error and leaves the field's value unchanged.
///
/// # Panics
///
/// Panics when called outside of a field renderer for fields of type `F`.
#[must_use]
pub fn use_crud_text_input<F: TypeErasedField>(codec: CrudTextCodec) -> UseCrudTextInputReturn {
    let field = use_crud_field::<F>();
    let props = input_props(&field);
    let is_optional = field.is_optional();
    let value = field.value;
    let set = field.set;
    let CrudTextCodec { format, parse } = codec;

    // Show external changes, e.g. a reloaded entity, but keep text the user is still typing: text
    // that parses to the current value, or that does not parse yet, stays untouched.
    let text = RwSignal::new(format(&value.get_untracked()));
    let effect_parse = parse.clone();
    Effect::new(move |_| {
        let formatted = format(&value.get());
        let shown = text.get_untracked();
        let shown_formatted = if shown.trim().is_empty() {
            Some(String::new())
        } else {
            effect_parse(&shown).ok().map(|parsed| format(&parsed))
        };
        if shown_formatted.is_some_and(|shown_formatted| shown_formatted != formatted) {
            text.set(formatted);
        }
    });

    let set_value = Callback::new(move |input: String| {
        text.set(input.clone());
        let result = if input.trim().is_empty() && is_optional {
            Ok(Value::Null)
        } else {
            parse(&input)
        };
        set.run(result.map_err(|err| Arc::new(InvalidInput(err)) as Arc<dyn std::error::Error>));
    });

    UseCrudTextInputReturn {
        value: text.into(),
        set_value,
        props,
    }
}

/// The kind of input that edits values of a [`ValueKind`].
///
/// CrudKit's default field renderers are built from this mapping. Custom inputs should match on it
/// exhaustively, so that every value kind gets an input.
#[derive(Clone, Debug)]
pub enum CrudInputKind {
    /// Null and unit values, which have nothing to edit.
    Nothing,
    /// A checkbox or switch, see [`use_crud_toggle_input`].
    Toggle,
    /// A number field, see [`use_crud_number_input`].
    Number(CrudNumberKind),
    /// A text field using `codec`, see [`use_crud_text_input`].
    Text {
        /// Converts between field values and text.
        codec: CrudTextCodec,
        /// Whether the text spans multiple lines, e.g. pretty-printed JSON.
        multiline: bool,
    },
    /// Arrays and custom types, which need an application renderer.
    Custom,
}

impl CrudInputKind {
    /// Returns the input for values of `kind`.
    #[must_use]
    pub fn of(kind: ValueKind) -> Self {
        let text = |codec| Self::Text {
            codec,
            multiline: false,
        };
        match kind {
            ValueKind::Null | ValueKind::Void => Self::Nothing,
            ValueKind::Bool => Self::Toggle,
            ValueKind::U8 => Self::Number(CrudNumberKind::U8),
            ValueKind::U16 => Self::Number(CrudNumberKind::U16),
            ValueKind::U32 => Self::Number(CrudNumberKind::U32),
            ValueKind::U64 => Self::Number(CrudNumberKind::U64),
            ValueKind::U128 => Self::Number(CrudNumberKind::U128),
            ValueKind::I8 => Self::Number(CrudNumberKind::I8),
            ValueKind::I16 => Self::Number(CrudNumberKind::I16),
            ValueKind::I32 => Self::Number(CrudNumberKind::I32),
            ValueKind::I64 => Self::Number(CrudNumberKind::I64),
            ValueKind::I128 => Self::Number(CrudNumberKind::I128),
            ValueKind::F32 => Self::Number(CrudNumberKind::F32),
            ValueKind::F64 => Self::Number(CrudNumberKind::F64),
            ValueKind::String => text(CrudTextCodec::string()),
            ValueKind::Json => Self::Text {
                codec: CrudTextCodec::json(),
                multiline: true,
            },
            ValueKind::Uuid => text(CrudTextCodec::uuid()),
            ValueKind::PrimitiveDateTime => text(CrudTextCodec::primitive_date_time()),
            ValueKind::OffsetDateTime => text(CrudTextCodec::offset_date_time()),
            ValueKind::Duration => text(CrudTextCodec::duration()),
            ValueKind::Array | ValueKind::Other => Self::Custom,
        }
    }
}

/// The primitive type of a number field. Pass the matching type to [`use_crud_number_input`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrudNumberKind {
    /// An `u8` field.
    U8,
    /// An `u16` field.
    U16,
    /// An `u32` field.
    U32,
    /// An `u64` field.
    U64,
    /// An `u128` field.
    U128,
    /// An `i8` field.
    I8,
    /// An `i16` field.
    I16,
    /// An `i32` field.
    I32,
    /// An `i64` field.
    I64,
    /// An `i128` field.
    I128,
    /// An `f32` field.
    F32,
    /// An `f64` field.
    F64,
}

/// Numeric primitives supported by [`use_crud_number_input`].
pub trait CrudNumber: NumberValue + Copy + Send + Sync + 'static {
    /// Reads this number from a field value.
    fn from_value(value: &Value) -> Option<Self>;
    /// Wraps this number into a field value.
    fn into_value(self) -> Value;
}

macro_rules! impl_crud_number {
    ($($ty:ty => $variant:ident, $accessor:ident;)*) => {
        $(
            impl CrudNumber for $ty {
                fn from_value(value: &Value) -> Option<Self> {
                    value.$accessor()
                }

                fn into_value(self) -> Value {
                    Value::$variant(self)
                }
            }
        )*
    };
}

impl_crud_number! {
    u8 => U8, as_u8;
    u16 => U16, as_u16;
    u32 => U32, as_u32;
    u64 => U64, as_u64;
    u128 => U128, as_u128;
    i8 => I8, as_i8;
    i16 => I16, as_i16;
    i32 => I32, as_i32;
    i64 => I64, as_i64;
    i128 => I128, as_i128;
    f32 => F32, as_f32;
    f64 => F64, as_f64;
}

/// Output of [`use_crud_number_input`].
///
/// Spread `value` and `set_value` onto Leptonic's `NumberField`.
#[derive(Debug, Clone)]
pub struct UseCrudNumberInputReturn<T: CrudNumber> {
    /// The shown number, `None` while the field is empty.
    pub value: Signal<Option<T>>,
    /// Receives numbers the user committed.
    pub set_value: Callback<Option<T>>,
    /// Flags of the input.
    pub props: CrudInputProps,
}

/// Binds the rendered field to a number input of type `T`.
///
/// An empty input becomes `Value::Null` for optional fields and leaves required fields unchanged.
///
/// # Panics
///
/// Panics when called outside of a field renderer for fields of type `F`.
#[must_use]
pub fn use_crud_number_input<F: TypeErasedField, T: CrudNumber>() -> UseCrudNumberInputReturn<T> {
    let field = use_crud_field::<F>();
    let props = input_props(&field);
    let is_optional = field.is_optional();
    let value = field.value;
    let set = field.set;
    UseCrudNumberInputReturn {
        value: Signal::derive(move || T::from_value(&value.get())),
        set_value: Callback::new(move |number: Option<T>| match number {
            Some(number) => set.run(Ok(number.into_value())),
            None if is_optional => set.run(Ok(Value::Null)),
            None => {}
        }),
        props,
    }
}

/// Output of [`use_crud_toggle_input`].
///
/// Spread `is_selected` and `set_selected` onto Leptonic's `Checkbox` or `Switch`.
#[derive(Debug, Clone)]
pub struct UseCrudToggleInputReturn {
    /// Whether the toggle is selected.
    pub is_selected: Signal<bool>,
    /// Receives the selection the user chose.
    pub set_selected: Callback<bool>,
    /// Flags of the input.
    pub props: CrudInputProps,
}

/// Binds the rendered `bool` field to a checkbox or switch. `Value::Null` shows as unselected.
///
/// # Panics
///
/// Panics when called outside of a field renderer for fields of type `F`.
#[must_use]
pub fn use_crud_toggle_input<F: TypeErasedField>() -> UseCrudToggleInputReturn {
    let field = use_crud_field::<F>();
    let props = input_props(&field);
    let value = field.value;
    let set = field.set;
    UseCrudToggleInputReturn {
        is_selected: Signal::derive(move || value.get().as_bool().unwrap_or_default()),
        set_selected: Callback::new(move |selected: bool| set.run(Ok(Value::Bool(selected)))),
        props,
    }
}
