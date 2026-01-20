use crate::context::CrudContext;
use crate::prelude::{CrudResource, ValidationTrigger};
use crate::validator::EntityValidator;
use crudkit_validation::validator::ValidatorInfo;
use crudkit_validation::ViolationsByValidator;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

/// Run entity validation using all registered validators.
/// This is a helper function to reduce code duplication across CRUD operations.
pub fn run_entity_validation<R: CrudResource>(
    validators: &[Arc<dyn EntityValidator<R>>],
    active_model: &R::ActiveModel,
    trigger: ValidationTrigger,
) -> ViolationsByValidator {
    let mut violations_by_validator = ViolationsByValidator::new();
    for validator in validators {
        violations_by_validator.extend(
            ValidatorInfo::new(validator.get_name(), validator.get_version()),
            validator.validate_single(active_model, trigger),
        );
    }
    violations_by_validator
}

/// Run delta validation using all registered validators.
/// Compares old and new state to detect violations related to the change.
pub fn run_delta_validation<R: CrudResource>(
    validators: &[Arc<dyn EntityValidator<R>>],
    old_active_model: &R::ActiveModel,
    new_active_model: &R::ActiveModel,
    trigger: ValidationTrigger,
) -> ViolationsByValidator {
    let mut violations_by_validator = ViolationsByValidator::new();
    for validator in validators {
        violations_by_validator.extend(
            ValidatorInfo::new(validator.get_name(), validator.get_version()),
            validator.validate_updated(old_active_model, new_active_model, trigger),
        );
    }
    violations_by_validator
}

/// State for debouncing global validation.
/// Ensures only one validation runs at a time, with at most one pending run.
///
/// Uses a single atomic state to avoid race conditions between separate flags.
#[derive(Debug)]
pub struct GlobalValidationState {
    /// State machine: IDLE (0), RUNNING (1), or RUNNING_WITH_PENDING (2).
    state: AtomicU8,
}

/// No validation is running.
const VALIDATION_IDLE: u8 = 0;
/// Validation is currently running.
const VALIDATION_RUNNING: u8 = 1;
/// Validation is running and another run is scheduled to follow.
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
/// - If a run is already scheduled, additional requests are ignored (the scheduled run will see all changes)
///
/// The state machine ensures atomicity: the decision to exit and state transition happen together,
/// preventing race conditions where a scheduled run could be lost.
///
/// Called after CRUD operations complete to check system-wide consistency.
/// Results are persisted to the validation repository and broadcast via WebSocket.
// TODO: global validation should validate ALL resoures at once. We cannot do that here...
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
            // If already RUNNING_WITH_PENDING, nothing to do - a follow-up is already scheduled.
            return;
        }
    }

    // We now own the "running" state.
    loop {
        // Run validation.
        //let mut violations_by_entity = ViolationsByEntity::new();

        // TODO: Also run standard EntityValidator's in global validation!

        // Run all resource validators in sequence.
        // TODO: This could be done in parallel.
        //for validator in &context.resource_validators {
        //    violations_by_entity.merge(
        //        ValidatorInfo::new(validator.get_name(), validator.get_version()),
        //        validator.validate_resource().await,
        //    );
        //}

        //if violations_by_entity.has_violations() {
        //    let v = ViolationsByEntity::of_entity_violations(enity_id, violations_by_entity);
        //
        //    // Persist the validation results.
        //    if let Err(e) = context
        //        .validation_result_repository
        //        .save_all(R::TYPE.name(), violations_by_entity)
        //        .await
        //    {
        //        tracing::warn!("Failed to persist global validation results: {e:?}");
        //    }
        //
        //    // Broadcast via WebSocket.
        //    let partial_serializable_validations: PartialSerializableValidations =
        //        HashMap::from([(String::from(R::TYPE.name()), violations_by_entity.into())]);
        //
        //    broadcast_full_validation_result(context, partial_serializable_validations).await;
        //}

        // Try to transition back to IDLE.
        // If someone requested another run (set RUNNING_WITH_PENDING), the CAS fails and we loop.
        match context.global_validation_state.state.compare_exchange(
            VALIDATION_RUNNING,
            VALIDATION_IDLE,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => break, // Successfully transitioned to IDLE, we're done.
            Err(current) => {
                // Someone set RUNNING_WITH_PENDING while we were running.
                debug_assert_eq!(current, VALIDATION_RUNNING_WITH_PENDING);
                // Reset to RUNNING so new requests can set RUNNING_WITH_PENDING again, then loop.
                context
                    .global_validation_state
                    .state
                    .store(VALIDATION_RUNNING, Ordering::Release);
            }
        }
    }
}
