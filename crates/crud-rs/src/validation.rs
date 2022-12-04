use std::collections::HashMap;

use async_trait::async_trait;
use crud_shared_types::{validation::{
    EntityViolations, ValidationViolation, ValidatorInfo,
    Violations, OwnedValidatorInfo,
}, id::Id};
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrudAction {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum When {
    Before,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationContext {
    /// The CRUD action that lead to the validation.
    pub action: CrudAction,
    /// Wether or not the validation occurs before or after applying the CRUD action.
    /// Critical violations created before the action is applied will prevent its application.
    pub when: When,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationTrigger {
    CrudAction(ValidationContext),
    GlobalValidation,
}

// TODO: Implement in resources!
pub trait AggregateValidator {
    /// Aggregate validation has no access to a trigger or context, as aggregate validation is only applied through global validation.
    fn validate(&self) -> Violations;
}

// TODO: delete?
pub trait EntityValidatorTrait<T, I: Id> {
    fn validate_single(&self, entity: &T, trigger: ValidationTrigger) -> EntityViolations<I>;
    fn validate_updated(&self, old: &T, new: &T, trigger: ValidationTrigger) -> EntityViolations<I>;
    fn get_name(&self) -> &'static str;
    fn get_version(&self) -> u32;
}

pub trait EntityValidatorsTrait<T, I: Id + Clone> {
    fn validate_single(&self, entity: &T, trigger: ValidationTrigger) -> EntityViolations<I>;
    fn validate_updated(&self, old: &T, new: &T, trigger: ValidationTrigger) -> EntityViolations<I>;
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

impl<I: Id> EntityValidationsExt for EntityViolations<I> {
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
pub trait ValidationResultSaverTrait<I: Id> {
    async fn delete_all_for(&self, entity_id: &I);

    async fn save_all(&self, validation_results: HashMap<I, HashMap<ValidatorInfo, Vec<PersistableViolation>>>);

    async fn list_all(&self) -> HashMap<I, HashMap<OwnedValidatorInfo, Vec<ValidationViolation>>>;
}

pub struct PersistableViolation {
    pub violation_severity: ValidationViolationType,
    pub violation_message: String,
}

/// Removes critical validations and validations without an id.
/// TODO: Add test
/// TODO: Convert SerializableId to String right here, not later
pub fn into_persistable<I: Id>(data: EntityViolations<I>) -> HashMap<I, HashMap<ValidatorInfo, Vec<PersistableViolation>>> {
    let mut entity_violations = HashMap::with_capacity(data.entity_violations.len());
    for (entity_id, validators) in data.entity_violations {
        if let Some(entity_id) = entity_id {
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
            entity_violations.insert(entity_id, validator_validations);
        }
    }
    entity_violations
}
