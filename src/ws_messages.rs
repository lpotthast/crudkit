use serde::{Serialize, Deserialize};

use crate::validation::{PartialSerializableValidations, FullSerializableValidations};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CrudWsMessage {
    EntityCreated(EntityCreated),
    EntityUpdated(EntityUpdated),
    EntityDeleted(EntityDeleted),
    PartialValidationResult(PartialSerializableValidations),
    FullValidationResult(FullSerializableValidations),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityCreated {
    pub aggregate_name: String,
    pub entity_id: i32,
    pub with_validation_errors: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityUpdated {
    pub aggregate_name: String,
    pub entity_id: i32,
    pub with_validation_errors: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityDeleted {
    pub aggregate_name: String,
    pub entity_id: i32,
}
