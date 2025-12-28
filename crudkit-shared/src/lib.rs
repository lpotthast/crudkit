#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use std::borrow::Cow;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use utoipa::openapi::Type;
use utoipa::{PartialSchema, ToSchema};

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

/// Values which can be used by crud fields.
#[derive(Debug, Clone, PartialEq, ToSchema, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Json(serde_json::Value),
    Uuid(uuid::Uuid),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    U8Vec(Vec<u8>),
    I32Vec(Vec<i32>),
    I64Vec(Vec<i64>),
    F32(f32),
    F64(f64),
    Bool(bool),
    PrimitiveDateTime(time::PrimitiveDateTime),
    OffsetDateTime(time::OffsetDateTime),
    Duration(TimeDuration),
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
