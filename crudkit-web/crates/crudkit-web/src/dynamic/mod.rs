use crate::{FieldOptions, Label, Layout, TabId, Value};
use crudkit_id::SerializableId;
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

pub mod crud_rest_data_provider;
pub mod requests;

pub mod prelude {
    pub use crate::prelude::*;

    pub use super::ActionPayload;
    pub use super::AnyActionPayload;
    pub use super::AnyElem;
    pub use super::AnyEnclosing;
    pub use super::AnyField;
    pub use super::AnyGroup;
    pub use super::AnyIdentifiable;
    pub use super::AnyModel;
    pub use super::AnyTab;
    pub use super::Field;
    pub use super::Identifiable;
    pub use super::Model;
    pub use super::NamedProperty;

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

pub trait NamedProperty: Send + Sync {
    fn get_name(&self) -> String;
}

/// A `#[typetag::serde]` annotation was omitted intentionally, as custom serializers are
/// expected to be provided through configuration.
#[typetag::serde] // Required for serialize/deserialize on AnyElem.
pub trait Field:
    Debug + NamedProperty + DynClone + DynEq + DynHash + SerializeAsKey + Send + Sync
{
    fn set_value(&self, model: &mut AnyModel, value: Value);
}
dyn_eq::eq_trait_object!(Field);
dyn_clone::clone_trait_object!(Field);
dyn_hash::hash_trait_object!(Field);

#[derive(Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct AnyField {
    inner: Arc<dyn Field>,
}

impl PartialEq for AnyField {
    fn eq(&self, other: &Self) -> bool {
        self.inner.dyn_eq(other.inner.as_any())
    }
}

impl AnyField {
    pub fn from<Concrete: Field>(concrete: Concrete) -> Self {
        Self {
            inner: Arc::new(concrete),
        }
    }

    //pub fn downcast<Concrete: Field>(self) -> Concrete {
    //    *self.inner.downcast::<Concrete>().expect("correct")
    //}
    //pub fn downcast_ref<Concrete: Field>(&self) -> &Concrete {
    //    self.inner.downcast_ref::<Concrete>().expect("correct")
    //}
    //pub fn downcast_mut<Concrete: Field>(&mut self) -> &mut Concrete {
    //    self.inner.downcast_mut::<Concrete>().expect("correct")
    //}
}

impl Deref for AnyField {
    type Target = dyn Field;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

/// Configuration trait for field serialization
pub trait SerializeAsKey: Send + Sync {
    fn serialize_as_key(&self) -> String;
}

#[derive(Debug, Eq, Hash)]
pub struct SerializableField {
    field: AnyField,
}

impl PartialEq for SerializableField {
    fn eq(&self, other: &Self) -> bool {
        self.field.dyn_eq(other.field.as_any())
    }
}

impl SerializableField {
    pub fn into_inner(self) -> AnyField {
        self.field
    }
}

impl Serialize for SerializableField {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.field.serialize_as_key().as_str().trim_matches('\"'))
    }
}

impl From<AnyField> for SerializableField {
    fn from(field: AnyField) -> Self {
        SerializableField { field }
    }
}

impl AsRef<AnyField> for SerializableField {
    fn as_ref(&self) -> &AnyField {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyModel {
    inner: Box<dyn Model>,
}

impl AnyModel {
    pub fn from<Concrete: Model>(concrete: Concrete) -> Self {
        Self {
            inner: Box::new(concrete),
        }
    }

    pub fn downcast<Concrete: Model>(self) -> Concrete {
        *self.inner.downcast::<Concrete>().expect("correct")
    }

    pub fn downcast_ref<Concrete: Model>(&self) -> &Concrete {
        self.inner.downcast_ref::<Concrete>().expect("correct")
    }

    pub fn downcast_mut<Concrete: Model>(&mut self) -> &mut Concrete {
        self.inner.downcast_mut::<Concrete>().expect("correct")
    }
}

impl Deref for AnyModel {
    type Target = dyn Model;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
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
pub enum AnyElem {
    Enclosing(AnyEnclosing),
    Field((AnyField, FieldOptions)),
    Separator,
}

impl AnyElem {
    pub fn field(field: impl Field, options: FieldOptions) -> Self {
        Self::Field((AnyField::from(field), options))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnyEnclosing {
    None(AnyGroup),
    Tabs(Vec<AnyTab>),
    Card(AnyGroup),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnyTab {
    /// A unique identifier for this tab.
    pub id: TabId,
    pub label: Label,
    pub group: AnyGroup,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnyGroup {
    pub layout: Layout,
    pub children: Vec<AnyElem>,
}
