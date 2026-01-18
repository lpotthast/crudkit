use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use async_trait::async_trait;
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crudkit_collaboration::CollabMessage;
use crudkit_id::Id;
use crudkit_validation::{
    EntityViolations, OwnedValidatorInfo, PartialSerializableValidations, ValidationViolation,
    ValidatorInfo,
};

use crate::{
    collaboration::CollaborationService, context::CrudContext, repository::RepositoryError,
    resource::CrudResource,
};

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
    /// Whether or not the validation occurs before or after applying the CRUD action.
    /// Critical violations created before the action is applied will prevent its application.
    pub when: When,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationTrigger {
    CrudAction(ValidationContext),
    GlobalValidation,
}

/// Trait for validators that check a single entity or entity change.
/// Used for adhoc validation during CRUD operations.
///
/// Each validator must provide a name and version for tracking validation results.
/// This allows the system to detect obsolete results when validators are updated.
pub trait EntityValidator<R: CrudResource>: Send + Sync {
    /// Returns the unique name of this validator.
    fn get_name(&self) -> &'static str;

    /// Returns the version of this validator.
    /// Increment when the validation logic changes to invalidate old results.
    fn get_version(&self) -> u32;

    fn validate_single(
        &self,
        entity: &R::ActiveModel,
        trigger: ValidationTrigger,
    ) -> EntityViolations<R::Id>;

    fn validate_updated(
        &self,
        old: &R::ActiveModel,
        new: &R::ActiveModel,
        trigger: ValidationTrigger,
    ) -> EntityViolations<R::Id>;
}

/// Trait for validators that check all entities of a resource type.
/// Used for global validation that runs asynchronously after CRUD operations.
///
/// Each validator must provide a name and version for tracking validation results.
#[async_trait]
pub trait AggregateValidator<R: CrudResource>: Send + Sync {
    /// Returns the unique name of this validator.
    fn get_name(&self) -> &'static str;

    /// Returns the version of this validator.
    fn get_version(&self) -> u32;

    /// Validate all entities of this resource type.
    /// Returns violations organized by entity ID.
    async fn validate_all(&self) -> EntityViolations<R::Id>;
}

/// No-op aggregate validator for resources that don't need global validation.
pub struct NoAggregateValidator;

#[async_trait]
impl<R: CrudResource> AggregateValidator<R> for NoAggregateValidator {
    fn get_name(&self) -> &'static str {
        "NoAggregate"
    }

    fn get_version(&self) -> u32 {
        1
    }

    async fn validate_all(&self) -> EntityViolations<R::Id> {
        EntityViolations::empty()
    }
}

/// Run entity validation using all registered validators.
/// This is a helper function to reduce code duplication across CRUD operations.
pub fn run_entity_validation<R: CrudResource>(
    validators: &[Arc<dyn EntityValidator<R>>],
    active_model: &R::ActiveModel,
    trigger: ValidationTrigger,
) -> EntityViolations<R::Id> {
    let mut results = EntityViolations::empty();
    for validator in validators {
        results.merge(validator.validate_single(active_model, trigger));
    }
    results
}

/// Run delta validation using all registered validators.
/// Compares old and new state to detect violations related to the change.
pub fn run_delta_validation<R: CrudResource>(
    validators: &[Arc<dyn EntityValidator<R>>],
    old_active_model: &R::ActiveModel,
    new_active_model: &R::ActiveModel,
    trigger: ValidationTrigger,
) -> EntityViolations<R::Id> {
    let mut results = EntityViolations::empty();
    for validator in validators {
        results.merge(validator.validate_updated(old_active_model, new_active_model, trigger));
    }
    results
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
        let mut validation_results = EntityViolations::empty();
        for validator in &context.aggregate_validators {
            validation_results.merge(validator.validate_all().await);
        }

        if validation_results.has_violations() {
            // Persist the validation results.
            let persistable = into_persistable(validation_results.clone());
            if let Err(e) = context
                .validation_result_repository
                .save_all(persistable)
                .await
            {
                tracing::warn!("Failed to persist global validation results: {e:?}");
            }

            // Broadcast via WebSocket.
            let partial_serializable_validations: PartialSerializableValidations =
                HashMap::from([(String::from(R::TYPE.into()), validation_results.into())]);

            broadcast_partial_validation_result(context, partial_serializable_validations).await;
        }

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

pub(crate) async fn broadcast_partial_validation_result<R: CrudResource>(
    context: &CrudContext<R>,
    partial_serializable_validations: PartialSerializableValidations,
) {
    if let Err(err) = context
        .collab_service
        .broadcast_json(CollabMessage::PartialValidationResult(
            partial_serializable_validations,
        ))
        .await
    {
        tracing::warn!("Failed to broadcast global validation result: {err:?}");
    }
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

/// Trait for persisting validation results to a storage backend.
#[async_trait]
pub trait ValidationResultSaver<I: Id> {
    type Error: RepositoryError + 'static;

    async fn delete_all_for(&self, entity_id: &I) -> Result<(), Self::Error>;

    async fn save_all(
        &self,
        validation_results: HashMap<I, HashMap<ValidatorInfo, Vec<PersistableViolation>>>,
    ) -> Result<(), Self::Error>;

    async fn list_all(
        &self,
    ) -> Result<HashMap<I, HashMap<OwnedValidatorInfo, Vec<ValidationViolation>>>, Self::Error>;
}

pub struct PersistableViolation {
    pub violation_severity: ValidationViolationType,
    pub violation_message: String,
}

/// Removes critical validations and validations without an id.
/// TODO: Add test
/// TODO: Convert SerializableId to String right here, not later
pub fn into_persistable<I: Id>(
    data: EntityViolations<I>,
) -> HashMap<I, HashMap<ValidatorInfo, Vec<PersistableViolation>>> {
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

pub struct NoopValidationResultRepository;

#[derive(Debug, Snafu)]
pub enum NoopError {}

impl RepositoryError for NoopError {}

#[async_trait]
impl<I: Id + Clone + Send + Sync + 'static> ValidationResultSaver<I>
    for NoopValidationResultRepository
{
    type Error = NoopError;

    async fn delete_all_for(&self, _entity_id: &I) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn save_all(
        &self,
        _validation_results: HashMap<I, HashMap<ValidatorInfo, Vec<PersistableViolation>>>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn list_all(
        &self,
    ) -> Result<HashMap<I, HashMap<OwnedValidatorInfo, Vec<ValidationViolation>>>, Self::Error>
    {
        Ok(HashMap::new())
    }
}
