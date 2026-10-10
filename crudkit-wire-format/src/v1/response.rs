//! Response bodies of the generated CRUD routes.

use super::{PartialViolationsV1, SerializableIdV1, WireFormatVersionV1};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response body of counting entities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ReadCountResponseV1 {
    /// Wire format version of the body.
    pub wire_format_version: WireFormatVersionV1,
    /// Number of matching entities.
    pub count: u64,
}

/// Response body of reading one entity of the application's read model `T`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ReadOneResponseV1<T> {
    /// Wire format version of the body.
    pub wire_format_version: WireFormatVersionV1,
    /// The matching entity, or `null` if no entity matched.
    pub entity: Option<T>,
}

/// Response body of reading many entities of the application's read model `T`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ReadManyResponseV1<T> {
    /// Wire format version of the body.
    pub wire_format_version: WireFormatVersionV1,
    /// The matching entities, in the requested order.
    pub entities: Vec<T>,
}

/// Response body of creating or updating an entity of the application's model `T`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct SavedV1<T> {
    /// Wire format version of the body.
    pub wire_format_version: WireFormatVersionV1,
    /// The saved entity.
    pub entity: T,
    /// Non-critical violations of the saved entity.
    pub violations: PartialViolationsV1,
}

/// Response body of deleting a single entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DeletedV1 {
    /// Wire format version of the body.
    pub wire_format_version: WireFormatVersionV1,
    /// Number of deleted entities.
    pub entities_affected: u64,
}

/// An entity that a mass deletion did not delete, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct FailedDeletionV1 {
    /// The entity.
    pub id: SerializableIdV1,
    /// Why the entity was not deleted.
    pub reason: String,
}

/// Response body of deleting many entities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DeletedManyV1 {
    /// Wire format version of the body.
    pub wire_format_version: WireFormatVersionV1,
    /// Number of deleted entities.
    pub deleted_count: u64,
    /// The deleted entities.
    pub deleted_ids: Vec<SerializableIdV1>,
    /// Entities whose deletion a lifecycle hook aborted.
    pub aborted: Vec<FailedDeletionV1>,
    /// Entities whose critical validation errors prevented their deletion.
    pub validation_failed: Vec<SerializableIdV1>,
    /// Entities whose deletion failed with an error.
    pub errors: Vec<FailedDeletionV1>,
}
