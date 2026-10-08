//! Type-safe entity identifiers with composite primary key support.
//!
//! This module provides traits and types for working with entity IDs in a type-safe manner.
//! Unlike most CRUD frameworks, this supports composite primary keys via the [`Id`] trait.

use dyn_clone::DynClone;
use dyn_eq::DynEq;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::sync::Arc;

pub mod prelude {
    pub use super::DynIdentifiable;
    pub use super::ErasedIdentifiable;
    pub use super::HasId;
    pub use super::Id;
    pub use super::IdField;
    pub use super::IdValue;
    pub use super::SerializableId;
}

/// Trait for models that have an identifier.
///
/// Not all models have an ID - `CreateModel` typically doesn't since the ID
/// is generated during insertion. `ReadModel` and `UpdateModel` should implement this.
pub trait HasId {
    /// The ID type for this model.
    type Id: Id + Clone + Send + Sync + 'static;

    /// Extract the ID from this model instance.
    fn id(&self) -> Self::Id;
}

impl<T: HasId> HasId for &T {
    type Id = T::Id;

    fn id(&self) -> Self::Id {
        T::id(self)
    }
}

/// Type-erased trait for anything that has an identifier in form of a `SerializableId`.
///
/// This trait is object-safe, allowing it to be used as `dyn ErasedIdentifiable`
/// for runtime polymorphism over different ID types.
pub trait ErasedIdentifiable: Debug + DynClone + DynEq + Send + Sync {
    fn id(&self) -> SerializableId;
}
dyn_eq::eq_trait_object!(ErasedIdentifiable);
dyn_clone::clone_trait_object!(ErasedIdentifiable);

impl<T> ErasedIdentifiable for T
where
    T: HasId + Debug + Clone + Eq + Send + Sync + 'static,
{
    fn id(&self) -> SerializableId {
        HasId::id(self).to_serializable_id()
    }
}

/// A type-erased, reference-counted identifiable value.
pub type DynIdentifiable = Arc<dyn ErasedIdentifiable>;

/// Values which might be part of an entities ID.
///
/// All variants must implement `Eq` for proper comparability!
/// This constraint excludes options like floats as parts of primary keys.
/// We might use the `ordered-float` create in the future to relax this constraint.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Bool(bool),
    String(String),
    // TODO: Relax lock-on on `uuid` crate through feature-gate?
    Uuid(uuid::Uuid),
    // TODO: Relax lock-on on `time` crate through feature-gate?
    PrimitiveDateTime(time::PrimitiveDateTime),
    OffsetDateTime(time::OffsetDateTime),
}

/// Shows the bare value, e.g. `4` or `2024-01-05 10:30:00.0`.
impl Display for IdValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::I8(value) => Display::fmt(value, f),
            Self::I16(value) => Display::fmt(value, f),
            Self::I32(value) => Display::fmt(value, f),
            Self::I64(value) => Display::fmt(value, f),
            Self::I128(value) => Display::fmt(value, f),
            Self::U8(value) => Display::fmt(value, f),
            Self::U16(value) => Display::fmt(value, f),
            Self::U32(value) => Display::fmt(value, f),
            Self::U64(value) => Display::fmt(value, f),
            Self::U128(value) => Display::fmt(value, f),
            Self::Bool(value) => Display::fmt(value, f),
            Self::String(value) => f.pad(value),
            Self::Uuid(value) => Display::fmt(value, f),
            Self::PrimitiveDateTime(value) => Display::fmt(value, f),
            Self::OffsetDateTime(value) => Display::fmt(value, f),
        }
    }
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
    fn name(&self) -> &str;

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

    /// Convert this typed ID to a type-erased `SerializableId` usable in non-generic contexts
    /// or when serialization/deserialization is required.
    fn to_serializable_id(&self) -> SerializableId;

    /// Attempt to reconstruct this ID from a type-erased `SerializableId`.
    /// Returns `None` if the `SerializableId` doesn't match the structure expected by this type.
    // TODO: rename to try_from_serializable_id and return error giving detailed info where the serializable id was not matching.
    fn from_serializable_id(id: &SerializableId) -> Option<Self>
    where
        Self: Sized;
}

/// A type-erased entity ID. Can be serialized and deserialized for storage or transmission.
///
/// The first tuple element stores a field name.
///
/// The type of resource or model this ID belongs to is not encoded in this datastructure.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SerializableId(pub Vec<SerializableIdEntry>);

impl SerializableId {
    pub fn entries(&self) -> impl Iterator<Item = &SerializableIdEntry> {
        self.0.iter()
    }

    pub fn into_entries(self) -> impl Iterator<Item = SerializableIdEntry> {
        self.0.into_iter()
    }
}

/// Shows the ID's fields with their values, e.g. `id=4` or `club_id=4, season=2024`.
impl Display for SerializableId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (index, entry) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            Display::fmt(entry, f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SerializableIdEntry {
    pub field_name: String,
    pub value: IdValue,
}
impl From<SerializableIdEntry> for (String, IdValue) {
    fn from(entry: SerializableIdEntry) -> Self {
        (entry.field_name, entry.value)
    }
}

impl From<(String, IdValue)> for SerializableIdEntry {
    fn from((field_name, value): (String, IdValue)) -> Self {
        Self { field_name, value }
    }
}

/// Shows the field with its value, e.g. `club_id=4`.
impl Display for SerializableIdEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.field_name, self.value)
    }
}

impl IdField for SerializableIdEntry {
    fn name(&self) -> &str {
        self.field_name.as_str()
    }

    fn to_value(&self) -> IdValue {
        self.value.clone()
    }
}

impl Id for SerializableId {
    type Field = SerializableIdEntry;
    type FieldIter = std::vec::IntoIter<Self::Field>;

    fn fields_iter(&self) -> Self::FieldIter {
        self.0.clone().into_iter()
    }

    fn fields(&self) -> Vec<Self::Field> {
        self.0.clone()
    }

    fn to_serializable_id(&self) -> SerializableId {
        self.clone()
    }

    fn from_serializable_id(id: &SerializableId) -> Option<Self>
    where
        Self: Sized,
    {
        Some(id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;

    #[test]
    fn ids_display_their_fields_and_values() {
        let id = SerializableId(vec![
            SerializableIdEntry {
                field_name: "club_id".to_owned(),
                value: IdValue::I64(4),
            },
            SerializableIdEntry {
                field_name: "season".to_owned(),
                value: IdValue::String("2024".to_owned()),
            },
        ]);
        assert_that!(id.to_string()).is_equal_to("club_id=4, season=2024".to_owned());
    }

    #[test]
    fn id_entries_display_their_field_and_value() {
        let entry = SerializableIdEntry {
            field_name: "season".to_owned(),
            value: IdValue::String("2024".to_owned()),
        };
        assert_that!(entry.to_string()).is_equal_to("season=2024".to_owned());
    }

    #[test]
    fn string_id_values_respect_width_and_fill() {
        let value = IdValue::String("ab".to_owned());
        assert_that!(format!("{value:*>4}")).is_equal_to("**ab".to_owned());
    }
}
