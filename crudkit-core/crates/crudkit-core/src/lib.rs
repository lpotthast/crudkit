#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Debug;
use time::format_description::well_known::Rfc3339;
use utoipa::openapi::Type;
use utoipa::ToSchema;

/// A trait for types that have a name.
pub trait Named {
    /// Returns the name of this item.
    fn name(&self) -> Cow<'static, str>;
}

/// Base trait for data models in the CRUD framework.
///
/// This is the minimal shared definition used by both backend (crudkit-rs)
/// and frontend (crudkit-web) layers. Each layer extends this with additional
/// bounds and methods appropriate for their context:
///
/// - **Backend (`crudkit-rs`)**: Uses this directly with `Field: crudkit_rs::Field`
/// - **Frontend (`crudkit-web`)**: Adds `Serialize`, `DeserializeOwned`, `PartialEq`,
///   field accessor methods, and more extensive `Field` bounds
pub trait Model: Clone + Debug + Send + Sync + 'static {
    /// The field enum type for this model.
    ///
    /// Each model has an associated field enum that provides typed access
    /// to individual fields.
    type Field: Clone + Debug + Send + Sync + 'static;
}

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

impl From<crudkit_id::IdValue> for Value {
    fn from(value: crudkit_id::IdValue) -> Self {
        match value {
            crudkit_id::IdValue::I32(value) => Value::I32(value),
            crudkit_id::IdValue::U32(value) => Value::U32(value),
            crudkit_id::IdValue::I64(value) => Value::I64(value),
            crudkit_id::IdValue::U64(value) => Value::U64(value),
            crudkit_id::IdValue::I128(value) => Value::I128(value),
            crudkit_id::IdValue::U128(value) => Value::U128(value),
            crudkit_id::IdValue::Bool(value) => Value::Bool(value),
            crudkit_id::IdValue::String(value) => Value::String(value),
            crudkit_id::IdValue::Uuid(value) => Value::Uuid(value),
            crudkit_id::IdValue::PrimitiveDateTime(value) => Value::PrimitiveDateTime(value),
            crudkit_id::IdValue::OffsetDateTime(value) => Value::OffsetDateTime(value),
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

/// Successful save result.
///
/// Returned when an entity is successfully created or updated.
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct Saved<T> {
    /// The saved entity.
    pub entity: T,

    /// Non-critical validation violations (warnings) associated with this entity.
    /// Empty if no violations exist.
    #[schema(value_type = Object)]
    pub violations: crudkit_validation::PartialSerializableAggregateViolations,
}

impl<T> Saved<T> {
    /// Returns true if there are any validation violations (warnings) for this entity.
    pub fn has_validation_errors(&self) -> bool {
        !self.violations.is_empty()
    }
}

/// Successful delete result.
///
/// Returned when entities are successfully deleted.
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct Deleted {
    /// Number of entities that were deleted.
    pub entities_affected: u64,
}

/// Result of a batch delete operation.
///
/// Provides detailed information about which entities were deleted successfully
/// and which failed for various reasons.
///
/// Note: IDs are represented as `serde_json::Value` to avoid circular dependencies.
/// They are serialized `SerializableId` values.
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct DeletedMany {
    /// Number of successfully deleted entities.
    pub deleted_count: u64,

    /// IDs of successfully deleted entities.
    #[schema(value_type = Vec<Object>)]
    pub deleted_ids: Vec<serde_json::Value>,

    /// IDs of entities where deletion was aborted by a lifecycle hook, with the abort reason.
    #[schema(value_type = Vec<Object>)]
    pub aborted: Vec<(serde_json::Value, String)>,

    /// IDs of entities that failed validation (critical validation errors prevented deletion).
    #[schema(value_type = Vec<Object>)]
    pub validation_failed: Vec<serde_json::Value>,

    /// IDs of entities that failed due to other errors, with the error message.
    #[schema(value_type = Vec<Object>)]
    pub errors: Vec<(serde_json::Value, String)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;

    #[test]
    fn delete_many_result_serializes_correctly() {
        let result = DeletedMany {
            deleted_count: 2,
            deleted_ids: vec![
                serde_json::json!([["id", {"I64": 1}]]),
                serde_json::json!([["id", {"I64": 2}]]),
            ],
            aborted: vec![],
            validation_failed: vec![],
            errors: vec![],
        };

        let _json = serde_json::to_string(&result).expect("serialization should succeed");
    }

    #[test]
    fn delete_many_result_deserializes_correctly() {
        let json = r#"{
            "deleted_count": 3,
            "deleted_ids": [[["id", {"I64": 1}]], [["id", {"I64": 2}]], [["id", {"I64": 3}]]],
            "aborted": [],
            "validation_failed": [[["id", {"I64": 4}]]],
            "errors": [[[["id", {"I64": 5}]], "Database error"]]
        }"#;

        let result: DeletedMany =
            serde_json::from_str(json).expect("deserialization should succeed");

        assert_that(result.deleted_count).is_equal_to(3);
        assert_that(result.deleted_ids.len()).is_equal_to(3);
        assert_that(result.aborted.len()).is_equal_to(0);
        assert_that(result.validation_failed.len()).is_equal_to(1);
        assert_that(result.errors.len()).is_equal_to(1);
    }

    #[test]
    fn delete_many_result_with_partial_failures() {
        let result = DeletedMany {
            deleted_count: 1,
            deleted_ids: vec![serde_json::json!([["id", {"I64": 1}]])],
            aborted: vec![(
                serde_json::json!([["id", {"I64": 2}]]),
                "Entity is referenced elsewhere".to_string(),
            )],
            validation_failed: vec![serde_json::json!([["id", {"I64": 3}]])],
            errors: vec![(
                serde_json::json!([["id", {"I64": 4}]]),
                "Database connection lost".to_string(),
            )],
        };

        // Round-trip test
        let json = serde_json::to_string(&result).expect("serialization should succeed");
        let deserialized: DeletedMany =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert_that(deserialized.deleted_count).is_equal_to(result.deleted_count);
        assert_that(deserialized.deleted_ids.len()).is_equal_to(result.deleted_ids.len());
        assert_that(deserialized.aborted.len()).is_equal_to(result.aborted.len());
        assert_that(deserialized.validation_failed.len())
            .is_equal_to(result.validation_failed.len());
        assert_that(deserialized.errors.len()).is_equal_to(result.errors.len());
    }
}
