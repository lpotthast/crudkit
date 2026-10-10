#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Platform-neutral frontend contracts of CrudKit: resources, typed and type-erased models, field
//! presentation, layouts, views, data providers, and UI-independent list, deletion, navigation,
//! and value-formatting logic.
//!
//! `use crudkit_web::prelude::*` brings the common types and derive macros into scope.

pub mod action;
pub mod data_provider;
pub mod delete;
pub mod dyn_data_provider;
pub mod field;
pub mod http;
pub mod layout;
pub mod list;
pub mod load_state;
pub mod model;
pub mod model_handler;
pub mod navigation;
pub mod resource;
pub mod value_format;
pub mod view;

pub use crudkit_core;
pub use crudkit_core::collaboration;
pub use crudkit_core::condition;
pub use crudkit_core::id;
pub use crudkit_core::validation;

pub mod prelude {
    pub use crudkit_core;
    pub use crudkit_core::collaboration;
    pub use crudkit_core::condition;
    pub use crudkit_core::id;
    pub use crudkit_core::validation;

    pub use crudkit_core_macros::CkId;
    pub use crudkit_web_macros::{CkActionPayload, CkField, CkResource};

    pub use crudkit_core::Named;
    pub use crudkit_core::id::HasId;

    pub use super::field::DateTimeDisplay;
    pub use super::field::FieldMode;
    pub use super::field::FieldOptions;
    pub use super::field::HeaderOptions;
    pub use super::field::Label;
    pub use super::http::RequestError;
    pub use super::http::ReqwestExecutor;
    pub use super::list::{ItemsPerPage, PageNr};
    pub use super::load_state::LoadState;
    pub use super::model::FieldAccess;
    pub use super::model::Model;
    pub use super::model_handler::ModelHandler;
    pub use super::resource::Resource;
    pub use super::view::CrudView;

    pub use super::data_provider::CrudRestDataProvider;
    pub use super::dyn_data_provider::DynCrudRestDataProvider;

    // CRUD request bodies.
    pub use crudkit_core::request::CreateOne;
    pub use crudkit_core::request::DeleteById;
    pub use crudkit_core::request::DeleteMany;
    pub use crudkit_core::request::DeleteOne;
    pub use crudkit_core::request::ReadCount;
    pub use crudkit_core::request::ReadMany;
    pub use crudkit_core::request::ReadOne;
    pub use crudkit_core::request::UpdateOne;

    pub use super::action::ActionPayload;
    pub use super::action::DynActionPayload;
    pub use super::action::EmptyActionPayload;
    pub use super::action::ErasedActionPayload;

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
    pub use super::model::DynReadField;
    pub use super::model::DynReadModel;
    pub use super::model::DynReadOrUpdateModel;
    pub use super::model::DynUpdateField;
    pub use super::model::DynUpdateModel;

    // Other model types.
    pub use super::model::IntoDynField;
    pub use super::model::TypeErasedField;

    pub use super::layout::Elem;
    pub use super::layout::Enclosing;
    pub use super::layout::Group;
    pub use super::layout::Layout;
    pub use super::layout::Tab;
    pub use super::layout::TabId;
}
