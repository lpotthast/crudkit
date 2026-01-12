use crudkit_shared::Value;
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::sync::Arc;
use utoipa::ToSchema;

pub mod prelude {
    pub use super::Id;
    pub use super::IdField;
    pub use super::IdValue;
    pub use super::SerializableId;
}

/// Values which might be part of an entities ID.
///
/// All variants must implement `Eq` for proper comparability!
/// This constraint excludes options like floats as parts of primary keys.
/// We might use the `ordered-float` create in the future to relax this constraint.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ToSchema, Serialize, Deserialize)]
pub enum IdValue {
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    I128(i128),
    U128(u128),
    Bool(bool),
    String(String),
    // TODO: Relax lock-on on `uuid` crate through feature-gate?
    Uuid(uuid::Uuid),
    // TODO: Relax lock-on on `time` crate through feature-gate?
    PrimitiveDateTime(time::PrimitiveDateTime),
    OffsetDateTime(time::OffsetDateTime),
}

/// A field of an entity ID.
///
/// The derive functionality automatically implements this trait for a (also) derived enum
/// which contains a variant for each field of a structs ID-tagged members.
/// The generated enum variants all carry a single value of the type of their original struct-field.
///
/// A `Vec<DynIdField>` could represent a dynamic entity ID.
pub trait IdField: Debug + Display + DynClone + Send + Sync {
    /// The name of this field in its type.
    fn name(&self) -> &'static str;

    /// The value of the field.
    fn to_value(&self) -> IdValue;
}
dyn_clone::clone_trait_object!(IdField);
pub type DynIdField = Arc<dyn IdField>;

/// An entity ID, comprised of multiple fields that form it.
///
/// ID's can be used as keys in data structures, as they are guaranteed to be `Eq`, `Ord` and
/// `Hash`!
///
/// You can generate a type-erased `SerializableId` using `to_serializable_id`.
pub trait Id:
    Debug + Display + DynClone + PartialEq + Eq + Hash + PartialOrd + Ord + Send + Sync
{
    /// The type of an individual ID field. This might be an enum, providing all possible fields.
    type Field: IdField + Sized;

    /// An iterator over all fields making up this ID.
    type FieldIter: Iterator<Item = Self::Field>;

    /// Iterate over all fields making up this ID.
    fn fields_iter(&self) -> Self::FieldIter;

    /// Get all fields making up this ID in  a new `Vec`.
    fn fields(&self) -> Vec<Self::Field>;

    /// Convert this typed ID to a type-erased ID usable in non-generic contexts or when
    /// serialization is required.
    fn to_serializable_id(&self) -> SerializableId;
}

/// A type-erased entity ID. Can be serialized/deserialized for storage or transmission.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ToSchema, Serialize, Deserialize)] // TODO: Serde passthrough?
#[schema(value_type = Vec<Object>
)] // TODO: Move away from unnamed (String, IdValue) and towars a named key/value tuple.
pub struct SerializableId(pub Vec<(String, IdValue)>);

impl From<IdValue> for Value {
    fn from(value: IdValue) -> Self {
        match value {
            IdValue::I32(value) => Value::I32(value),
            IdValue::U32(value) => Value::U32(value),
            IdValue::I64(value) => Value::I64(value),
            IdValue::U64(value) => Value::U64(value),
            IdValue::I128(value) => Value::I128(value),
            IdValue::U128(value) => Value::U128(value),
            IdValue::Bool(value) => Value::Bool(value),
            IdValue::String(value) => Value::String(value),
            IdValue::Uuid(value) => Value::Uuid(value),
            IdValue::PrimitiveDateTime(value) => Value::PrimitiveDateTime(value),
            IdValue::OffsetDateTime(value) => Value::OffsetDateTime(value),
        }
    }
}
