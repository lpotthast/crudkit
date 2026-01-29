//! Entity and aggregate validators for CRUD resources.
//!
//! Validators check entities for business rule violations. There are two types:
//! - `EntityValidator`: Validates individual entities or entity changes
//! - `AggregateValidator`: Validates aggregate-level constraints across all entities

use crate::prelude::{CrudResource, ValidationTrigger};
use async_trait::async_trait;
use crudkit_core::validation::violation::Violations;
use crudkit_core::validation::ViolationsByEntity;
use std::borrow::Cow;

/// Trait for validators that check a single entity or entity change.
///
/// Used for validation during CRUD operations. Each validator must provide
/// a name and version for tracking validation results.
///
/// # Validation Methods
///
/// - `validate_create`: Validates a CreateModel before insertion
/// - `validate_model`: Validates a Model (after insert, or existing entity)
/// - `validate_updated`: Validates an update by comparing old Model with new UpdateModel
///
/// # Example
///
/// ```ignore
/// struct ArticleTitleValidator;
///
/// impl Named for ArticleTitleValidator {
///     fn name(&self) -> Cow<'static, str> {
///         Cow::Borrowed("article_title")
///     }
/// }
///
/// impl EntityValidator<Article> for ArticleTitleValidator {
///     fn version(&self) -> u32 {
///         1
///     }
///
///     fn validate_create(
///         &self,
///         create_model: &ArticleCreateModel,
///         trigger: ValidationTrigger,
///     ) -> Violations {
///         let mut violations = Violations::empty();
///         if create_model.title.is_empty() {
///             violations.push(Violation::critical("Title cannot be empty"));
///         }
///         violations
///     }
/// }
/// ```
pub trait EntityValidator<R: CrudResource>: Send + Sync {
    /// Returns the unique name of this validator.
    fn name(&self) -> Cow<'static, str>;

    /// Returns the version of this validator.
    /// Increment when the validation logic changes to invalidate old results.
    fn version(&self) -> u32;

    /// Validate a CreateModel before insertion.
    ///
    /// Called during create operations before the entity is persisted.
    fn validate_create(
        &self,
        _create_model: &R::CreateModel,
        _trigger: ValidationTrigger,
    ) -> Violations {
        Violations::empty()
    }

    /// Validate an existing Model.
    ///
    /// Called after insert, or during delete operations.
    fn validate_model(&self, _model: &R::Model, _trigger: ValidationTrigger) -> Violations {
        Violations::empty()
    }

    /// Validate an update by comparing the old Model with the UpdateModel.
    ///
    /// Override this to implement delta validation that considers the change being made.
    /// By default, this returns no violations.
    ///
    // TODO: Is it ok to take UpdateModel? Previously, we would have defaulted to only validating the new state (using validate_model).
    //  But as we dont get the updated model, but only the update dto, this is no longer possible. Should we change this?
    fn validate_updated(
        &self,
        _old: &R::Model,
        _update: &R::UpdateModel,
        _trigger: ValidationTrigger,
    ) -> Violations {
        Violations::empty()
    }
}

/// Trait for validators that check aggregate-level constraints.
///
/// Used in global validation (which runs asynchronously after CRUD operations)
/// to validate constraints that span multiple entities.
///
/// # Example Use Cases
///
/// - Ensuring unique constraints across entities
/// - Validating referential integrity
/// - Checking aggregate totals or counts
#[async_trait]
pub trait AggregateValidator<R: CrudResource>: Send + Sync {
    /// Returns the unique name of this validator.
    fn name(&self) -> &'static str;

    /// Returns the version of this validator.
    fn version(&self) -> u32;

    /// Validate all entities of this resource type.
    /// Returns violations organized by entity ID.
    async fn validate_resource(&self) -> ViolationsByEntity<R::Id>;
}
