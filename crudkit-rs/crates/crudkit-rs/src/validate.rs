//! Validation helpers for CRUD operations.
//!
//! These functions run entity validation using registered validators.

use crate::context::CrudContext;
use crate::prelude::{CrudResource, ValidationTrigger};
use crate::validator::EntityValidator;
use crudkit_core::validation::validator::ValidatorInfo;
use crudkit_core::validation::ViolationsByValidator;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

/// Run entity validation on a CreateModel using all registered validators.
pub fn run_entity_validation<R: CrudResource>(
    validators: &[Arc<dyn EntityValidator<R>>],
    create_model: &R::CreateModel,
    trigger: ValidationTrigger,
) -> ViolationsByValidator {
    let mut violations_by_validator = ViolationsByValidator::new();
    for validator in validators {
        violations_by_validator.extend(
            ValidatorInfo::new(validator.name(), validator.version()),
            validator.validate_create(create_model, trigger),
        );
    }
    violations_by_validator
}

/// Run entity validation on a Model using all registered validators.
pub fn run_model_validation<R: CrudResource>(
    validators: &[Arc<dyn EntityValidator<R>>],
    model: &R::Model,
    trigger: ValidationTrigger,
) -> ViolationsByValidator {
    let mut violations_by_validator = ViolationsByValidator::new();
    for validator in validators {
        violations_by_validator.extend(
            ValidatorInfo::new(validator.name(), validator.version()),
            validator.validate_model(model, trigger),
        );
    }
    violations_by_validator
}

/// Run delta validation using all registered validators.
/// Compares old Model state with new UpdateModel to detect violations.
pub fn run_delta_validation<R: CrudResource>(
    validators: &[Arc<dyn EntityValidator<R>>],
    old_model: &R::Model,
    update_model: &R::UpdateModel,
    trigger: ValidationTrigger,
) -> ViolationsByValidator {
    let mut violations_by_validator = ViolationsByValidator::new();
    for validator in validators {
        violations_by_validator.extend(
            ValidatorInfo::new(validator.name(), validator.version()),
            validator.validate_updated(old_model, update_model, trigger),
        );
    }
    violations_by_validator
}

/// State for debouncing global validation.
/// Ensures only one validation runs at a time, with at most one pending run.
#[derive(Debug)]
pub struct GlobalValidationState {
    /// State machine: IDLE (0), RUNNING (1), or RUNNING_WITH_PENDING (2).
    state: AtomicU8,
}

const VALIDATION_IDLE: u8 = 0;
const VALIDATION_RUNNING: u8 = 1;
const VALIDATION_RUNNING_WITH_PENDING: u8 = 2;

impl Default for GlobalValidationState {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalValidationState {
    pub fn new() -> Self {
        Self {
            state: AtomicU8::new(VALIDATION_IDLE),
        }
    }
}

/// Run global validation asynchronously with debounce-like behavior.
///
/// This function ensures efficient validation by:
/// - If validation is already running and another request comes in, it schedules one follow-up run
/// - If a run is already scheduled, additional requests are ignored
pub async fn run_global_validation<R: CrudResource>(context: &CrudContext<R>) {
    // Try to transition from IDLE to RUNNING.
    match context.global_validation_state.state.compare_exchange(
        VALIDATION_IDLE,
        VALIDATION_RUNNING,
        Ordering::AcqRel,
        Ordering::Acquire,
    ) {
        Ok(_) => {
            // Successfully acquired the running state, proceed with validation.
        }
        Err(current) => {
            // Either RUNNING or RUNNING_WITH_PENDING.
            if current == VALIDATION_RUNNING {
                // Try to schedule a follow-up run.
                let _ = context.global_validation_state.state.compare_exchange(
                    VALIDATION_RUNNING,
                    VALIDATION_RUNNING_WITH_PENDING,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );
            }
            // If already RUNNING_WITH_PENDING, nothing to do.
            return;
        }
    }

    // We now own the "running" state.
    loop {
        // Run validation (currently a placeholder).
        // TODO: Implement actual aggregate validation here.

        // Try to transition back to IDLE.
        match context.global_validation_state.state.compare_exchange(
            VALIDATION_RUNNING,
            VALIDATION_IDLE,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => break,
            Err(current) => {
                debug_assert_eq!(current, VALIDATION_RUNNING_WITH_PENDING);
                context
                    .global_validation_state
                    .state
                    .store(VALIDATION_RUNNING, Ordering::Release);
            }
        }
    }
}
