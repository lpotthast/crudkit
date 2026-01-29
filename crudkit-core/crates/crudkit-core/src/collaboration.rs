//! Types used for sharing data between different users via WebSocket.

use serde::{Deserialize, Serialize};

use crate::id::SerializableId;
use crate::validation::{FullSerializableValidations, PartialSerializableValidations};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CollabMessage {
    /// An entity was created.
    /// Expect an additional `PartialValidationResult` soon.
    EntityCreated(EntityCreated),

    /// An entity was updated.
    /// Expect an additional `PartialValidationResult` soon.
    EntityUpdated(EntityUpdated),

    /// An entity was deleted.
    /// Any known violations for the referenced entity should be deleted.
    EntityDeleted(EntityDeleted),

    /// Some entities were validated.
    /// These results should be merged into the already known validation state forming an updated
    /// world view.
    PartialValidationResult(PartialSerializableValidations),

    /// All entities / resources were validated.
    /// The currently known validation state can be overwritten with this new result.
    FullValidationResult(FullSerializableValidations),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityCreated {
    pub resource_name: String,
    pub entity_id: SerializableId,
    /// These can only be non-critical violations, as critical's would have prevented the save.
    pub with_validation_errors: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityUpdated {
    pub resource_name: String,
    pub entity_id: SerializableId,
    /// These can only be non-critical violations, as critical's would have prevented the save.
    pub with_validation_errors: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityDeleted {
    pub resource_name: String,
    pub entity_id: SerializableId,
}
