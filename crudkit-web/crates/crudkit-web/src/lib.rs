#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{borrow::Cow, fmt::Debug, hash::Hash};

pub mod dynamic;
pub mod error;
pub mod files;
pub mod generic;
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

pub use crudkit_condition;
pub use crudkit_core;
pub use crudkit_id;
pub use crudkit_validation;
pub use crudkit_websocket;

pub(crate) mod prelude {
    pub use crudkit_condition;
    pub use crudkit_core;
    pub use crudkit_id;
    pub use crudkit_validation;
    pub use crudkit_websocket;

    pub use derive_crud_action_payload::CkActionPayload;
    pub use derive_crud_resource::CkResource;
    pub use derive_crud_selectable::CkSelectable;
    pub use derive_crudkit_id::CkId;
    pub use derive_field::CkField;
    pub use derive_field_value::CkFieldValue;

    pub use super::CrudActionPayload;
    pub use super::CrudDataTrait;
    pub use super::CrudFieldNameTrait;
    pub use super::CrudFieldValueTrait;
    pub use super::CrudIdTrait;
    pub use super::CrudMainTrait;
    pub use super::CrudResourceTrait;
    pub use super::DeletableModel;
    pub use super::EmptyActionPayload;
    pub use super::FieldMode;
    pub use super::FieldOptions;
    pub use super::HeaderOptions;
    pub use super::Label;
    pub use super::Layout;
    pub use super::NoData;
    pub use super::OrderByUpdateOptions;
    pub use super::TabId;
    pub use super::error::ErrorInfo;
    pub use super::files::FileResource;
    pub use super::files::ListFileError;
    pub use super::files::ListFilesResponse;
    pub use super::request_error::RequestError;
    pub use super::reqwest_executor::ReqwestExecutor;
    pub use super::view::CrudSimpleView;
    pub use super::view::CrudView;
    pub use super::view::SerializableCrudView;
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
    type CreateModel: CrudDataTrait + Default + Send + Sync;

    type ReadModelIdField: crudkit_id::IdField + Serialize + Send + Sync;
    type ReadModelId: Serialize
        + DeserializeOwned
        + crudkit_id::Id<Field = Self::ReadModelIdField>
        + PartialEq
        + Clone
        + Send
        + Sync;
    type ReadModel: Serialize
        + CrudDataTrait
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
    type UpdateModel: Serialize
        + CrudDataTrait
        + CrudIdTrait<Id = Self::UpdateModelId>
        + Send
        + Sync;

    type ActionPayload: Serialize + CrudActionPayload + Send + Sync;
}

/// Marker trait for specifying data which can be used as payload for CRUD actions.
/// Use the `CrudActionPayload` derive macro from derive-crud-action-payload to implement this for your type.
pub trait CrudActionPayload:
    PartialEq + Clone + Debug + Serialize + DeserializeOwned + Send + Sync
{
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct EmptyActionPayload {}

impl CrudActionPayload for EmptyActionPayload {}

/// This does not have CrudIdTrait as a super trait, as not all data models
/// (namely the CreateModel) can supply an ID!
pub trait CrudDataTrait:
    PartialEq + Clone + Debug + Serialize + DeserializeOwned + Send + Sync
{
    type Field: CrudFieldNameTrait
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

pub trait CrudFieldNameTrait {
    fn get_name(&self) -> &'static str;
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum DeletableModel<
    ReadModel: CrudDataTrait + CrudIdTrait,
    UpdateModel: CrudDataTrait + CrudIdTrait,
> {
    Read(ReadModel),
    Update(UpdateModel),
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
    //validations: Vec<u32>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Layout {
    Columns1,
    Columns2,
    Columns3,
    Columns4,
}

impl Default for Layout {
    fn default() -> Self {
        Self::Columns2
    }
}
