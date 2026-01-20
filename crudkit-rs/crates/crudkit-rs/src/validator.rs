use crate::prelude::{CrudResource, ValidationTrigger};
use async_trait::async_trait;
use crudkit_validation::violation::Violations;
use crudkit_validation::ViolationsByEntity;
use std::borrow::Cow;

/// Trait for validators that check a single entity or entity change.
/// Used for adhoc validation during CRUD operations.
///
/// Each validator must provide a name and version for tracking validation results.
/// This allows the system to detect obsolete results when validators are updated.
pub trait EntityValidator<R: CrudResource>: Send + Sync {
    /// Returns the unique name of this validator.
    fn get_name(&self) -> Cow<'static, str>;

    /// Returns the version of this validator.
    /// Increment when the validation logic changes to invalidate old results.
    fn get_version(&self) -> u32;

    /// Each validation attempt may report multiple errors.
    /// Between each validation attempt, previously reported violations are forgotten.
    fn validate_single(&self, entity: &R::ActiveModel, trigger: ValidationTrigger) -> Violations;

    /// Defaults to validation of the new state using `validate_single`. Override this if you
    /// need to look at the diff of a change to drive the validation.
    fn validate_updated(
        &self,
        _old: &R::ActiveModel,
        new: &R::ActiveModel,
        trigger: ValidationTrigger,
    ) -> Violations {
        self.validate_single(new, trigger)
    }
}

/// Trait for additional validators that validate an entire resource type.
///
/// Used in addition to `EntityValidator`s in global validation
/// (which runs asynchronously after CRUD operations).
///
/// Each validator must provide a name and version for tracking validation results.
// TODO: Add a (good) example when this would be useful: For example "number of people" must be equal. (probably a weak example though...)
#[async_trait]
pub trait AggregateValidator<R: CrudResource>: Send + Sync {
    /// Returns the unique name of this validator.
    fn get_name(&self) -> &'static str;

    /// Returns the version of this validator.
    fn get_version(&self) -> u32;

    /// Validate all entities of this resource type.
    /// Returns violations organized by entity ID.
    async fn validate_resource(&self) -> ViolationsByEntity<R::Id>;
}
