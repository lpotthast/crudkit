//! Bindings between a rendered field and Leptonic's input atoms.
//!
//! Each hook reads the [`CrudFieldBinding`] of the field being rendered and returns the value, the
//! setter, and the flags for one Leptonic field atom, named after the atom's props. Input reported
//! through the setter updates the form's draft and the field's reactive value. Optional fields map
//! an empty input to `Value::Null`.

use crate::config::CrudText;
use crate::hooks::field::{CrudFieldBinding, use_crud_field_binding};
use crate::hooks::texts::{CrudTexts, use_crud_texts};
use crudkit_core::{TimeDuration, Value, ValueKind};
use crudkit_web::field::DateTimeDisplay;
use crudkit_web::field::FieldMode;
use crudkit_web::prelude::*;
use crudkit_web::value_format::{
    DurationParts, format_date_time, format_date_time_input, parse_date_time_input,
};
use leptonic::{Id, NumberValue};
use leptos::prelude::*;
use std::fmt;
use std::sync::Arc;
use time::format_description::well_known::Rfc3339;

/// Flags shared by all CrudKit inputs.
#[derive(Debug, Clone)]
pub struct CrudInputProps {
    /// DOM id for the input element.
    pub id: Id,
    /// The field's label: the one its layout gives, else its name.
    pub label: String,
    /// Whether the input is disabled by its layout options.
    pub is_disabled: Signal<bool>,
    /// Whether the input only presents its value: in display mode, and while the form saves.
    pub is_read_only: Signal<bool>,
    /// Whether a value is required.
    pub is_required: Signal<bool>,
    /// Whether the last input was rejected.
    pub is_invalid: Signal<bool>,
    /// Why the last input was rejected.
    pub error: Signal<Option<String>>,
}

fn input_props(field: &CrudFieldBinding) -> CrudInputProps {
    let is_disabled = field.options.disabled;
    let is_display_only = field.mode != FieldMode::Editable;
    let is_saving = field.is_saving;
    let is_required = !field.is_optional;
    let error = field.error;
    CrudInputProps {
        id: field.dom_id.clone(),
        label: field.label(),
        is_disabled: Signal::stored(is_disabled),
        // Read-only rather than disabled while saving, so the input keeps its focus.
        is_read_only: Signal::derive(move || is_display_only || is_saving.get()),
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

/// Parses text into a field's [`Value`], or returns why the text is invalid.
pub type CrudValueParser = Arc<dyn Fn(&str) -> Result<Value, CrudInputError> + Send + Sync>;

/// Why a [`CrudTextCodec`] rejected the text of an input. Inputs show it in the user's language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrudInputError {
    /// The text is not a UUID ([`CrudText::InputInvalidUuid`]).
    Uuid,
    /// The text is not a date-time in the form `YYYY-MM-DDTHH:MM[:SS]`
    /// ([`CrudText::InputInvalidDateTime`]).
    DateTime,
    /// The text is not a date-time with offset in RFC 3339 form
    /// ([`CrudText::InputInvalidOffsetDateTime`]).
    OffsetDateTime,
    /// The text is not a duration in the form `HH:MM:SS[.cc]`
    /// ([`CrudText::InputInvalidDuration`]).
    Duration,
    /// The text is not JSON ([`CrudText::InputInvalidJson`]).
    Json {
        /// The line of the first error, counted from one.
        line: u64,
        /// The column of the first error in characters, counted from one.
        column: u64,
    },
    /// The text ends before its JSON is complete ([`CrudText::InputIncompleteJson`]).
    JsonIncomplete,
    /// A message of the application, e.g. of its own codec, shown as it is.
    Message(String),
}

impl From<String> for CrudInputError {
    /// The application's `message`.
    fn from(message: String) -> Self {
        Self::Message(message)
    }
}

impl CrudInputError {
    /// Returns the message explaining this error in the language of `texts`, without tracking.
    #[must_use]
    pub fn message(&self, texts: CrudTexts) -> String {
        let text = match self {
            Self::Uuid => CrudText::InputInvalidUuid,
            Self::DateTime => CrudText::InputInvalidDateTime,
            Self::OffsetDateTime => CrudText::InputInvalidOffsetDateTime,
            Self::Duration => CrudText::InputInvalidDuration,
            Self::Json { line, column } => CrudText::InputInvalidJson {
                line: *line,
                column: *column,
            },
            Self::JsonIncomplete => CrudText::InputIncompleteJson,
            Self::Message(message) => return message.clone(),
        };
        texts.get_untracked(text)
    }
}

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
                    .map_err(|err| json_error(text, &err))
            }),
        }
    }

    /// Codec for `Value::Uuid` fields.
    #[must_use]
    pub fn uuid() -> Self {
        Self::parsed_or(Value::Uuid, Value::as_uuid, |_| CrudInputError::Uuid)
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
                    .map_err(|_| CrudInputError::DateTime)
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
                    .map_err(|_| CrudInputError::OffsetDateTime)
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
                    .map_err(|_| CrudInputError::Duration)
            }),
        }
    }

    /// Codec for values that format with `Display` and parse with `FromStr`. Text that does not
    /// parse shows the parse error's `Display` text ([`CrudInputError::Message`]), so choose a
    /// type whose errors are written in the user's language.
    pub fn parsed<T>(to_value: fn(T) -> Value, from_value: fn(&Value) -> Option<T>) -> Self
    where
        T: fmt::Display + std::str::FromStr + 'static,
        T::Err: fmt::Display,
    {
        Self::parsed_or(to_value, from_value, |err| {
            CrudInputError::Message(err.to_string())
        })
    }

    /// Codec for values that format with `Display` and parse with `FromStr`, e.g. a type of the
    /// application. Text that does not parse reports what `invalid` returns for the parse error,
    /// e.g. [`CrudInputError::Message`] in the user's language.
    pub fn parsed_or<T>(
        to_value: fn(T) -> Value,
        from_value: fn(&Value) -> Option<T>,
        invalid: fn(T::Err) -> CrudInputError,
    ) -> Self
    where
        T: fmt::Display + std::str::FromStr + 'static,
    {
        Self {
            format: Arc::new(move |value| {
                from_value(value)
                    .map(|it| it.to_string())
                    .unwrap_or_default()
            }),
            parse: Arc::new(move |text| text.trim().parse::<T>().map(to_value).map_err(invalid)),
        }
    }
}

/// Returns why `text` is not JSON, given the error of its parser. The parser counts columns in
/// bytes, so this counts the characters of the error's line up to that byte.
fn json_error(text: &str, err: &serde_json::Error) -> CrudInputError {
    if err.is_eof() {
        return CrudInputError::JsonIncomplete;
    }
    let line = err.line().max(1);
    let column = text.split('\n').nth(line - 1).map_or(err.column(), |line| {
        line.char_indices()
            .take_while(|(index, _)| *index < err.column())
            .count()
    });
    CrudInputError::Json {
        line: line as u64,
        column: column.max(1) as u64,
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
/// input error, in the user's language, and leaves the field's value unchanged.
///
/// # Panics
///
/// Panics when called outside of a field, see [`use_crud_field_binding`].
#[must_use]
pub fn use_crud_text_input(codec: CrudTextCodec) -> UseCrudTextInputReturn {
    let field = use_crud_field_binding();
    let props = input_props(&field);
    let is_optional = field.is_optional;
    let value = field.value;
    let set = field.set;
    let texts = use_crud_texts();
    let CrudTextCodec { format, parse } = codec;

    // Show external changes, e.g. a reloaded entity, but keep text the user is still typing: text
    // that parses to the current value, or that does not parse yet, stays untouched. Empty text
    // is the absent value of an optional field, and is read like other text in a required one,
    // so that an emptied required field keeps showing what makes it invalid.
    let text = RwSignal::new(format(&value.get_untracked()));
    let effect_parse = parse.clone();
    Effect::new(move |_| {
        let formatted = format(&value.get());
        let shown = text.get_untracked();
        let shown_formatted = if shown.trim().is_empty() && is_optional {
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
        set.run(result.map_err(|err| {
            Arc::new(InvalidInput(err.message(texts))) as Arc<dyn std::error::Error>
        }));
    });

    UseCrudTextInputReturn {
        value: text.into(),
        set_value,
        props,
    }
}

/// Formats `value` as text for display, e.g. in a table cell or a read-only view.
///
/// Numbers show their digits, booleans [`CrudText::ValueYes`] or [`CrudText::ValueNo`], date-times
/// without offset follow `date_time_display`, lists their formatted elements separated by commas,
/// and everything else the text its input shows. Absent values (`Null` and the unit value) are
/// empty.
#[must_use]
pub fn format_crud_value(
    value: &Value,
    date_time_display: DateTimeDisplay,
    texts: CrudTexts,
) -> String {
    match value {
        Value::Null | Value::Void(()) => String::new(),
        Value::Bool(true) => texts.get(CrudText::ValueYes),
        Value::Bool(false) => texts.get(CrudText::ValueNo),
        Value::U8(it) => it.to_string(),
        Value::U16(it) => it.to_string(),
        Value::U32(it) => it.to_string(),
        Value::U64(it) => it.to_string(),
        Value::U128(it) => it.to_string(),
        Value::I8(it) => it.to_string(),
        Value::I16(it) => it.to_string(),
        Value::I32(it) => it.to_string(),
        Value::I64(it) => it.to_string(),
        Value::I128(it) => it.to_string(),
        Value::F32(it) => it.to_string(),
        Value::F64(it) => it.to_string(),
        Value::String(it) => it.clone(),
        // A single line, unlike the pretty-printed JSON of its input.
        Value::Json(it) => it.to_string(),
        Value::Uuid(_) => (CrudTextCodec::uuid().format)(value),
        Value::PrimitiveDateTime(it) => format_date_time(*it, date_time_display),
        Value::OffsetDateTime(_) => (CrudTextCodec::offset_date_time().format)(value),
        Value::Duration(_) => (CrudTextCodec::duration().format)(value),
        Value::Array(values) => values
            .iter()
            .map(|value| format_crud_value(value, date_time_display, texts))
            .collect::<Vec<_>>()
            .join(", "),
        Value::Other(other) => format!("{other:?}"),
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
/// Panics when called outside of a field, see [`use_crud_field_binding`].
#[must_use]
pub fn use_crud_number_input<T: CrudNumber>() -> UseCrudNumberInputReturn<T> {
    let field = use_crud_field_binding();
    let props = input_props(&field);
    let is_optional = field.is_optional;
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
/// Panics when called outside of a field, see [`use_crud_field_binding`].
#[must_use]
pub fn use_crud_toggle_input() -> UseCrudToggleInputReturn {
    let field = use_crud_field_binding();
    let props = input_props(&field);
    let value = field.value;
    let set = field.set;
    UseCrudToggleInputReturn {
        is_selected: Signal::derive(move || value.get().as_bool().unwrap_or_default()),
        set_selected: Callback::new(move |selected: bool| set.run(Ok(Value::Bool(selected)))),
        props,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;

    #[test]
    fn json_errors_name_their_line_and_their_column_in_characters() {
        let parse = CrudTextCodec::json().parse;

        let error = parse("{\n  \"größe\": x\n}").err();

        assert_that!(error).is_equal_to(Some(CrudInputError::Json {
            line: 2,
            column: 12,
        }));
        assert_that!(
            error
                .map(|error| error.message(CrudTexts::default()))
                .unwrap_or_default()
        )
        .is_equal_to("Invalid JSON at line 2, column 12.".to_owned());
    }

    #[test]
    fn json_that_ends_early_is_incomplete() {
        let parse = CrudTextCodec::json().parse;

        assert_that!(parse("[1, 2").err()).is_equal_to(Some(CrudInputError::JsonIncomplete));
        assert_that!(parse("").err()).is_equal_to(Some(CrudInputError::JsonIncomplete));
    }

    #[test]
    fn messages_of_the_application_show_as_they_are() {
        let codec = CrudTextCodec::parsed_or(Value::Uuid, Value::as_uuid, |_| {
            "Gib eine Kennung ein.".to_owned().into()
        });

        let error = (codec.parse)("not an id").err();

        assert_that!(error.clone()).is_equal_to(Some(CrudInputError::Message(
            "Gib eine Kennung ein.".to_owned(),
        )));
        assert_that!(error.map(|error| error.message(CrudTexts::default())))
            .is_equal_to(Some("Gib eine Kennung ein.".to_owned()));
    }
}
