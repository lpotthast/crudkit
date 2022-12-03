use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};

use crate::id::{SerializableId, Id};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub validator_name: &'static str,
    pub validator_version: i32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct OwnedValidatorInfo {
    pub validator_name: String,
    pub validator_version: i32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct EntityInfo<I: Id> {
    // TODO: use a safer enum type here?
    pub aggregate_name: &'static str,
    /// We might generate violations for entities which do not have an id yet, because they were not yet created!
    pub entity_id: Option<I>,
}

// TODO: Rename to PersistableEntityInfo, as it is only used in validation persistance?
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct StrictEntityInfo {
    // TODO: use a safer enum type here?
    pub aggregate_name: &'static str,
    pub entity_id: SerializableId,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct StrictOwnedEntityInfo {
    // TODO: use a safer enum type here?
    pub aggregate_name: String,
    pub entity_id: SerializableId,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ValidationViolation {
    Major(String),
    Critical(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Violations {
    pub violations: Vec<ValidationViolation>,
}

impl Violations {
    pub fn empty() -> Self {
        Self {
            violations: Vec::new(),
        }
    }

    pub fn push(&mut self, violation: ValidationViolation) {
        self.violations.push(violation);
    }

    pub fn is_empty(&self) -> bool {
        self.violations.is_empty()
    }

    pub fn has_critical_violations(&self) -> bool {
        for violation in &self.violations {
            match violation {
                ValidationViolation::Critical(_) => return true,
                _ => {}
            }
        }
        false
    }
}

// TODO: Delete?
pub enum ValidationResult {
    Unrelated(),
}

pub struct AggregateValidations<I: Id> {
    pub map: HashMap<String, AggregateViolations<I>>,
}

pub struct AggregateViolations<I: Id> {
    /// Violations targeting the resource as a whole. Not tied to a specific entity.
    pub general: Violations,
    /// Violations unrelated to a specific entity.
    pub create: Violations,
    /// Violations targeting specific entities.
    pub by_entity: EntityViolations<I>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntityViolations<I: Id> {
    pub entity_violations: HashMap<EntityInfo<I>, HashMap<ValidatorInfo, Violations>>,
}

impl<I: Id> EntityViolations<I> {
    pub fn empty() -> Self {
        Self {
            entity_violations: HashMap::new(),
        }
    }

    pub fn of(entity: EntityInfo<I>, validator: ValidatorInfo, violations: Violations) -> Self {
        let mut validator_violations = HashMap::new();
        validator_violations.insert(validator, violations);

        let mut entity_violations = HashMap::new();
        entity_violations.insert(entity, validator_violations);

        Self { entity_violations }
    }

    pub fn number_of_violations(&self) -> usize {
        let mut estimated_size = 0;
        for (_, validators) in &self.entity_violations {
            for (_, violations) in validators {
                estimated_size += violations.violations.len();
            }
        }
        estimated_size
    }

    pub fn has_violations(&self) -> bool {
        for (_entity_id, validator_violations) in &self.entity_violations {
            for (_validator_info, violations) in validator_violations {
                if !violations.is_empty() {
                    return true;
                }
            }
        }
        false
    }

    pub fn has_critical_violations(&self) -> bool {
        for (_entity_id, validator_violations) in &self.entity_violations {
            for (_validator_info, violations) in validator_violations {
                if violations.has_critical_violations() {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct FullSerializableAggregateViolations {
    /// Violations targeting the resource as a whole. Not tied to a specific entity.
    pub general: Vec<ValidationViolation>,

    /// Violations unrelated to any known entity.
    pub create: Vec<ValidationViolation>,

    /// Violations targeting specific entities.
    /// If the map does not contain an entry for an entity ID, then no information is present.
    /// If it does, the contained Vec must hold ALL violations for the entity at hand.
    pub by_entity: HashMap<SerializableId, Vec<ValidationViolation>>,
}

impl FullSerializableAggregateViolations {
    pub fn has_entity_or_general_violations(&self) -> bool {
        let has_general_violations = !self.general.is_empty();
        //let has_create_violations = !self.create.is_empty();
        let has_entity_violations = !self.by_entity.is_empty()
            && self
                .by_entity
                .iter()
                .any(|(_, violations)| !violations.is_empty());
        has_general_violations || /* has_create_violations || */ has_entity_violations
    }
}

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct SerializableAggregateViolations {
    /// Violations targeting the resource as a whole. Not tied to a specific entity.
    /// If Option::Empty, no information is present. In an Option::Some, the contained Vec must hold ALL violations for the resource at hand.
    pub general: Option<Vec<ValidationViolation>>,

    /// Violations unrelated to any known entity.
    /// If Option::Empty, no information is present. In an Option::Some, the contained Vec must hold ALL violations for the resource at hand.
    pub create: Option<Vec<ValidationViolation>>,

    /// Violations targeting specific entities.
    /// If the map does not contain an entry for an entity ID, then no information is present.
    /// If it does, the contained Vec must hold ALL violations for the entity at hand.
    pub by_entity: HashMap<SerializableId, Vec<ValidationViolation>>,
}

pub type AggregateName = String;

pub type FullSerializableValidations = HashMap<String, FullSerializableAggregateViolations>;

pub type PartialSerializableValidations = HashMap<String, SerializableAggregateViolations>;

impl<I: Id> Into<PartialSerializableValidations> for EntityViolations<I> {
    fn into(self) -> PartialSerializableValidations {
        let mut result: PartialSerializableValidations =
            HashMap::with_capacity(self.entity_violations.len());
        for (entity_info, validators) in self.entity_violations {
            let aggregate_violations = result
                .entry(entity_info.aggregate_name.to_owned())
                .or_insert_with(Default::default);

            if let Some(entity_id) = entity_info.entity_id {
                let entity_violations = aggregate_violations
                    .by_entity
                    .entry(entity_id)
                    .or_insert_with(Default::default);
                for (_, mut violations) in validators {
                    entity_violations.append(&mut violations.violations);
                }
            } else {
                for (_, mut violations) in validators {
                    aggregate_violations
                        .create
                        .get_or_insert_with(Default::default)
                        .append(&mut violations.violations);
                }
            }
        }
        result
    }
}
