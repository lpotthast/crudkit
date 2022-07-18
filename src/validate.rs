use crud_shared_types::validation::{EntityViolations, ValidationViolation};
use sea_orm::ActiveValue;

use crate::validation::ValidationViolationType;

pub fn validate_required<T: Into<sea_orm::Value>>(
    name: &'static str,
    val: &ActiveValue<T>,
    violation_type: ValidationViolationType,
    result: &mut EntityViolations,
) {
    match val {
        ActiveValue::NotSet => {
            let err = format!(
                "Field {name} not set but required!"
            );
            result.push(match violation_type {
                ValidationViolationType::Major => ValidationViolation::Critical(err),
                ValidationViolationType::Critical => ValidationViolation::Critical(err),
            });
        },
        _ => {}
    }
}

pub fn validate_not_empty(
    name: &'static str,
    val: &ActiveValue<String>,
    result: &mut EntityViolations,
) {
    match val {
        ActiveValue::Set(v) | ActiveValue::Unchanged(v) => {
            if v.len() == 0 {
                result.push(ValidationViolation::Major(format!("Field {name} with value \"{v}\" must not be empty!")));
            }
        }
        _ => {}
    }
}

pub fn validate_min_length(
    name: &'static str,
    val: &ActiveValue<String>,
    min_len: usize,
    result: &mut EntityViolations,
) {
    match val {
        ActiveValue::Set(v) | ActiveValue::Unchanged(v) => {
            if v.len() < min_len {
                result.push(ValidationViolation::Major(format!("Field '{name}' with value '{v}' does not meet length requirement: length is {}, min_length: {min_len}!", v.len())));
            }
        }
        _ => {}
    }
}

pub fn validate_max_length(
    name: &'static str,
    val: &ActiveValue<String>,
    max_len: usize,
    result: &mut EntityViolations,
) {
    match val {
        ActiveValue::Set(v) | ActiveValue::Unchanged(v) => {
            if v.len() > max_len {
                result.push(ValidationViolation::Major(format!("Field {name} with value \"{v}\" does not meet length requirement: length is {}, max_length: {max_len}!", v.len())));
            }
        }
        _ => {}
    }
}
