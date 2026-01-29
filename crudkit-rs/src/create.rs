//! Create operation for CRUD resources.

use crate::{
    auth::RequestContext,
    collaboration,
    data::CrudIdTrait,
    error::CrudError,
    lifetime::CrudLifetime,
    prelude::*,
    validate::{run_entity_validation, run_global_validation, run_model_validation},
    validation::{CrudAction, ValidationContext, ValidationTrigger, When},
};

use crudkit_core::Saved;
use crudkit_core::id::Id;
use crudkit_core::resource::ResourceName;
use crudkit_core::validation::violation::Violations;
use crudkit_core::validation::{
    PartialSerializableAggregateViolations, PartialSerializableValidations, ViolationsByEntity,
};

use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use utoipa::ToSchema;

/// Request body for creating a single entity.
#[derive(Debug, ToSchema, Deserialize)]
pub struct CreateOne<T> {
    /// The entity data to create.
    pub entity: T,
}

/// Create a single entity.
///
/// # Flow
///
/// 1. Run `before_create` hook (can modify the create model)
/// 2. Run pre-insert validation
/// 3. If critical violations exist, return error
/// 4. Insert entity via repository
/// 5. Run `after_create` hook
/// 6. Run post-insert validation
/// 7. Persist and broadcast any violations
/// 8. Broadcast creation event
/// 9. Trigger global validation
#[tracing::instrument(level = "info", skip(context, request))]
pub async fn create_one<R: CrudResource>(
    request: RequestContext<R::Auth>,
    context: Arc<CrudContext<R>>,
    body: CreateOne<R::CreateModel>,
) -> Result<Saved<R::Model>, CrudError> {
    let mut create_model: R::CreateModel = body.entity;

    let hook_data = R::HookData::default();

    // Run before_create hook - can modify the create_model.
    let hook_data = R::Lifetime::before_create(
        &mut create_model,
        &context.res_context,
        request.clone(),
        hook_data,
    )
    .await
    .map_err(CrudError::from)?;

    // Run validations before inserting the entity.
    let trigger = ValidationTrigger::CrudAction(ValidationContext {
        action: CrudAction::Create,
        when: When::Before,
    });

    // TODO: Should this be done BEFORE or AFTER running the `before_create` hook?
    let violations_by_validator =
        run_entity_validation::<R>(&context.validators, &create_model, trigger);

    if violations_by_validator.has_critical_violations() {
        // Critical validation errors are returned synchronously in the HTTP response.
        return Err(CrudError::CriticalValidationErrors {
            // NOTE: All violations created here do not have an ID, as the entity was not yet saved.
            violations: PartialSerializableAggregateViolations::from(violations_by_validator, None),
        });
    }

    // Clone for use in after_create hook (original may be moved to repository).
    let create_model_clone = create_model.clone();

    // Insert the entity via the repository.
    // The repository handles the conversion to storage-specific types internally.
    let inserted_entity: R::Model =
        context
            .repository
            .insert(create_model)
            .await
            .map_err(|err| CrudError::Repository {
                reason: Arc::new(err),
            })?;

    // Run after_create hook.
    let _hook_data = R::Lifetime::after_create(
        &create_model_clone,
        &inserted_entity,
        &context.res_context,
        request,
        hook_data,
    )
    .await
    .map_err(CrudError::from)?;

    let entity_id = inserted_entity.id();
    let serializable_id = entity_id.to_serializable_id();

    // Reevaluate the entity for violations and broadcast all of them if some exist.
    let trigger = ValidationTrigger::CrudAction(ValidationContext {
        action: CrudAction::Create,
        when: When::After,
    });

    let violations_by_validator =
        run_model_validation::<R>(&context.validators, &inserted_entity, trigger);

    let has_violations = violations_by_validator.has_violations();

    if has_violations {
        // Persist the validation results for later access/use.
        context
            .validation_result_repository
            .save_all(
                R::TYPE.name(),
                ViolationsByEntity::of_entity_violations(
                    entity_id,
                    violations_by_validator.clone(),
                ),
            )
            .await
            .map_err(|err| CrudError::SaveValidations {
                reason: Arc::new(err),
            })?;
    }

    let partial = PartialSerializableAggregateViolations::from(
        violations_by_validator,
        Some(serializable_id.clone()),
    );

    if has_violations {
        // Broadcast the PARTIAL validation result to all registered WebSocket connections.
        // We successfully created the entry now.
        // To delete any leftover "create" violations in the frontend, set create to Some empty vector.
        let mut violations = partial.clone();
        violations.create = Some(Violations::empty());

        let partial_serializable_validations: PartialSerializableValidations =
            HashMap::from([(ResourceName::from(R::TYPE.name()), violations)]);

        collaboration::broadcast_partial_validation_result(
            &context,
            partial_serializable_validations,
        )
        .await;
    }

    // Inform all users that the entity was created.
    collaboration::broadcast_creation_event(&context, serializable_id, has_violations).await;

    // Trigger global validation to check system-wide consistency.
    run_global_validation::<R>(&context).await;

    Ok(Saved {
        entity: inserted_entity,
        violations: partial,
    })
}
