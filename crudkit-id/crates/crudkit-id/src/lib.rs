use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use utoipa::ToSchema;

pub mod prelude {
    pub use super::Id;
    pub use super::IdField;
    pub use super::IdValue;
    pub use super::SerializableId;
}

/// "ID-able" values. Values which might be part of an entities ID. All variants must implement `Eq` for proper comparability!
/// This constraint excludes options like floats as parts of primary keys.
/// We might use the `ordered-float` create in the future to relax this constraint.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ToSchema, Serialize, Deserialize)]
pub enum IdValue {
    String(String),
    UuidV4(uuid::Uuid),
    UuidV7(uuid::Uuid),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    Bool(bool),
    PrimitiveDateTime(time::PrimitiveDateTime),
    OffsetDateTime(time::OffsetDateTime),
}

/// A type implementing this trait represents fields of an ID value.
///
/// The derive functionality automatically implements this trait for a (also) derived enum
/// which contains a variant for each field of a structs ID-tagged members.
/// The generated enum variants all carry a single value of the type of their original struct-field.
///
/// A `Vec<dyn IdField>` could represent a dynamic entity ID.
pub trait IdField: Debug + Display + DynClone + Send + Sync {
    /// The name of this field in its type.
    fn name(&self) -> &'static str;
    /// The value of the field.
    fn to_value(&self) -> IdValue;
}
dyn_clone::clone_trait_object!(IdField);

/// Structs marked with this trait might be used as IDs in the crud system.
///
/// Id's might be used as keys in data structures, as they are guaranteed to be Eq, Ord and Hash!
///
/// You might want to generate a type-erased `SerializableId` using `into_serializable_id`.
pub trait Id:
    Debug + Display + DynClone + PartialEq + Eq + Hash + PartialOrd + Ord + Send + Sync
{
    /// This might be an enum, providing all possible fields.
    type Field: IdField + Sized;
    type FieldIter: Iterator<Item = Self::Field>;

    fn fields_iter(&self) -> Self::FieldIter;
    fn fields(&self) -> Vec<Self::Field>;

    fn into_serializable_id(&self) -> SerializableId;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ToSchema, Serialize, Deserialize)] // TODO: Serde passthrough?
#[schema(value_type = Vec<Object>)] // TODO: Move away from unnamed (String, IdValue) and towars a named key/value tuple.
pub struct SerializableId(pub Vec<(String, IdValue)>);
