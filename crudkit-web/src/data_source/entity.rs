//! Reading, creating, and updating one entity: what an instance asks a data source for, and what
//! the data source answers.

use crate::model::{DynCreateModel, DynUpdateModel};
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
    /// A data source that sends only what changed, e.g. as a merge patch, compares `entity` with
    /// it. Its version, if the model has one, names the version the change is based on.
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
}

/// The entity that a data source created or changed.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CrudSaved {
    /// The entity as the data source stored it, which the form continues from.
    pub entity: DynUpdateModel,
    /// Non-critical validation violations of the entity. Empty if there are none.
    pub violations: PartialSerializableAggregateViolations,
}

impl CrudSaved {
    /// The stored `entity`, without violations.
    #[must_use]
    pub fn new(entity: DynUpdateModel) -> Self {
        Self {
            entity,
            violations: PartialSerializableAggregateViolations::default(),
        }
    }

    /// Returns this result with the non-critical `violations` of the entity.
    #[must_use]
    pub fn with_violations(mut self, violations: PartialSerializableAggregateViolations) -> Self {
        self.violations = violations;
        self
    }
}
