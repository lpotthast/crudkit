//! Reading, creating, and updating one entity: what an instance asks a data source for, and what
//! the data source answers.

use super::{CrudCompletion, CrudVersion, merge_patch};
use crate::model::{DynCreateModel, DynUpdateModel, UntaggedJsonError};
use crudkit_core::id::SerializableId;
use crudkit_core::validation::PartialSerializableAggregateViolations;

/// The entity that [`CrudDataSource::read`](super::CrudDataSource::read) reads.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CrudReadRequest {
    /// The ID of the entity.
    pub id: SerializableId,
}

impl CrudReadRequest {
    /// Requests the entity `id`.
    #[must_use]
    pub fn new(id: SerializableId) -> Self {
        Self { id }
    }
}

/// The entity that [`CrudDataSource::create`](super::CrudDataSource::create) creates.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CrudCreateRequest {
    /// The entity as the user entered it. The instance filled the fields that refer to the
    /// parents of the scope, if the create model has them.
    pub entity: DynCreateModel,
}

impl CrudCreateRequest {
    /// Requests creating `entity`.
    #[must_use]
    pub fn new(entity: DynCreateModel) -> Self {
        Self { entity }
    }
}

/// The change that [`CrudDataSource::update`](super::CrudDataSource::update) makes.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CrudUpdateRequest {
    /// The ID of the changed entity, as it was loaded. The user may have changed ID fields in
    /// `entity`.
    pub id: SerializableId,
    /// The entity as the user changed it.
    pub entity: DynUpdateModel,
    /// The entity as the user's change started from: the version that was loaded or saved last.
    /// A data source that sends only what changed compares `entity` with it, e.g. with
    /// [`Self::merge_patch`].
    pub baseline: DynUpdateModel,
}

impl CrudUpdateRequest {
    /// Requests changing the entity `id` from `baseline` to `entity`.
    #[must_use]
    pub fn new(id: SerializableId, entity: DynUpdateModel, baseline: DynUpdateModel) -> Self {
        Self {
            id,
            entity,
            baseline,
        }
    }

    /// Returns the version that the change is based on: the version of the baseline. `None` if
    /// the model has no version.
    #[must_use]
    pub fn version(&self) -> Option<&CrudVersion> {
        self.baseline.version()
    }

    /// Returns what the user changed, as the JSON merge patch (RFC 7396) that turns the baseline
    /// into the entity, both serialized as the model serializes them. A member that the user
    /// cleared is `null`, and an unchanged member is left out.
    ///
    /// # Errors
    ///
    /// Returns an error if one of the models cannot be serialized as a plain JSON value.
    pub fn merge_patch(&self) -> Result<serde_json::Value, UntaggedJsonError> {
        Ok(merge_patch(
            &self.baseline.to_untagged_json()?,
            &self.entity.to_untagged_json()?,
        ))
    }
}

/// The entity that a data source created or changed.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CrudSaved {
    /// The entity as the data source stored it, which the form continues from. After an accepted
    /// change, the entity is in its state at the time of the answer.
    pub entity: DynUpdateModel,
    /// Non-critical validation violations of the entity. Empty if there are none.
    pub violations: PartialSerializableAggregateViolations,
    /// Whether the change is complete, or accepted while the entity is still changing.
    pub completion: CrudCompletion,
}

impl CrudSaved {
    /// The stored `entity` of a completed change, without violations.
    #[must_use]
    pub fn new(entity: DynUpdateModel) -> Self {
        Self {
            entity,
            violations: PartialSerializableAggregateViolations::default(),
            completion: CrudCompletion::Completed,
        }
    }

    /// The `entity` of an accepted change, in its state at the time of the answer, without
    /// violations. The entity is still changing, e.g. because an API answered `202 Accepted`.
    #[must_use]
    pub fn accepted(entity: DynUpdateModel) -> Self {
        Self {
            completion: CrudCompletion::Accepted,
            ..Self::new(entity)
        }
    }

    /// Returns this result with the non-critical `violations` of the entity.
    #[must_use]
    pub fn with_violations(mut self, violations: PartialSerializableAggregateViolations) -> Self {
        self.violations = violations;
        self
    }
}
