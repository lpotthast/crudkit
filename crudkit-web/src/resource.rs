//! The frontend description of a CRUD resource.

use crate::model::Model;
use crudkit_core::id::{HasId, Id};
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

/// The central trait defining a CRUD resource for the frontend.
///
/// This trait combines resource identification with model type definitions.
/// Each resource has associated types for Create, Read, Update models and an `ActionPayload` type.
pub trait Resource: PartialEq + Default + Debug + Clone + Serialize + Send + Sync {
    /// Returns the resource name used in API URLs.
    fn resource_name() -> &'static str;

    type CreateModel: Model + Default + Send + Sync;

    type ReadModelId: Serialize + DeserializeOwned + Id + PartialEq + Clone + Send + Sync;
    type ReadModel: Serialize
        + Model
        + Into<Self::UpdateModel>
        + HasId<Id = Self::ReadModelId>
        + Send
        + Sync;

    type UpdateModelId: Serialize + DeserializeOwned + Id + PartialEq + Clone + Send + Sync;
    type UpdateModel: Serialize + Model + HasId<Id = Self::UpdateModelId> + Send + Sync;

    type ActionPayload: Serialize + crate::action::ActionPayload + Send + Sync;
}
