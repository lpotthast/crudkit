use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct EntityInfo {
    pub entity_name: &'static str,
    /// We might generate violations for entities which do not have an id yet, because they were not yet created!
    pub entity_id: Option<i32>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct StrictEntityInfo {
    pub entity_name: &'static str,
    pub entity_id: i32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct StrictOwnedEntityInfo {
    pub entity_name: String,
    pub entity_id: i32,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ValidationViolation {
    Major(String),
    Critical(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct EntityViolations {
    pub violations: Vec<ValidationViolation>,
}

impl EntityViolations {
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

#[derive(Debug, Serialize)]
pub struct EntityValidations {
    pub entity_violations: HashMap<EntityInfo, HashMap<ValidatorInfo, EntityViolations>>,
}

impl EntityValidations {
    pub fn empty() -> Self {
        Self {
            entity_violations: HashMap::new(),
        }
    }

    pub fn of(entity: EntityInfo, validator: ValidatorInfo, violations: EntityViolations) -> Self {
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
        for (_entity_info, validator_violations) in &self.entity_violations {
            for (_validator_info, violations) in validator_violations {
                if !violations.is_empty() {
                    return true;
                }
            }
        }
        false
    }

    pub fn has_critical_violations(&self) -> bool {
        for (_entity_info, validator_violations) in &self.entity_violations {
            for (_validator_info, violations) in validator_violations {
                if violations.has_critical_violations() {
                    return true;
                }
            }
        }
        false
    }
}

pub type SerializableValidations = HashMap<String, HashMap<i32, Vec<ValidationViolation>>>;

impl Into<SerializableValidations> for &EntityValidations
{
    fn into(self) -> SerializableValidations {
        let mut aggregate_violations = HashMap::with_capacity(self.entity_violations.len());
        for (entity_info, validators) in &self.entity_violations {
            // If the validation is for a known ID, use that id.
            // All violations for unknown entities go under -1.
            let entity_id = match entity_info.entity_id {
                Some(id) => id,
                None => -1,
            };

            let mut entity_violations = HashMap::with_capacity(validators.len());
            for (_, violations) in validators {
                let mut vec = Vec::with_capacity(violations.violations.len());
                for violation in &violations.violations {
                    vec.push(violation.clone())
                }
                entity_violations.insert(entity_id, vec);
            }
            aggregate_violations.insert(entity_info.entity_name.to_owned(), entity_violations);
        }
        aggregate_violations
    }
}
