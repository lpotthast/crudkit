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
    pub use crudkit_core_macros::CkId;
    pub use derive_field::CkField;

    pub use super::CrudFieldValueTrait;
    // Backward compatibility alias for FieldAccess.
        pub use super::CrudIdTrait;
    // Backward compatibility alias for HasId.
    pub use super::CrudMainTrait;
    // Backward compatibility alias for Resource.
    pub use super::CrudModel;
    // Backward compatibility alias for Model.
    pub use super::CrudResourceTrait;
    pub use super::error::ErrorInfo;
    pub use super::files::FileResource;
    pub use super::files::ListFileError;
    pub use super::files::ListFilesResponse;
    pub use super::request_error::CrudOperationError;
    pub use super::request_error::RequestError;
    pub use super::reqwest_executor::ReqwestExecutor;
    pub use super::view::CrudView;
    pub use super::view::SerializableCrudView;
    // Backward compatibility trait.
    pub use super::FieldAccess;
    pub use super::FieldMode;
    pub use super::FieldOptions;
    pub use super::HasId;
    pub use super::HeaderOptions;
    pub use super::Label;
    pub use super::Model;
    pub use super::Named;
    pub use super::NoData;
    pub use super::OrderByUpdateOptions;
    pub use super::Resource;
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

    // Type-erased traits (Erased* prefix).
    pub use super::model::ErasedCreateField;
    pub use super::model::ErasedCreateModel;
    pub use super::model::ErasedField;
    pub use super::model::ErasedIdentifiable;
    pub use super::model::ErasedModel;
    pub use super::model::ErasedReadField;
    pub use super::model::ErasedReadModel;
    pub use super::model::ErasedUpdateField;
    pub use super::model::ErasedUpdateModel;

    // `Box`ed/`Arc`ed trait object wrappers (Dyn* prefix).
    pub use super::model::DynCreateField;
    pub use super::model::DynCreateModel;
    pub use super::model::DynIdentifiable;
    pub use super::model::DynModel;
    pub use super::model::DynReadField;
    pub use super::model::DynReadModel;
    pub use super::model::DynReadOrUpdateModel;
    pub use super::model::DynUpdateField;
    pub use super::model::DynUpdateModel;

    // Other model types.
    pub use super::model::ReadOrUpdateModel;
    pub use super::model::SerializableReadField;
    pub use super::model::SerializeAsKey;
    pub use super::model::TypeErasedField;

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

/// The central trait defining a CRUD resource for the frontend.
///
/// This trait combines resource identification with model type definitions.
/// Each resource has associated types for Create, Read, Update models and an ActionPayload type.
pub trait Resource: PartialEq + Default + Debug + Clone + Serialize + Send + Sync {
    /// Returns the resource name used in API URLs.
    fn resource_name() -> &'static str
    where
        Self: Sized;

    type CreateModel: Model + Default + Send + Sync;

    type ReadModelIdField: crudkit_id::IdField + Serialize + Send + Sync;
    type ReadModelId: Serialize
        + DeserializeOwned
        + crudkit_id::Id<Field = Self::ReadModelIdField>
        + PartialEq
        + Clone
        + Send
        + Sync;
    type ReadModel: Serialize
        + Model
        + Into<Self::UpdateModel>
        + HasId<Id = Self::ReadModelId>
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
    type UpdateModel: Serialize + Model + HasId<Id = Self::UpdateModelId> + Send + Sync;

    type ActionPayload: Serialize + CrudActionPayload + Send + Sync;
}

/// Backward compatibility alias.
pub use Resource as CrudMainTrait;


/// Backward compatibility alias - now merged into Resource.
pub use Resource as CrudResourceTrait;


/// Trait for typed model access in the frontend.
///
/// This extends the base `Model` trait from crudkit-core with frontend-specific
/// requirements for serialization, field enumeration, and value access.
///
/// Note: This does not have `HasId` as a supertrait, as not all data models
/// (namely the CreateModel) can supply an ID!
///
/// The bounds are a superset of `crudkit_core::Model`, so any type implementing
/// this trait also satisfies the core Model trait.
pub trait Model:
    PartialEq + Clone + Debug + Serialize + DeserializeOwned + Send + Sync + 'static
{
    type Field: Named
        + FieldAccess<Self>
        + PartialEq
        + Eq
        + Hash
        + Clone
        + Debug
        + Serialize
        + DeserializeOwned
        + Send
        + Sync
        + 'static;

    fn all_fields() -> Vec<Self::Field>;
    fn field(field_name: &str) -> Self::Field;
}

/// Backward compatibility alias.
pub use Model as CrudModel;


/// Re-export `HasId` from crudkit-id for typed ID access.
pub use crudkit_id::HasId;


/// Alias for backward compatibility.
pub use crudkit_id::HasId as CrudIdTrait;


/// Re-export `Named` from crudkit_core for backwards compatibility and convenience.
pub use crudkit_core::Named;


/// Trait for typed field value access.
pub trait FieldAccess<T> {
    fn value(&self, entity: &T) -> Value;
    fn set_value(&self, entity: &mut T, value: Value);
}

/// Backward compatibility alias.
pub use FieldAccess as CrudFieldValueTrait;


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
