//! Type-erased (dynamic) types for runtime polymorphism.
//!
//! These types allow crudkit-leptos components to work with any resource type
//! without knowing concrete types at compile time.
//!
//! # Three-Tier Type Erasure Pattern
//!
//! The framework uses a consistent three-tier pattern for runtime polymorphism:
//!
//! | Tier | Pattern | Purpose | Example |
//! |------|---------|---------|---------|
//! | 1 | Typed traits | Compile-time generic code | `Model`, `FieldAccess<T>` |
//! | 2 | `Erased*` traits | Object-safe trait objects | `ErasedModel`, `ErasedField` |
//! | 3 | `Dyn*` wrappers | Boxed/Arc wrappers with helpers | `DynModel`, `DynCreateField` |
//!
//! ## Why Three Tiers?
//!
//! - **Tier 1 (Typed)**: Provides full type safety and zero-cost abstractions. Used when
//!   concrete types are known at compile time.
//!
//! - **Tier 2 (Erased)**: Makes traits dyn-compatible by removing generic parameters.
//!   Uses `#[typetag::serde]` for serialization and `downcast_rs` for type recovery.
//!
//! - **Tier 3 (Dyn wrappers)**: Provides ergonomic wrappers around `Box<dyn Erased*>` or
//!   `Arc<dyn Erased*>` with convenience methods like `downcast_ref<T>()`.
//!
//! ## Conversion Flow
//!
//! ```text
//! T: Model  →  auto-impl ErasedModel  →  wrap in DynModel  →  downcast<T>() when needed
//! ```
//!
//! ## Naming Conventions
//!
//! - `Erased*` prefix: Traits for type-erased behavior (e.g., `ErasedModel`)
//! - `Dyn*` prefix: Boxed trait object wrappers (e.g., `DynModel = Box<dyn ErasedModel>`)
//!
//! The wrappers provide:
//! - `new(concrete)` - Create from concrete type
//! - `downcast<T>()` - Consume and recover concrete type (panics on mismatch)
//! - `downcast_ref<T>()` - Borrow as concrete type (panics on mismatch)
//! - `downcast_mut<T>()` - Mutably borrow as concrete type (panics on mismatch)

use crate::{HasId, Model, Named};
use crudkit_core::Value;
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;
use erased_serde::__private::serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

// Re-export from crudkit-id for backwards compatibility.
pub use crudkit_core::id::DynIdentifiable;
pub use crudkit_core::id::ErasedIdentifiable;

#[typetag::serde]
pub trait ErasedModel:
    ErasedIdentifiable + Debug + DynClone + DynEq + downcast_rs::Downcast + Send + Sync
{
}
dyn_eq::eq_trait_object!(ErasedModel);
dyn_clone::clone_trait_object!(ErasedModel);
downcast_rs::impl_downcast!(ErasedModel);

// Note: Every ErasedCreateModel needs to be `Default`, but that would introduce a `Sized` bound,
// rendering this trait dyn-incompatible.
#[typetag::serde]
pub trait ErasedCreateModel: ErasedModel {}
dyn_eq::eq_trait_object!(ErasedCreateModel);
dyn_clone::clone_trait_object!(ErasedCreateModel);
downcast_rs::impl_downcast!(ErasedCreateModel);

#[typetag::serde]
pub trait ErasedUpdateModel: ErasedModel {}
dyn_eq::eq_trait_object!(ErasedUpdateModel);
dyn_clone::clone_trait_object!(ErasedUpdateModel);
downcast_rs::impl_downcast!(ErasedUpdateModel);

#[typetag::serde]
pub trait ErasedReadModel: ErasedModel {}
dyn_eq::eq_trait_object!(ErasedReadModel);
dyn_clone::clone_trait_object!(ErasedReadModel);
downcast_rs::impl_downcast!(ErasedReadModel);

macro_rules! impl_dyn_model {
    ($dyn_ty:tt, $erased_ty:tt) => {
        /// Any type-erased (boxed) model.
        #[derive(Debug, Clone, Eq)]
        pub struct $dyn_ty {
            pub(crate) inner: Box<dyn $erased_ty>,
        }

        impl PartialEq for $dyn_ty {
            fn eq(&self, other: &Self) -> bool {
                self.inner.dyn_eq(DynEq::as_any(&other.inner))
            }
        }

        impl<T: $erased_ty> From<T> for $dyn_ty {
            fn from(value: T) -> Self {
                Self {
                    inner: Box::new(value),
                }
            }
        }

        impl $dyn_ty {
            pub fn new<Concrete: $erased_ty>(concrete: Concrete) -> Self {
                Self {
                    inner: Box::new(concrete),
                }
            }

            pub fn downcast<Concrete: $erased_ty>(self) -> Concrete {
                *self.inner.downcast::<Concrete>().expect("correct")
            }

            pub fn downcast_ref<Concrete: $erased_ty>(&self) -> &Concrete {
                self.inner.downcast_ref::<Concrete>().expect("correct")
            }

            pub fn downcast_mut<Concrete: $erased_ty>(&mut self) -> &mut Concrete {
                self.inner.downcast_mut::<Concrete>().expect("correct")
            }
        }

        impl std::ops::Deref for $dyn_ty {
            type Target = dyn $erased_ty;

            fn deref(&self) -> &Self::Target {
                self.inner.as_ref()
            }
        }
    };
}

impl_dyn_model!(DynModel, ErasedModel);
impl_dyn_model!(DynCreateModel, ErasedCreateModel);
impl_dyn_model!(DynReadModel, ErasedReadModel);
impl_dyn_model!(DynUpdateModel, ErasedUpdateModel);

#[derive(Debug, Clone, PartialEq)]
pub enum ReadOrUpdateModel<ReadModel: Model + HasId, UpdateModel: Model + HasId> {
    Read(ReadModel),
    Update(UpdateModel),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynReadOrUpdateModel {
    Read(DynReadModel),
    Update(DynUpdateModel),
}

/// This trait is implemented for each derived type (enum) describing a models fields.
#[typetag::serde]
pub trait ErasedField:
    Debug + Named + DynClone + DynEq + DynHash + SerializeAsKey + Send + Sync
{
    fn set_value(&self, model: &mut DynModel, value: Value);

    /// Returns the kind of value this field holds.
    fn value_kind(&self) -> crudkit_core::ValueKind;

    /// Returns whether this field is optional.
    fn is_optional(&self) -> bool;
}
dyn_eq::eq_trait_object!(ErasedField);
dyn_clone::clone_trait_object!(ErasedField);
dyn_hash::hash_trait_object!(ErasedField);

/// A field known to be part of a `create` model. The type erased version is `DynCreateField`.
#[typetag::serde]
pub trait ErasedCreateField: ErasedField {
    fn set_value(&self, model: &mut DynCreateModel, value: Value);
    fn value_kind(&self) -> crudkit_core::ValueKind;
    fn is_optional(&self) -> bool;
}
dyn_eq::eq_trait_object!(ErasedCreateField);
dyn_clone::clone_trait_object!(ErasedCreateField);
dyn_hash::hash_trait_object!(ErasedCreateField);

/// A field known to be part of a `read` model. The type erased version is `DynReadField`.
#[typetag::serde]
pub trait ErasedReadField: ErasedField {
    fn set_value(&self, model: &mut DynReadModel, value: Value);
    fn value_kind(&self) -> crudkit_core::ValueKind;
    fn is_optional(&self) -> bool;
}
dyn_eq::eq_trait_object!(ErasedReadField);
dyn_clone::clone_trait_object!(ErasedReadField);
dyn_hash::hash_trait_object!(ErasedReadField);

/// A field known to be part of an `update` model. The type erased version is `DynUpdateField`.
#[typetag::serde]
pub trait ErasedUpdateField: ErasedField {
    fn set_value(&self, model: &mut DynUpdateModel, value: Value);
    fn value_kind(&self) -> crudkit_core::ValueKind;
    fn is_optional(&self) -> bool;
}
dyn_eq::eq_trait_object!(ErasedUpdateField);
dyn_clone::clone_trait_object!(ErasedUpdateField);
dyn_hash::hash_trait_object!(ErasedUpdateField);

/// Marker trait for type-erased field wrappers (`DynCreateField`, `DynReadField`, `DynUpdateField`).
///
/// This trait is implemented for the `Dyn*Field` wrapper types to provide a common
/// abstraction over all type-erased fields regardless of their model type.
pub trait TypeErasedField:
    Named + Debug + Clone + PartialEq + Eq + Hash + Send + Sync + 'static
{
    /// Returns the kind of value this field holds.
    fn value_kind(&self) -> crudkit_core::ValueKind;

    /// Returns whether this field is optional.
    fn is_optional(&self) -> bool;
}

macro_rules! impl_dyn_field {
    ($dyn_ty:tt, $erased_ty:tt, $dyn_model_ty:tt) => {
        /// Any field. Usable in collections.
        #[derive(Debug, Clone, Eq, Hash, serde::Serialize, serde::Deserialize)]
        pub struct $dyn_ty {
            inner: Arc<dyn $erased_ty>,
        }

        impl PartialEq for $dyn_ty {
            fn eq(&self, other: &Self) -> bool {
                self.inner.dyn_eq(DynEq::as_any(&other.inner))
            }
        }

        impl<T: $erased_ty> From<T> for $dyn_ty {
            fn from(value: T) -> Self {
                Self {
                    inner: Arc::new(value),
                }
            }
        }

        impl $dyn_ty {
            pub fn new<Concrete: $erased_ty>(concrete: Concrete) -> Self {
                Self {
                    inner: Arc::new(concrete),
                }
            }

            pub fn set_value(&self, model: &mut $dyn_model_ty, value: Value) {
                use std::ops::Deref;
                $erased_ty::set_value(self.inner.deref(), model, value);
            }
        }

        impl std::ops::Deref for $dyn_ty {
            type Target = dyn $erased_ty;

            fn deref(&self) -> &Self::Target {
                self.inner.as_ref()
            }
        }

        impl Named for $dyn_ty {
            fn name(&self) -> std::borrow::Cow<'static, str> {
                self.inner.name()
            }
        }

        impl TypeErasedField for $dyn_ty {
            fn value_kind(&self) -> crudkit_core::ValueKind {
                $erased_ty::value_kind(self.inner.as_ref())
            }

            fn is_optional(&self) -> bool {
                $erased_ty::is_optional(self.inner.as_ref())
            }
        }
    };
}

impl_dyn_field!(DynCreateField, ErasedCreateField, DynCreateModel);
impl_dyn_field!(DynReadField, ErasedReadField, DynReadModel);
impl_dyn_field!(DynUpdateField, ErasedUpdateField, DynUpdateModel);

/// Configuration trait for field serialization.
pub trait SerializeAsKey: Send + Sync {
    fn serialize_as_key(&self) -> String;
}

// TODO: This only exists for DynReadField. Why? Should we abstract?
#[derive(Debug, Eq, Hash)]
pub struct SerializableReadField {
    field: DynReadField,
}

impl PartialEq for SerializableReadField {
    fn eq(&self, other: &Self) -> bool {
        self.field.dyn_eq(DynEq::as_any(&other.field))
    }
}

impl SerializableReadField {
    pub fn into_inner(self) -> DynReadField {
        self.field
    }
}

impl Serialize for SerializableReadField {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.field.serialize_as_key().as_str().trim_matches('\"'))
    }
}

impl From<DynReadField> for SerializableReadField {
    fn from(field: DynReadField) -> Self {
        SerializableReadField { field }
    }
}

impl AsRef<DynReadField> for SerializableReadField {
    fn as_ref(&self) -> &DynReadField {
        &self.field
    }
}
