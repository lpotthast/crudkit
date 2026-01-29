#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{borrow::Cow, fmt::Debug, hash::Hash};

pub mod action;
pub mod data_provider;
pub mod dyn_data_provider;
pub mod error;
pub mod files;
pub mod layout;
pub mod model;
pub mod request;
pub mod request_error;
pub mod reqwest_executor;
pub mod view;
/*
* Reexport common modules.
* This allows the user to only
*
* - `use crudkit_web::prelude::*` and
* - derive all common proc macros
*
* without the need to add more use declaration or
* to manually depend on other crud crates such as "crudkit_id",
* which are required for many derive macro implementations.
*/
use crate::request_error::RequestError;
use crudkit_core::Value;

use crate::action::CrudActionPayload;
pub use crudkit_collaboration;
pub use crudkit_condition;
pub use crudkit_core;
pub use crudkit_id;
pub use crudkit_validation;

pub mod prelude {
    pub use crudkit_collaboration;
    pub use crudkit_condition;
    pub use crudkit_core;
    pub use crudkit_id;
    pub use crudkit_validation;

    pub use derive_crud_action_payload::CkActionPayload;
    pub use derive_crud_resource::CkResource;
    pub use derive_crudkit_id::CkId;
    pub use derive_field::CkField;

    pub use super::error::ErrorInfo;
    pub use super::files::FileResource;
    pub use super::files::ListFileError;
    pub use super::files::ListFilesResponse;
    pub use super::request_error::CrudOperationError;
    pub use super::request_error::RequestError;
    pub use super::reqwest_executor::ReqwestExecutor;
    pub use super::view::CrudView;
    pub use super::view::SerializableCrudView;
    pub use super::CrudFieldValueTrait;
    pub use super::CrudIdTrait;
    pub use super::CrudMainTrait;
    pub use super::CrudModel;
    pub use super::CrudResourceTrait;
    pub use super::FieldMode;
    pub use super::FieldOptions;
    pub use super::HeaderOptions;
    pub use super::Label;
    pub use super::Named;
    pub use super::NoData;
    pub use super::OrderByUpdateOptions;
    pub use super::TabId;

    pub use super::data_provider::CreateOne;
    pub use super::data_provider::CrudRestDataProvider;
    pub use super::data_provider::DeleteById;
    pub use super::data_provider::ReadCount;
    pub use super::data_provider::ReadMany;
    pub use super::data_provider::ReadOne;
    pub use super::data_provider::UpdateOne;

    pub use super::action::ActionPayload;
    pub use super::action::AnyActionPayload;
    pub use super::action::CrudActionPayload;
    pub use super::action::EmptyActionPayload;

    pub use super::request::delete;
    pub use super::request::get;
    pub use super::request::post;
    pub use super::request::post_json;
    pub use super::request::put;
    pub use super::request::request;

    // Dynamic (type-erased) types
    pub use super::model::AnyCreateField;
    pub use super::model::AnyCreateModel;
    pub use super::model::AnyIdentifiable;
    pub use super::model::AnyModel;
    pub use super::model::AnyReadField;
    pub use super::model::AnyReadModel;
    pub use super::model::AnyReadOrUpdateModel;
    pub use super::model::AnyUpdateField;
    pub use super::model::AnyUpdateModel;
    pub use super::model::CreateField;
    pub use super::model::CreateModel;
    pub use super::model::DynField;
    pub use super::model::Field;
    pub use super::model::Identifiable;
    pub use super::model::Model;
    pub use super::model::ReadField;
    pub use super::model::ReadModel;
    pub use super::model::ReadOrUpdateModel;
    pub use super::model::SerializableReadField;
    pub use super::model::SerializeAsKey;
    pub use super::model::UpdateField;
    pub use super::model::UpdateModel;

    pub use super::layout::Elem;
    pub use super::layout::Enclosing;
    pub use super::layout::Group;
    pub use super::layout::Layout;
    pub use super::layout::Tab;

    // Dynamic data provider (Dyn-prefixed types only, DeleteById and ReadCount come from data_provider)
    pub use super::dyn_data_provider::DynCreateOne;
    pub use super::dyn_data_provider::DynCrudRestDataProvider;
    pub use super::dyn_data_provider::DynDeleteMany;
    pub use super::dyn_data_provider::DynReadMany;
    pub use super::dyn_data_provider::DynReadOne;
    pub use super::dyn_data_provider::DynUpdateOne;
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoData {
    NotYetLoaded,
    FetchFailed(RequestError),
    // TODO: Can probably be deleted at some point...
    FetchReturnedNothing,
    CreateFailed(RequestError),
    // TODO: Can probably be deleted at some point...
    CreateReturnedNothing,
    UpdateFailed(RequestError),
    // TODO: Can probably be deleted at some point...
    UpdateReturnedNothing,
}

// TODO: impl Clone if both types are clone, same for debug, ...
pub trait CrudMainTrait:
    CrudResourceTrait + PartialEq + Default + Debug + Clone + Serialize + Send + Sync
{
    type CreateModel: CrudModel + Default + Send + Sync;

    type ReadModelIdField: crudkit_id::IdField + Serialize + Send + Sync;
    type ReadModelId: Serialize
        + DeserializeOwned
        + crudkit_id::Id<Field = Self::ReadModelIdField>
        + PartialEq
        + Clone
        + Send
        + Sync;
    type ReadModel: Serialize
        + CrudModel
        + Into<Self::UpdateModel>
        + CrudIdTrait<Id = Self::ReadModelId>
        + Send
        + Sync;

    type UpdateModelIdField: crudkit_id::IdField + Serialize + Send + Sync;
    type UpdateModelId: Serialize
        + DeserializeOwned
        + crudkit_id::Id<Field = Self::UpdateModelIdField>
        + PartialEq
        + Clone
        + Send
        + Sync;
    type UpdateModel: Serialize + CrudModel + CrudIdTrait<Id = Self::UpdateModelId> + Send + Sync;

    type ActionPayload: Serialize + CrudActionPayload + Send + Sync;
}

/// This does not have CrudIdTrait as a super trait, as not all data models
/// (namely the CreateModel) can supply an ID!
pub trait CrudModel:
    PartialEq + Clone + Debug + Serialize + DeserializeOwned + Send + Sync
{
    type Field: Named
        + CrudFieldValueTrait<Self>
        + PartialEq
        + Eq
        + Hash
        + Clone
        + Debug
        + Serialize
        + DeserializeOwned
        + Send
        + Sync;

    fn get_all_fields() -> Vec<Self::Field>;
    fn get_field(field_name: &str) -> Self::Field;
}

/// Allows us to access the ID of an entity.
/// The ID type must provide more fine-grained access (for example to individual fields).
pub trait CrudIdTrait {
    type Id: crudkit_id::Id;

    fn get_id(&self) -> Self::Id;
}

impl<T: CrudIdTrait> CrudIdTrait for &T {
    type Id = T::Id;

    fn get_id(&self) -> Self::Id {
        T::get_id(&self)
    }
}

pub trait CrudResourceTrait {
    fn get_resource_name() -> &'static str
    where
        Self: Sized;
}

/// Re-export `Named` from crudkit_core for backwards compatibility and convenience.
pub use crudkit_core::Named;


pub trait CrudFieldValueTrait<T> {
    fn get_value(&self, entity: &T) -> Value;
    fn set_value(&self, entity: &mut T, value: Value);
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FieldMode {
    /// The value is displayed in a simple form.
    Display,

    Readable,

    Editable,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HeaderOptions {
    pub display_name: Cow<'static, str>,
    pub min_width: bool, // TODO: Rename to fit content.
    pub ordering_allowed: bool,
    pub date_time_display: DateTimeDisplay,
}

// TODO: we might want to use the builder pattern instead of relying on ..Default.default()
impl Default for HeaderOptions {
    fn default() -> Self {
        Self {
            display_name: Cow::Borrowed(""),
            min_width: false,
            ordering_allowed: true,
            date_time_display: DateTimeDisplay::LocalizedLocal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DateTimeDisplay {
    IsoUtc,
    LocalizedLocal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Label {
    pub name: String,
}

impl Label {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldOptions {
    pub disabled: bool,
    pub label: Option<Label>,
    pub date_time_display: DateTimeDisplay,
}

impl Default for FieldOptions {
    fn default() -> Self {
        Self {
            disabled: false,
            label: None,
            date_time_display: DateTimeDisplay::LocalizedLocal,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct OrderByUpdateOptions {
    pub append: bool,
}

pub type TabId = Cow<'static, str>;
