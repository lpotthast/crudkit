//! Deleting entities: what an instance asks a data source for, what the data source answers, and
//! the default deletion of many entities one after the other.

use super::{CrudCompletion, CrudDataSource, CrudScope, CrudVersion};
use crate::http::RequestError;
use crate::model::DynReadOrUpdateModel;
use crudkit_core::id::SerializableId;

/// The entity that [`CrudDataSource::delete`] deletes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CrudDeleteRequest {
    /// The ID of the entity.
    pub id: SerializableId,
    /// The entity as the user last saw it.
    pub entity: DynReadOrUpdateModel,
}

impl CrudDeleteRequest {
    /// Requests deleting `entity`.
    #[must_use]
    pub fn new(entity: DynReadOrUpdateModel) -> Self {
        Self {
            id: entity.id(),
            entity,
        }
    }

    /// Returns the version that the deletion is based on: the version of the entity as the user
    /// last saw it. `None` if the model has no version.
    #[must_use]
    pub fn version(&self) -> Option<&CrudVersion> {
        self.entity.version()
    }
}

/// The outcome of [`CrudDataSource::delete`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CrudDeleted {
    /// The number of entities deleted, `1` once the entity is gone or its deletion is accepted.
    pub entities_affected: u64,
    /// Whether the deletion is complete, or accepted while the entity is still being deleted. The
    /// entity of an accepted deletion still exists, e.g. in the state `deleting`, which instances
    /// show once they reload.
    pub completion: CrudCompletion,
}

impl CrudDeleted {
    /// The outcome of a completed deletion that deleted `entities_affected` entities.
    #[must_use]
    pub fn new(entities_affected: u64) -> Self {
        Self {
            entities_affected,
            completion: CrudCompletion::Completed,
        }
    }

    /// The outcome of an accepted deletion of one entity, which still runs, e.g. because an API
    /// answered `202 Accepted`.
    #[must_use]
    pub fn accepted() -> Self {
        Self {
            entities_affected: 1,
            completion: CrudCompletion::Accepted,
        }
    }
}

/// The entities that [`CrudDataSource::delete_many`] deletes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CrudDeleteManyRequest {
    /// The entities as the user last saw them.
    pub entities: Vec<DynReadOrUpdateModel>,
}

impl CrudDeleteManyRequest {
    /// Requests deleting `entities`.
    #[must_use]
    pub fn new(entities: Vec<DynReadOrUpdateModel>) -> Self {
        Self { entities }
    }
}

/// The outcome of [`CrudDataSource::delete_many`], for each entity.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct CrudDeletedMany {
    /// The IDs of the deleted entities.
    pub deleted: Vec<SerializableId>,
    /// The IDs of the entities whose deletion the data source accepted, and which are still
    /// being deleted.
    pub accepted: Vec<SerializableId>,
    /// The entities that were not deleted, with the reason of each. A refusal with
    /// [`RequestError::Forbidden`], [`RequestError::UnprocessableEntity`],
    /// [`RequestError::Outdated`], or [`RequestError::InvalidFields`] counts as aborted, and one
    /// with [`RequestError::CriticalValidationErrors`] as failed validation.
    pub failed: Vec<(SerializableId, RequestError)>,
    /// The entities that the data source did not try to delete, because a failure that concerns
    /// more than one entity, e.g. a lost connection, stopped the deletion.
    pub skipped: Vec<SerializableId>,
}

impl CrudDeletedMany {
    /// An outcome in which no entity was deleted yet. Fill its fields.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Deletes the entities of `request` with [`CrudDataSource::delete`], one after the other.
///
/// An entity counts as deleted only if its deletion affected an entity, and as accepted if the
/// data source accepted its deletion. A failure that concerns only the entity, e.g. a refusal, is
/// recorded and the deletion goes on. Any other failure, e.g. a lost connection or a missing
/// authentication, is recorded for the entity, and the remaining entities are skipped.
pub(super) async fn delete_one_after_another<S: CrudDataSource + ?Sized>(
    data_source: &S,
    scope: &CrudScope,
    request: CrudDeleteManyRequest,
) -> CrudDeletedMany {
    let mut outcome = CrudDeletedMany::new();
    let mut entities = request.entities.into_iter();
    for entity in entities.by_ref() {
        let request = CrudDeleteRequest::new(entity);
        let id = request.id.clone();
        match data_source.delete(scope, request).await {
            Ok(deleted) if deleted.completion == CrudCompletion::Accepted => {
                outcome.accepted.push(id);
            }
            Ok(deleted) if deleted.entities_affected > 0 => outcome.deleted.push(id),
            Ok(_) => outcome.failed.push((
                id,
                RequestError::NotFound("the entity was not deleted".to_owned()),
            )),
            Err(error) => {
                let stops = !concerns_only_the_entity(&error);
                outcome.failed.push((id, error));
                if stops {
                    break;
                }
            }
        }
    }
    outcome.skipped.extend(entities.map(|entity| entity.id()));
    outcome
}

/// Returns whether `error` concerns only the entity a request was about, so that requests about
/// other entities may still succeed.
fn concerns_only_the_entity(error: &RequestError) -> bool {
    match error {
        RequestError::BadRequest(_)
        | RequestError::Forbidden(_)
        | RequestError::NotFound(_)
        | RequestError::UnprocessableEntity(_)
        | RequestError::CriticalValidationErrors { .. }
        | RequestError::Outdated(_)
        | RequestError::InvalidFields { .. }
        | RequestError::InvalidRequest(_) => true,
        RequestError::Unauthorized(_)
        | RequestError::InternalServerError(_)
        | RequestError::Request(_)
        | RequestError::Deserialize(_) => false,
    }
}
