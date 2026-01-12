use crudkit_condition::ConditionClauseValue;
use crudkit_id::IdValue;
use crudkit_shared::TimeDuration;
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use tracing::warn;

/// Any field value must be:
///
/// - `Display`: This is our fallback for field rendering should no
/// specialized rendering have been provided.
#[typetag::serde]
pub trait FieldValue: Debug + DynClone /*+ DynEq*/ + Send + Sync + 'static {
    // TODO: Rendering must support localization! See time::PrimitiveDateTime impl.
    /// Renders the value. The default implementation uses `Debug`.
    /// Types implementing `Display` should override to use it instead.
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
dyn_clone::clone_trait_object!(FieldValue);
//dyn_eq::eq_trait_object!(FieldValue);

#[typetag::serde]
impl FieldValue for bool {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[typetag::serde]
impl FieldValue for u8 {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[typetag::serde]
impl FieldValue for u16 {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[typetag::serde]
impl FieldValue for u32 {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[typetag::serde]
impl FieldValue for u64 {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[typetag::serde]
impl FieldValue for u128 {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[typetag::serde]
impl FieldValue for i8 {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[typetag::serde]
impl FieldValue for i16 {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[typetag::serde]
impl FieldValue for i32 {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[typetag::serde]
impl FieldValue for i64 {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[typetag::serde]
impl FieldValue for i128 {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[typetag::serde]

impl FieldValue for f32 {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[typetag::serde]
impl FieldValue for f64 {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[typetag::serde]
impl FieldValue for String {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[typetag::serde]
impl FieldValue for serde_json::Value {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[typetag::serde]
impl FieldValue for time::PrimitiveDateTime {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &self
                .format(&Rfc3339)
                .expect("infallible using well-known format"),
        )
    }
}

#[typetag::serde]
impl FieldValue for time::OffsetDateTime {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &self
                .format(&Rfc3339)
                .expect("infallible using well-known format"),
        )
    }
}

#[typetag::serde]
impl FieldValue for TimeDuration {
    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: Correct formatting? Should we do a manual: [h]:[m]:[s]?
        f.write_fmt(format_args!("{}", &self.0))
    }
}

// TODO: The serialize/deserialize impls should be compatible with Option!
#[derive(Debug, Serialize, Deserialize)]
enum MaybeFieldValue<V: FieldValue> {
    None,
    Some(V),
}

impl<V: FieldValue> Display for MaybeFieldValue<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MaybeFieldValue::None => f.write_str("-"),
            MaybeFieldValue::Some(v) => v.render(f),
        }
    }
}

//#[typetag::serde]
//impl<V: FieldValue> FieldValue for MaybeFieldValue<V> {
//    fn render(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//        write!(f, "{self}")
//    }
//}

/// The value of a field.
/// All variants should be stateless / copy-replaceable.
// TODO: DEFERRED: Implement Serialize and Deserialize with typetag when wasm is supported in typetag. Comment in "typetag" occurrences.
#[derive(Debug, Clone)]
pub enum Value {
    Void(()),

    Bool(bool),
    OptionalBool(Option<bool>),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    OptionalU8(Option<u8>),
    OptionalU16(Option<u16>),
    OptionalU32(Option<u32>),
    OptionalU64(Option<u64>),
    OptionalU128(Option<u128>),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    OptionalI8(Option<i8>),
    OptionalI16(Option<i16>),
    OptionalI32(Option<i32>),
    OptionalI64(Option<i64>),
    OptionalI128(Option<i128>),

    F32(f32),
    F64(f64),
    OptionalF32(Option<f32>),
    OptionalF64(Option<f64>),

    String(String),
    OptionalString(Option<String>),

    // Ecosystem support.
    // -- serde
    Json(serde_json::Value),
    OptionalJson(Option<serde_json::Value>),

    // -- uuid
    Uuid(uuid::Uuid),
    OptionalUuid(Option<uuid::Uuid>),

    // -- time
    PrimitiveDateTime(time::PrimitiveDateTime),
    OffsetDateTime(time::OffsetDateTime),
    Duration(TimeDuration),
    OptionalPrimitiveDateTime(Option<time::PrimitiveDateTime>),
    OptionalOffsetDateTime(Option<time::OffsetDateTime>),
    OptionalDuration(Option<TimeDuration>),

    // Extension support.
    Other(Box<dyn FieldValue>),
}

impl From<crudkit_shared::Value> for Value {
    fn from(value: crudkit_shared::Value) -> Self {
        match value {
            crudkit_shared::Value::String(value) => Value::String(value),
            crudkit_shared::Value::Json(value) => Value::Json(value),
            crudkit_shared::Value::Uuid(value) => Value::Uuid(value),
            crudkit_shared::Value::I32(value) => Value::I32(value),
            crudkit_shared::Value::I64(value) => Value::I64(value),
            crudkit_shared::Value::U8Vec(_values) => todo!("support vector types"),
            crudkit_shared::Value::I32Vec(_values) => todo!("support vector types"),
            crudkit_shared::Value::I64Vec(_values) => todo!("support vector types"),
            crudkit_shared::Value::U32(value) => Value::U32(value),
            crudkit_shared::Value::U64(value) => Value::U64(value),
            crudkit_shared::Value::F32(value) => Value::F32(value),
            crudkit_shared::Value::F64(value) => Value::F64(value),
            crudkit_shared::Value::Bool(value) => Value::Bool(value),
            crudkit_shared::Value::PrimitiveDateTime(value) => Value::PrimitiveDateTime(value),
            crudkit_shared::Value::OffsetDateTime(value) => Value::OffsetDateTime(value),
            crudkit_shared::Value::Duration(value) => Value::Duration(value),
        }
    }
}

impl From<IdValue> for Value {
    fn from(id_value: IdValue) -> Self {
        match id_value {
            IdValue::String(value) => Value::String(value), // TODO: How can we differentiate between String and Text?
            IdValue::Uuid(value) => Value::Uuid(value),
            IdValue::I32(value) => Value::I32(value),
            IdValue::U32(value) => Value::U32(value),
            IdValue::I64(value) => Value::I64(value),
            IdValue::U64(value) => Value::U64(value),
            IdValue::I128(value) => Value::I128(value),
            IdValue::U128(value) => Value::U128(value),
            IdValue::Bool(value) => Value::Bool(value),
            IdValue::PrimitiveDateTime(value) => Value::PrimitiveDateTime(value),
            IdValue::OffsetDateTime(value) => Value::OffsetDateTime(value),
        }
    }
}

impl Value {
    pub fn take_bool(self) -> bool {
        match self {
            Self::Bool(bool) => bool,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_bool(self) -> Option<bool> {
        match self {
            Self::OptionalBool(bool) => bool,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_u8(self) -> u8 {
        match self {
            Self::U8(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_u16(self) -> u16 {
        match self {
            Self::U16(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_u32(self) -> u32 {
        match self {
            Self::U32(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_u64(self) -> u64 {
        match self {
            Self::U64(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_u128(self) -> u128 {
        match self {
            Self::U128(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_u8(self) -> Option<u8> {
        match self {
            Self::U8(value) => Some(value),
            Self::OptionalU8(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_u16(self) -> Option<u16> {
        match self {
            Self::U16(value) => Some(value),
            Self::OptionalU16(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_u32(self) -> Option<u32> {
        match self {
            Self::U32(value) => Some(value),
            Self::OptionalU32(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_u64(self) -> Option<u64> {
        match self {
            Self::U64(value) => Some(value),
            Self::OptionalU64(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_u128(self) -> Option<u128> {
        match self {
            Self::U128(value) => Some(value),
            Self::OptionalU128(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_i8(self) -> i8 {
        match self {
            Self::I8(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_i16(self) -> i16 {
        match self {
            Self::I16(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_i32(self) -> i32 {
        match self {
            Self::I32(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_i64(self) -> i64 {
        match self {
            Self::I64(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_i128(self) -> i128 {
        match self {
            Self::I128(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_i8(self) -> Option<i8> {
        match self {
            Self::I8(value) => Some(value),
            Self::OptionalI8(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_i16(self) -> Option<i16> {
        match self {
            Self::I16(value) => Some(value),
            Self::OptionalI16(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_i32(self) -> Option<i32> {
        match self {
            Self::I32(value) => Some(value),
            Self::OptionalI32(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_i64(self) -> Option<i64> {
        match self {
            Self::I64(value) => Some(value),
            Self::OptionalI64(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_i128(self) -> Option<i128> {
        match self {
            Self::I128(value) => Some(value),
            Self::OptionalI128(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_f32(self) -> f32 {
        match self {
            Self::F32(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_f64(self) -> f64 {
        match self {
            Self::F64(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_f32(self) -> Option<f32> {
        match self {
            Self::F32(value) => Some(value),
            Self::OptionalF32(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_f64(self) -> Option<f64> {
        match self {
            Self::F64(value) => Some(value),
            Self::OptionalF64(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_string(self) -> String {
        match self {
            Self::String(string) => string,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_string(self) -> Option<String> {
        match self {
            Self::OptionalString(string) => string,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_json_value(self) -> serde_json::Value {
        match self {
            Self::Json(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_json_value(self) -> Option<serde_json::Value> {
        match self {
            Self::OptionalJson(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_uuid(self) -> uuid::Uuid {
        match self {
            Self::Uuid(uuid) => uuid,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_uuid(self) -> Option<uuid::Uuid> {
        match self {
            Self::OptionalUuid(uuid) => uuid,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_primitive_date_time(self) -> time::PrimitiveDateTime {
        match self {
            Self::PrimitiveDateTime(primitive_date_time) => primitive_date_time,
            Self::String(string) => time::PrimitiveDateTime::parse(&string, &Rfc3339).unwrap(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_offset_date_time(self) -> time::OffsetDateTime {
        match self {
            Self::OffsetDateTime(offset_date_time) => offset_date_time,
            Self::String(string) => time::OffsetDateTime::parse(&string, &Rfc3339).unwrap(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_duration(self) -> TimeDuration {
        match self {
            Self::Duration(duration) => duration,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_primitive_date_time(self) -> Option<time::PrimitiveDateTime> {
        match self {
            Self::PrimitiveDateTime(primitive_date_time) => Some(primitive_date_time),
            Self::OptionalPrimitiveDateTime(optional_primitive_date_time) => {
                optional_primitive_date_time
            }
            // TODO: We might want to catch parsing errors and return an empty optional here.
            Self::String(string) => {
                Some(time::PrimitiveDateTime::parse(&string, &Rfc3339).unwrap())
            }
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_offset_date_time(self) -> Option<time::OffsetDateTime> {
        match self {
            Self::OffsetDateTime(offset_date_time) => Some(offset_date_time),
            Self::OptionalOffsetDateTime(optional_offset_date_time) => optional_offset_date_time,
            // TODO: We might want to catch parsing errors and return an empty optional here.
            Self::String(string) => Some(time::OffsetDateTime::parse(&string, &Rfc3339).unwrap()),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_duration(self) -> Option<TimeDuration> {
        match self {
            Self::Duration(duration) => Some(duration),
            Self::OptionalDuration(optional_duration) => optional_duration,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_other(self) -> Box<dyn FieldValue> {
        match self {
            Value::Other(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
}

#[derive(Debug, Error)]
#[error("The value '{value:?}' cannot be used in a condition clause.")]
pub struct NotConditionClauseCompatible {
    value: Value,
}

impl TryInto<ConditionClauseValue> for Value {
    type Error = NotConditionClauseCompatible;

    fn try_into(self) -> Result<ConditionClauseValue, Self::Error> {
        match self {
            value @ Value::Void(()) => Err(NotConditionClauseCompatible { value }),

            Value::Bool(value) => Ok(ConditionClauseValue::Bool(value)),
            value @ Value::OptionalBool(_) => Err(NotConditionClauseCompatible { value }),

            Value::I8(value) => Ok(ConditionClauseValue::I8(value)),
            Value::I16(value) => Ok(ConditionClauseValue::I16(value)),
            Value::I32(value) => Ok(ConditionClauseValue::I32(value)),
            Value::I64(value) => Ok(ConditionClauseValue::I64(value)),
            Value::I128(value) => Ok(ConditionClauseValue::I128(value)),
            value @ Value::OptionalI8(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalI16(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalI32(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalI64(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalI128(_) => Err(NotConditionClauseCompatible { value }),

            Value::U8(value) => Ok(ConditionClauseValue::U8(value)),
            Value::U16(value) => Ok(ConditionClauseValue::U16(value)),
            Value::U32(value) => Ok(ConditionClauseValue::U32(value)),
            Value::U64(value) => Ok(ConditionClauseValue::U64(value)),
            Value::U128(value) => Ok(ConditionClauseValue::U128(value)),
            value @ Value::OptionalU8(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalU16(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalU32(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalU64(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalU128(_) => Err(NotConditionClauseCompatible { value }),

            Value::F32(value) => Ok(ConditionClauseValue::F32(value)),
            Value::F64(value) => Ok(ConditionClauseValue::F64(value)),
            value @ Value::OptionalF32(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalF64(_) => Err(NotConditionClauseCompatible { value }),

            Value::String(value) => Ok(ConditionClauseValue::String(value)),
            value @ Value::OptionalString(_) => Err(NotConditionClauseCompatible { value }),

            // Ecosystem support.
            Value::Json(value) => Ok(ConditionClauseValue::Json(
                serde_json::to_string(&value).unwrap(),
            )),
            value @ Value::OptionalJson(_) => Err(NotConditionClauseCompatible { value }),
            Value::Uuid(value) => Ok(ConditionClauseValue::Uuid(value)),
            value @ Value::OptionalUuid(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::PrimitiveDateTime(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OffsetDateTime(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalPrimitiveDateTime(_) => {
                Err(NotConditionClauseCompatible { value })
            }
            value @ Value::OptionalOffsetDateTime(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::Duration(_) => Err(NotConditionClauseCompatible { value }),
            value @ Value::OptionalDuration(_) => Err(NotConditionClauseCompatible { value }),

            value @ Value::Other(_) => Err(NotConditionClauseCompatible { value }),
        }
    }
}
