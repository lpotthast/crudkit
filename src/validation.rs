use std::collections::HashMap;

use async_trait::async_trait;
use crud_shared_types::validation::{
    EntityValidations, StrictEntityInfo, StrictOwnedEntityInfo, ValidationViolation, ValidatorInfo,
};
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(16))")]
pub enum ValidationViolationType {
    #[sea_orm(string_value = "MAJOR")]
    Major,
    #[sea_orm(string_value = "CRITICAL")]
    Critical,
}

pub trait ValidationViolationTypeExt {
    fn is_of_type(&self, violation_type: ValidationViolationType) -> bool;
}

impl ValidationViolationTypeExt for ValidationViolation {
    fn is_of_type(&self, violation_type: ValidationViolationType) -> bool {
        match self {
            ValidationViolation::Major(_) => violation_type == ValidationViolationType::Major,
            ValidationViolation::Critical(_) => violation_type == ValidationViolationType::Critical,
        }
    }
}

impl From<ValidationViolation> for ValidationViolationType {
    fn from(val: ValidationViolation) -> Self {
        match val {
            ValidationViolation::Major(_) => ValidationViolationType::Major,
            ValidationViolation::Critical(_) => ValidationViolationType::Critical,
        }
    }
}

impl From<&ValidationViolation> for ValidationViolationType {
    fn from(val: &ValidationViolation) -> Self {
        match val {
            ValidationViolation::Major(_) => ValidationViolationType::Major,
            ValidationViolation::Critical(_) => ValidationViolationType::Critical,
        }
    }
}

pub trait EntityValidationsExt {
    fn has_violation_of_type(&self, violation_type: ValidationViolationType) -> bool;
}

impl EntityValidationsExt for EntityValidations {
    fn has_violation_of_type(&self, violation_type: ValidationViolationType) -> bool {
        for validator_violations in self.entity_violations.values() {
            for violations in validator_violations.values() {
                for violation in &violations.violations {
                    if violation.is_of_type(violation_type) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[async_trait]
pub trait ValidationResultSaverTrait {
    async fn delete_for(&self, entity_info: StrictOwnedEntityInfo);

    async fn save_all(
        &self,
        validation_results: HashMap<
            StrictEntityInfo,
            HashMap<ValidatorInfo, Vec<PersistableViolation>>,
        >,
    );
}

pub struct PersistableViolation {
    pub violation_severity: ValidationViolationType,
    pub violation_message: String,
}

/// Removes critical validations and validations without an id.
/// TODO: Add test
pub fn into_persistable(data: EntityValidations) -> HashMap<StrictEntityInfo, HashMap<ValidatorInfo, Vec<PersistableViolation>>> {
    let mut entity_violations = HashMap::with_capacity(data.entity_violations.len());
    for (entity_info, validators) in data.entity_violations {
        if let Some(entity_id) = entity_info.entity_id {
            let mut validator_validations = HashMap::with_capacity(validators.len());
            for (validator_info, violations) in validators {
                let mut vec = Vec::with_capacity(violations.violations.len());
                for violation in violations.violations {
                    match violation {
                        ValidationViolation::Major(msg) => vec.push(PersistableViolation {
                            violation_severity: ValidationViolationType::Major,
                            violation_message: msg,
                        }),
                        ValidationViolation::Critical(_) => {
                            // skip critical violations, as they should not be persisted.
                        }
                    };
                }
                validator_validations.insert(validator_info, vec);
            }
            entity_violations.insert(
                StrictEntityInfo {
                    entity_name: entity_info.entity_name,
                    entity_id,
                },
                validator_validations,
            );
        }
    }
    entity_violations
}
