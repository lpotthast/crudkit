use std::collections::HashMap;

use async_trait::async_trait;
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub validator_name: &'static str,
    pub validator_version: i32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct EntityInfo {
    pub entity_name: &'static str,
    /// We might generate violations for entities which do not have an id yet, because they were not yet created!
    pub entity_id: Option<i32>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(16))")]
pub enum ValidationViolationType {
    #[sea_orm(string_value = "MAJOR")]
    Major,
    #[sea_orm(string_value = "CRITICAL")]
    Critical,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ValidationViolation {
    Major(String),
    Critical(String),
}

impl Into<ValidationViolationType> for ValidationViolation {
    fn into(self) -> ValidationViolationType {
        match self {
            ValidationViolation::Major(_) => ValidationViolationType::Major,
            ValidationViolation::Critical(_) => ValidationViolationType::Critical,
        }
    }
}

impl Into<ValidationViolationType> for &ValidationViolation {
    fn into(self) -> ValidationViolationType {
        match self {
            ValidationViolation::Major(_) => ValidationViolationType::Major,
            ValidationViolation::Critical(_) => ValidationViolationType::Critical,
        }
    }
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
}

pub trait EntityValidatorTrait<T> {
    fn validate_single(&self, entity: &T) -> EntityValidations;
    fn validate_updated(&self, old: &T, new: &T) -> EntityValidations;
    fn get_name(&self) -> &'static str;
    fn get_version(&self) -> u32;
}

pub trait EntityValidatorsTrait<T> {
    fn validate_single(&self, entity: &T) -> EntityValidations;
    fn validate_updated(&self, old: &T, new: &T) -> EntityValidations;
}

#[async_trait]
pub trait ValidationResultSaverTrait {
    async fn save_all(&self, validation_results: Vec<PersistableValidationResult>);
}

pub struct PersistableValidationResult {
    pub entity_name: String,
    pub entity_id: i32,
    pub validator_name: String,
    pub validator_version: i32,
    pub violation_severity: ValidationViolationType,
    pub violation_message: String,
}

impl Into<Vec<PersistableValidationResult>> for EntityValidations {
    fn into(self) -> Vec<PersistableValidationResult> {
        let mut vec = Vec::with_capacity(self.number_of_violations());
        for (entity_info, validators) in self.entity_violations {
            if let Some(entity_id) = entity_info.entity_id {
                for (validator_info, violations) in validators {
                    for violation in violations.violations {
                        vec.push(PersistableValidationResult {
                            entity_name: entity_info.entity_name.to_owned(),
                            entity_id,
                            validator_name: validator_info.validator_name.to_owned(),
                            validator_version: validator_info.validator_version,
                            violation_severity: (&violation).into(),
                            violation_message: match violation {
                                ValidationViolation::Major(msg) => msg,
                                ValidationViolation::Critical(msg) => msg,
                            },
                        });
                    }
                }
            }
        }
        vec
    }
}
