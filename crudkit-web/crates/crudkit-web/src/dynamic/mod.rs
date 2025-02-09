use crate::{FieldOptions, Label, Layout, TabId, Value};
use crudkit_id::SerializableId;
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
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

#[typetag::serde]
pub trait Field: Debug + NamedProperty + DynClone + DynEq + DynHash + Send + Sync {
    fn set_value(&self, model: &mut AnyModel, value: Value);
}
dyn_eq::eq_trait_object!(Field);
dyn_clone::clone_trait_object!(Field);
dyn_hash::hash_trait_object!(Field);

pub type AnyField = Arc<dyn Field>;

#[typetag::serde]
pub trait Model:
    Identifiable + Debug + DynClone + DynEq + downcast_rs::Downcast + Send + Sync
{
}
dyn_eq::eq_trait_object!(Model);
dyn_clone::clone_trait_object!(Model);
downcast_rs::impl_downcast!(Model);

pub type AnyModel = Box<dyn Model>;

pub trait ActionPayload: Debug + DynClone + DynEq + Send + Sync {}

pub type AnyActionPayload = Arc<dyn ActionPayload>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnyElem {
    Enclosing(AnyEnclosing),
    Field((AnyField, FieldOptions)),
    Separator,
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
