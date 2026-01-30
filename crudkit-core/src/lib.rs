#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Core types shared by both backend (crudkit-rs) and frontend (crudkit-web) layers.
//!
//! This crate provides the foundational types for the crudkit ecosystem:
//!
//! - **`id`**: Type-safe entity identifiers with composite primary key support
//! - **`resource`**: Resource naming types
//! - **`condition`**: Query filtering DSL
//! - **`validation`**: Entity validation framework with severity levels
//! - **`collaboration`**: Types for multi-user collaboration via WebSocket
//!
//! # Re-exports
//!
//! For convenience, commonly used types are re-exported at the crate root.

pub mod collaboration;
pub mod condition;
pub mod id;
pub mod resource;
pub mod validation;

// Re-export commonly used types at crate root.
pub use id::{HasId, Id, IdField, IdValue, SerializableId, SerializableIdEntry};
pub use resource::ResourceName;
pub use validation::{
    FullSerializableAggregateViolations, FullSerializableValidations,
    PartialSerializableAggregateViolations, PartialSerializableValidations, ViolationsByValidator,
};

use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;
use time::format_description::well_known::Rfc3339;
use utoipa::openapi::Type;
use utoipa::ToSchema;

// ============================================================================
// Model traits
// ============================================================================

/// A trait for types that have a name.
pub trait Named {
    /// Returns the name of this item.
    fn name(&self) -> Cow<'static, str>;
}

/// Base trait for data models in the CRUD framework.
///
/// This is the minimal shared definition used by both backend (crudkit-rs)
/// and frontend (crudkit-web) layers. Each layer extends this with additional
/// bounds and methods appropriate for their context.
///
/// # Model Trait Hierarchy
///
/// The framework has three `Model` traits, each adding layer-specific bounds:
///
/// | Crate | Trait | Additional Bounds | Purpose |
/// |-------|-------|-------------------|---------|
/// | `crudkit-core` | `Model` (this trait) | — | Minimal shared definition |
/// | `crudkit-rs` | `Model` | `Field: crudkit_rs::Field` | Backend with field metadata |
/// | `crudkit-web` | `Model` | `Serialize`, `DeserializeOwned`, `PartialEq`, accessor methods | Frontend with serialization |
///
/// All three share the same fundamental structure: a model type with an associated
/// `Field` enum. The differences are in what additional capabilities each layer requires.
///
/// # Marker Traits for Model Roles
///
/// Models play different roles in CRUD operations. The backend defines marker traits
/// in `crudkit-rs::data`:
///
/// - **`CreateModel`**: DTOs for creating new entities (typically lacks auto-generated fields like ID).
/// - **`UpdateModel`**: DTOs for updating existing entities.
/// - **`ReadModel`**: Entities returned from queries (must have ID, be serializable).
///
/// These marker traits bundle the required bounds for each role without adding new methods.
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

// ============================================================================
// Value system
// ============================================================================

/// Represents the type kind of a `Value` variant.
///
/// This enum provides runtime type information for field values, enabling
/// dynamic dispatch based on value types without matching on `Value` directly.
///
/// # Design
///
/// Each variant corresponds to a `Value` variant. Optionality is tracked
/// separately via field metadata (`is_optional()`), not via separate variants.
///
/// # Usage
///
/// Primarily used by:
/// - Field metadata (`FieldAccess::value_kind()`) to describe expected value types
/// - Field renderers to select the appropriate UI component
/// - Code generation to map Rust types to Value variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValueKind {
    Null,
    Void,
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    String,
    Json,
    Uuid,
    PrimitiveDateTime,
    OffsetDateTime,
    Duration,
    /// Homogeneous array (element type known from field metadata).
    Array,
    /// Fallback for custom types using `Value::Other`.
    Other,
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

/// Extension trait for custom field value types.
///
/// Implement this trait for custom types that need to be stored in `Value::Other`.
/// Built-in types (bool, integers, String, etc.) have dedicated `Value` variants
/// and don't need this trait.
#[typetag::serde]
pub trait FieldValue: Debug + DynClone + Send + Sync + 'static {}
dyn_clone::clone_trait_object!(FieldValue);

/// Values which can be used by crud fields.
///
/// The optionality of a field is tracked separately via field metadata (`is_optional()`),
/// not via separate Optional* variants. Use `Value::Null` to represent an absent value
/// for optional fields.
#[derive(Debug, Clone)]
pub enum Value {
    /// Explicit absence of a value: An optional field having no current value.
    Null,

    /// The unit type `()` field.
    ///
    /// Unlike `Null` (which represents "no value present for this optional field"),
    /// `Void` simply is the value for the unit value `()`.
    Void(()),

    // Primitives.
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    F32(f32),
    F64(f64),

    // Common types.
    String(String),

    // Ecosystem support.
    // -- serde
    Json(serde_json::Value),
    // -- uuid
    Uuid(uuid::Uuid),
    // -- time
    PrimitiveDateTime(time::PrimitiveDateTime),
    OffsetDateTime(time::OffsetDateTime),
    Duration(TimeDuration),

    // Collections.
    /// Homogeneous array (element type known from field metadata).
    Array(Vec<Value>),

    // Extension support.
    Other(Box<dyn FieldValue>),
}

impl From<IdValue> for Value {
    fn from(value: IdValue) -> Self {
        match value {
            IdValue::I8(value) => Value::I8(value),
            IdValue::I16(value) => Value::I16(value),
            IdValue::I32(value) => Value::I32(value),
            IdValue::I64(value) => Value::I64(value),
            IdValue::I128(value) => Value::I128(value),
            IdValue::U8(value) => Value::U8(value),
            IdValue::U16(value) => Value::U16(value),
            IdValue::U32(value) => Value::U32(value),
            IdValue::U64(value) => Value::U64(value),
            IdValue::U128(value) => Value::U128(value),
            IdValue::Bool(value) => Value::Bool(value),
            IdValue::String(value) => Value::String(value),
            IdValue::Uuid(value) => Value::Uuid(value),
            IdValue::PrimitiveDateTime(value) => Value::PrimitiveDateTime(value),
            IdValue::OffsetDateTime(value) => Value::OffsetDateTime(value),
        }
    }
}

/// Generates `as_<variant>` accessor methods that return `Option<T>` by copying.
macro_rules! impl_as {
    ($($variant:ident, $ty:ty);* $(;)?) => {
        paste::paste! {
            $(
                pub fn [<as_ $variant:snake>](&self) -> Option<$ty> {
                    match self {
                        Self::$variant(v) => Some(*v),
                        _ => None,
                    }
                }
            )*
        }
    };
}

/// Generates `as_<variant>` accessor methods that return `Option<&T>` by reference.
macro_rules! impl_as_ref {
    ($($variant:ident, $ty:ty);* $(;)?) => {
        paste::paste! {
            $(
                pub fn [<as_ $variant:snake>](&self) -> Option<&$ty> {
                    match self {
                        Self::$variant(v) => Some(v),
                        _ => None,
                    }
                }
            )*
        }
    };
}

/// Generates `take_<variant>` consuming accessor methods that return `Option<T>`.
macro_rules! impl_take {
    ($($variant:ident, $ty:ty);* $(;)?) => {
        paste::paste! {
            $(
                pub fn [<take_ $variant:snake>](self) -> Option<$ty> {
                    match self {
                        Self::$variant(v) => Some(v),
                        _ => None,
                    }
                }
            )*
        }
    };
}

/// Generates `expect_<variant>` methods that call `as_<variant>` and panic on None.
macro_rules! impl_expect {
    ($($variant:ident, $ty:ty);* $(;)?) => {
        paste::paste! {
            $(
                pub fn [<expect_ $variant:snake>](&self) -> $ty {
                    self.[<as_ $variant:snake>]().expect(concat!("Value is not ", stringify!($variant)))
                }
            )*
        }
    };
}

impl Value {
    /// Returns true if this value is Null.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    // === Copy accessors (primitives) ===

    impl_as! {
        Bool, bool;
        U8, u8;
        U16, u16;
        U32, u32;
        U64, u64;
        U128, u128;
        I8, i8;
        I16, i16;
        I32, i32;
        I64, i64;
        I128, i128;
        F32, f32;
        F64, f64;
        Uuid, uuid::Uuid;
    }

    // === Reference accessors ===

    impl_as_ref! {
        String, String;
        Json, serde_json::Value;
        Duration, TimeDuration;
        Array, Vec<Value>;
        Other, Box<dyn FieldValue>;
    }

    // === Date/time accessors with String parsing fallback ===

    pub fn as_primitive_date_time(&self) -> Option<time::PrimitiveDateTime> {
        match self {
            Self::PrimitiveDateTime(v) => Some(*v),
            Self::String(s) => time::PrimitiveDateTime::parse(s, &Rfc3339).ok(),
            _ => None,
        }
    }

    pub fn as_offset_date_time(&self) -> Option<time::OffsetDateTime> {
        match self {
            Self::OffsetDateTime(v) => Some(*v),
            Self::String(s) => time::OffsetDateTime::parse(s, &Rfc3339).ok(),
            _ => None,
        }
    }

    // === Taking ownership (consuming accessors) ===

    impl_take! {
        String, String;
        Json, serde_json::Value;
        Duration, TimeDuration;
        Array, Vec<Value>;
        Other, Box<dyn FieldValue>;
    }

    // === Expect methods (panic on wrong type or Null) ===

    impl_expect! {
        Bool, bool;
        U8, u8;
        U16, u16;
        U32, u32;
        U64, u64;
        U128, u128;
        I8, i8;
        I16, i16;
        I32, i32;
        I64, i64;
        I128, i128;
        F32, f32;
        F64, f64;
        Uuid, uuid::Uuid;
        PrimitiveDateTime, time::PrimitiveDateTime;
        OffsetDateTime, time::OffsetDateTime;

        String, &String;
        Json, &serde_json::Value;
        Duration, &TimeDuration;
        Array, &Vec<Value>;
        Other, &Box<dyn FieldValue>;
    }

    // === Array utilities ===

    /// Verifies that all elements in a Value array are of the same type.
    ///
    /// Returns `Ok(())` if the slice is empty or all elements have the same discriminant.
    /// Returns `Err(index)` with the index of the first element that differs from the first.
    pub fn verify_array_homogeneity(values: &[Value]) -> Result<(), usize> {
        let Some(first) = values.first() else {
            return Ok(());
        };
        let expected = std::mem::discriminant(first);
        for (i, v) in values.iter().enumerate().skip(1) {
            if std::mem::discriminant(v) != expected {
                return Err(i);
            }
        }
        Ok(())
    }
}

// ============================================================================
// Result types
// ============================================================================

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
    pub violations: PartialSerializableAggregateViolations,
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
