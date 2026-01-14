use crate::{CrudFieldNameTrait, FieldOptions, Label, Layout, TabId, Value};
use crudkit_id::SerializableId;
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;

pub mod crud_rest_data_provider;
pub mod requests;

pub mod prelude {
    pub use crate::prelude::*;

    pub use super::ActionPayload;
    pub use super::AnyActionPayload;
    pub use super::AnyIdentifiable;
    pub use super::AnyModel;
    pub use super::DynField;
    pub use super::Elem;
    pub use super::Enclosing;
    pub use super::Field;
    pub use super::Group;
    pub use super::Identifiable;
    pub use super::Model;
    pub use super::NamedProperty;
    pub use super::Tab;

    pub use super::crud_rest_data_provider::CreateOne;
    pub use super::crud_rest_data_provider::CrudRestDataProvider;
    pub use super::crud_rest_data_provider::DeleteById;
    pub use super::crud_rest_data_provider::ReadCount;
    pub use super::crud_rest_data_provider::ReadMany;
    pub use super::crud_rest_data_provider::ReadOne;
    pub use super::crud_rest_data_provider::UpdateOne;

    pub use super::requests::request;
    pub use super::requests::request_delete;
    pub use super::requests::request_get;
    pub use super::requests::request_post;
    pub use super::requests::request_put;
}

/// Anything that has an identifier in form of a `SerializableId`.
///
/// Trait is expected to be object safe.
pub trait Identifiable: Debug + DynClone + DynEq + Send + Sync {
    fn get_id(&self) -> SerializableId;
}
dyn_eq::eq_trait_object!(Identifiable);
dyn_clone::clone_trait_object!(Identifiable);

pub type AnyIdentifiable = Arc<dyn Identifiable>;

// TODO: Drop this trait? Rename CrudFieldNameTrait to NamedProperty.
pub trait NamedProperty: Send + Sync {
    fn get_name(&self) -> String;
}

/// This trait is implemented for each derived enum describing a models fields.
#[typetag::serde]
pub trait Field:
    Debug + CrudFieldNameTrait + DynClone + DynEq + DynHash + SerializeAsKey + Send + Sync
{
    fn set_value(&self, model: &mut AnyModel, value: Value);
}
dyn_eq::eq_trait_object!(Field);
dyn_clone::clone_trait_object!(Field);
dyn_hash::hash_trait_object!(Field);

/// A field known to be part of a `create` model. The type erased version is `AnyCreateField`.
#[typetag::serde]
pub trait CreateField: Field {
    fn set_value(&self, model: &mut AnyCreateModel, value: Value);
}
dyn_eq::eq_trait_object!(CreateField);
dyn_clone::clone_trait_object!(CreateField);
dyn_hash::hash_trait_object!(CreateField);

/// A field known to be part of a `read` model. The type erased version is `AnyReadField`.
#[typetag::serde]
pub trait ReadField: Field {
    fn set_value(&self, model: &mut AnyReadModel, value: Value);
}
dyn_eq::eq_trait_object!(ReadField);
dyn_clone::clone_trait_object!(ReadField);
dyn_hash::hash_trait_object!(ReadField);

/// A field known to be part of an `update` model. The type erased version is `AnyUpdateField`.
#[typetag::serde]
pub trait UpdateField: Field {
    fn set_value(&self, model: &mut AnyUpdateModel, value: Value);
}
dyn_eq::eq_trait_object!(UpdateField);
dyn_clone::clone_trait_object!(UpdateField);
dyn_hash::hash_trait_object!(UpdateField);

/// Any field as a trait object. Implemented for `AnyCreateField`, `AnyReadField` and
/// `AnyUpdateField`.
///
/// For a model type `Person`, `PersonField` would be the generated enum stating all fields.
/// In this scenario, `PersonField` would implement `Field` and `UpdateField` (the latter, as the
/// model is the update model and neither the read nor create model of the person resource).
///
/// A `PersonField` instance (a variant) can be type-erased as `AnyUpdateField`, as PersonField
/// declares the fields of an update model.
///
/// In a context where type-erased fields of any model (create, read or update) should be accepted,
/// `DynField` can be used.
pub trait DynField: Debug + Clone + PartialEq + Eq + Hash + Send + Sync + 'static {
    fn get_name(&self) -> &'static str;
}

macro_rules! impl_any_field {
    ($any_ty:tt, $concrete_ty:tt, $any_model_ty:tt) => {
        /// Any field. Usable in collections.
        #[derive(Debug, Clone, Eq, Hash, Serialize, Deserialize)]
        pub struct $any_ty {
            inner: Arc<dyn $concrete_ty>,
        }

        impl PartialEq for $any_ty {
            fn eq(&self, other: &Self) -> bool {
                self.inner.dyn_eq(DynEq::as_any(&other.inner))
            }
        }

        impl<T: $concrete_ty> From<T> for $any_ty {
            fn from(value: T) -> Self {
                Self {
                    inner: Arc::new(value),
                }
            }
        }

        impl $any_ty {
            pub fn new<Concrete: $concrete_ty>(concrete: Concrete) -> Self {
                Self {
                    inner: Arc::new(concrete),
                }
            }

            pub fn set_value(&self, model: &mut $any_model_ty, value: Value) {
                $concrete_ty::set_value(self.inner.deref(), model, value);
            }
        }

        impl Deref for $any_ty {
            type Target = dyn $concrete_ty;

            fn deref(&self) -> &Self::Target {
                self.inner.as_ref()
            }
        }

        impl DynField for $any_ty {
            fn get_name(&self) -> &'static str {
                self.inner.get_name()
            }
        }
    };
}

impl_any_field!(AnyCreateField, CreateField, AnyCreateModel);
impl_any_field!(AnyReadField, ReadField, AnyReadModel);
impl_any_field!(AnyUpdateField, UpdateField, AnyUpdateModel);

/// Configuration trait for field serialization
pub trait SerializeAsKey: Send + Sync {
    fn serialize_as_key(&self) -> String;
}

#[derive(Debug, Eq, Hash)]
pub struct SerializableReadField {
    field: AnyReadField,
}

impl PartialEq for SerializableReadField {
    fn eq(&self, other: &Self) -> bool {
        self.field.dyn_eq(DynEq::as_any(&other.field))
    }
}

impl SerializableReadField {
    pub fn into_inner(self) -> AnyReadField {
        self.field
    }
}

impl Serialize for SerializableReadField {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.field.serialize_as_key().as_str().trim_matches('\"'))
    }
}

impl From<AnyReadField> for SerializableReadField {
    fn from(field: AnyReadField) -> Self {
        SerializableReadField { field }
    }
}

impl AsRef<AnyReadField> for SerializableReadField {
    fn as_ref(&self) -> &AnyReadField {
        &self.field
    }
}

#[typetag::serde]
pub trait Model:
    Identifiable + Debug + DynClone + DynEq + downcast_rs::Downcast + Send + Sync
{
}
dyn_eq::eq_trait_object!(Model);
dyn_clone::clone_trait_object!(Model);
downcast_rs::impl_downcast!(Model);

// Note: Every CreateModel needs to be `Default`, but that would introduce a `Sized` bound,
// rendering this trait dyn-incompatible.
#[typetag::serde]
pub trait CreateModel: Model {}
dyn_eq::eq_trait_object!(CreateModel);
dyn_clone::clone_trait_object!(CreateModel);
downcast_rs::impl_downcast!(CreateModel);

#[typetag::serde]
pub trait UpdateModel: Model {}
dyn_eq::eq_trait_object!(UpdateModel);
dyn_clone::clone_trait_object!(UpdateModel);
downcast_rs::impl_downcast!(UpdateModel);

#[typetag::serde]
pub trait ReadModel: Model {}
dyn_eq::eq_trait_object!(ReadModel);
dyn_clone::clone_trait_object!(ReadModel);
downcast_rs::impl_downcast!(ReadModel);

macro_rules! impl_any_model {
    ($any_ty:tt, $concrete_ty:tt) => {
        #[derive(Debug, Clone, Eq)]
        pub struct $any_ty {
            inner: Box<dyn $concrete_ty>,
        }

        impl PartialEq for $any_ty {
            fn eq(&self, other: &Self) -> bool {
                self.inner.dyn_eq(DynEq::as_any(&other.inner))
            }
        }

        impl<T: $concrete_ty> From<T> for $any_ty {
            fn from(value: T) -> Self {
                Self {
                    inner: Box::new(value),
                }
            }
        }

        impl $any_ty {
            pub fn new<Concrete: $concrete_ty>(concrete: Concrete) -> Self {
                Self {
                    inner: Box::new(concrete),
                }
            }

            pub fn downcast<Concrete: $concrete_ty>(self) -> Concrete {
                *self.inner.downcast::<Concrete>().expect("correct")
            }

            pub fn downcast_ref<Concrete: $concrete_ty>(&self) -> &Concrete {
                self.inner.downcast_ref::<Concrete>().expect("correct")
            }

            pub fn downcast_mut<Concrete: $concrete_ty>(&mut self) -> &mut Concrete {
                self.inner.downcast_mut::<Concrete>().expect("correct")
            }
        }

        impl Deref for $any_ty {
            type Target = dyn $concrete_ty;

            fn deref(&self) -> &Self::Target {
                self.inner.as_ref()
            }
        }
    };
}

impl_any_model!(AnyModel, Model);
impl_any_model!(AnyCreateModel, CreateModel);
impl_any_model!(AnyReadModel, ReadModel);
impl_any_model!(AnyUpdateModel, UpdateModel);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnyReadOrUpdateModel {
    Read(AnyReadModel),
    Update(AnyUpdateModel),
}

pub trait ActionPayload: Debug + DynClone + DynEq + downcast_rs::Downcast + Send + Sync {}
dyn_eq::eq_trait_object!(ActionPayload);
dyn_clone::clone_trait_object!(ActionPayload);
downcast_rs::impl_downcast!(ActionPayload);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyActionPayload {
    inner: Box<dyn ActionPayload>,
}

impl AnyActionPayload {
    pub fn from<Concrete: ActionPayload>(concrete: Concrete) -> Self {
        Self {
            inner: Box::new(concrete),
        }
    }

    pub fn downcast<Concrete: ActionPayload>(self) -> Concrete {
        *self.inner.downcast::<Concrete>().expect("correct")
    }

    pub fn downcast_ref<Concrete: ActionPayload>(&self) -> &Concrete {
        self.inner.downcast_ref::<Concrete>().expect("correct")
    }

    pub fn downcast_mut<Concrete: ActionPayload>(&mut self) -> &mut Concrete {
        self.inner.downcast_mut::<Concrete>().expect("correct")
    }
}

impl Deref for AnyActionPayload {
    type Target = dyn ActionPayload;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Elem<F> {
    Enclosing(Enclosing<F>),
    Field((F, FieldOptions)),
    Separator,
}

impl Elem<AnyCreateField> {
    pub fn create_field(field: impl CreateField, options: FieldOptions) -> Self {
        Self::Field((field.into(), options))
    }
}

impl Elem<AnyUpdateField> {
    pub fn field(field: impl UpdateField, options: FieldOptions) -> Self {
        Self::Field((field.into(), options))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Enclosing<F> {
    None(Group<F>),
    Tabs(Vec<Tab<F>>),
    Card(Group<F>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tab<F> {
    /// A unique identifier for this tab.
    pub id: TabId,
    pub label: Label,
    pub group: Group<F>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group<F> {
    pub layout: Layout,
    pub children: Vec<Elem<F>>,
}
