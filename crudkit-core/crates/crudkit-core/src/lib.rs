#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Debug;
use time::format_description::well_known::Rfc3339;
use utoipa::openapi::Type;
use utoipa::ToSchema;

#[derive(Default, PartialEq, Eq, Hash, Clone, Copy, Debug, ToSchema, Serialize, Deserialize)]
pub enum Order {
    #[default]
    #[serde(rename(
        serialize = "asc",
        deserialize = "asc",
        deserialize = "ascending",
        deserialize = "Asc"
    ))]
    Asc,
    #[serde(rename(
        serialize = "desc",
        deserialize = "desc",
        deserialize = "descending",
        deserialize = "Desc"
    ))]
    Desc,
}

/// A duration of time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeDuration(pub time::Duration);

/// Serialize as microseconds (i64)
impl Serialize for TimeDuration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(self.0.whole_microseconds() as i64)
    }
}

/// Deserialize from microseconds (i64)
impl<'de> Deserialize<'de> for TimeDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let microseconds = i64::deserialize(deserializer)?;
        Ok(TimeDuration(time::Duration::microseconds(microseconds)))
    }
}

impl utoipa::ToSchema for TimeDuration {
    fn name() -> Cow<'static, str> {
        Cow::Borrowed("TimeDuration")
    }
}

impl utoipa::PartialSchema for TimeDuration {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::SchemaType::Type(Type::Integer))
            .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::Int64,
            )))
            .description(Some("Duration in microseconds"))
            .into()
    }
}

#[typetag::serde]
pub trait FieldValue: Debug + DynClone + Send + Sync + 'static {}
dyn_clone::clone_trait_object!(FieldValue);

#[typetag::serde]
impl FieldValue for bool {}

#[typetag::serde]
impl FieldValue for u8 {}

#[typetag::serde]
impl FieldValue for u16 {}

#[typetag::serde]
impl FieldValue for u32 {}

#[typetag::serde]
impl FieldValue for u64 {}

#[typetag::serde]
impl FieldValue for u128 {}

#[typetag::serde]
impl FieldValue for i8 {}

#[typetag::serde]
impl FieldValue for i16 {}

#[typetag::serde]
impl FieldValue for i32 {}

#[typetag::serde]
impl FieldValue for i64 {}

#[typetag::serde]
impl FieldValue for i128 {}

#[typetag::serde]

impl FieldValue for f32 {}

#[typetag::serde]
impl FieldValue for f64 {}

#[typetag::serde]
impl FieldValue for String {}

#[typetag::serde]
impl FieldValue for serde_json::Value {}

#[typetag::serde]
impl FieldValue for time::PrimitiveDateTime {}

#[typetag::serde]
impl FieldValue for time::OffsetDateTime {}

#[typetag::serde]
impl FieldValue for TimeDuration {}

/// Values which can be used by crud fields.
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

    U8Vec(Vec<u8>),
    I32Vec(Vec<i32>),
    I64Vec(Vec<i64>),

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

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct Saved<T> {
    pub entity: T,
    pub with_validation_errors: bool,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub enum SaveResult<T> {
    Saved(Saved<T>),
    Aborted { reason: String },
    CriticalValidationErrors,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub enum DeleteResult {
    Deleted(u64),
    Aborted { reason: String },
    CriticalValidationErrors,
}
