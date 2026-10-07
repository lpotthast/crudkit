//! Messages that inform connected clients about changes made by other users.

use super::{FullViolationsV1, PartialViolationsV1, SerializableIdV1, WireFormatVersionV1};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A collaboration message, e.g. sent over a websocket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CollabMessageV1 {
    /// Wire format version of the message.
    pub wire_format_version: WireFormatVersionV1,
    pub event: CollabEventV1,
}

/// What happened. Serialized with a `kind` tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CollabEventV1 {
    /// An entity was created. A partial validation result for it usually follows.
    EntityCreated {
        resource_name: String,
        entity_id: SerializableIdV1,
        /// Whether the entity has non-critical violations.
        with_validation_errors: bool,
    },
    /// An entity was updated. A partial validation result for it usually follows.
    EntityUpdated {
        resource_name: String,
        entity_id: SerializableIdV1,
        /// Whether the entity has non-critical violations.
        with_validation_errors: bool,
    },
    /// An entity was deleted. Receivers should forget its known violations.
    EntityDeleted {
        resource_name: String,
        entity_id: SerializableIdV1,
    },
    /// Some entities were validated. Receivers merge the results into what they know.
    PartialValidationResult {
        resources: Vec<ResourcePartialViolationsV1>,
    },
    /// Every entity of the listed resources was validated. Receivers replace what they know about
    /// these resources.
    FullValidationResult {
        resources: Vec<ResourceFullViolationsV1>,
    },
}

/// The partial violations of one resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ResourcePartialViolationsV1 {
    pub resource_name: String,
    pub violations: PartialViolationsV1,
}

/// The full violations of one resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ResourceFullViolationsV1 {
    pub resource_name: String,
    pub violations: FullViolationsV1,
}
