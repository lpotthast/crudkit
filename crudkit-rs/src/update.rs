//! Update operation for CRUD resources.

use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use utoipa::ToSchema;

use crudkit_core::condition::Condition;
use crudkit_core::Saved;
use crudkit_core::id::Id;
use crudkit_core::resource::ResourceName;
use crudkit_core::validation::{
    PartialSerializableAggregateViolations, PartialSerializableValidations, ViolationsByEntity,
};

use crate::data::CrudIdTrait;
use crate::validate::{run_delta_validation, run_global_validation};
use crate::validation::{CrudAction, ValidationContext, ValidationTrigger, When};
use crate::{
    auth::RequestContext,
    collaboration,
    error::CrudError,
    lifetime::{CrudLifetime, UpdateRequest},
    prelude::*,
};

/// Request body for updating a single entity.
#[derive(Debug, ToSchema, Deserialize)]
pub struct UpdateOne<T> {
    /// Condition to identify the entity to update.
    pub condition: Option<Condition>,
    /// The update data.
    pub entity: T,
}

/// Update a single entity.
///
/// # Flow
///
/// 1. Fetch the existing entity matching the condition
/// 2. Run `before_update` hook (can modify the update model)
/// 3. Run delta validation (comparing old and new state)
/// 4. If critical violations exist, return error
/// 5. Update entity via repository (repository applies changes internally)
/// 6. Delete old validation results
/// 7. Persist any new violations
/// 8. Broadcast validation results
/// 9. Run `after_update` hook
/// 10. Broadcast update event
/// 11. Trigger global validation
#[tracing::instrument(level = "info", skip(context, request))]
pub async fn update_one<R: CrudResource>(
    request: RequestContext<R::Auth>,
    context: Arc<CrudContext<R>>,
    body: UpdateOne<R::UpdateModel>,
) -> Result<Saved<R::Model>, CrudError> {
    // Fetch the existing entity.
    let existing_model = context
        .repository
        .fetch_one(None, None, None, body.condition.as_ref())
        .await
        .map_err(|err| CrudError::Repository {
            reason: Arc::new(err),
        })?
        .ok_or(CrudError::NotFound)?;

    // Keep a copy of the old state for delta validation.
    let old_model = existing_model.clone();

    let mut update_model = body.entity;

    let update_request = UpdateRequest {
        condition: body.condition,
    };

    let hook_data = R::HookData::default();

    // Run before_update hook - can modify the update_model.
    let hook_data = R::Lifetime::before_update(
        &existing_model,
        &mut update_model,
        &update_request,
        &context.res_context,
        request.clone(),
        hook_data,
    )
    .await
    .map_err(CrudError::from)?;

    // Get the entity ID before we move models.
    let entity_id = existing_model.id();
    let serializable_id = entity_id.to_serializable_id();

    // Run delta validations comparing old and new state before updating.
    let trigger = ValidationTrigger::CrudAction(ValidationContext {
        action: CrudAction::Update,
        when: When::Before,
    });

    let mut partial_validation_results =
        run_delta_validation::<R>(&context.validators, &old_model, &update_model, trigger);

    // Critical violations must block the save immediately.
    if partial_validation_results.has_critical_violations() {
        return Err(CrudError::CriticalValidationErrors {
            violations: PartialSerializableAggregateViolations::from(
                partial_validation_results,
                Some(serializable_id.clone()),
            ),
        });
    }

    // After this point, all critical violations are no longer of interest.
    partial_validation_results.drop_critical();

    // Update the entity via the repository.
    // The repository handles applying the UpdateModel to the existing Model internally.
    let result = context
        .repository
        .update(existing_model, update_model.clone())
        .await
        .map_err(|err| CrudError::Repository {
            reason: Arc::new(err),
        })?;

    // Delete all previously stored violations for this entity.
    context
        .validation_result_repository
        .delete_all_of_entity(R::TYPE.name(), &entity_id)
        .await
        .map_err(|err| CrudError::DeleteValidations {
            reason: Arc::new(err),
        })?;

    let has_violations = partial_validation_results.has_violations();

    if has_violations {
        // Persist the validation results for later access/use.
        context
            .validation_result_repository
            .save_all(
                R::TYPE.name(),
                ViolationsByEntity::of_entity_violations(
                    entity_id.clone(),
                    partial_validation_results.clone(),
                ),
            )
            .await
            .map_err(|err| CrudError::SaveValidations {
                reason: Arc::new(err),
            })?;
    }

    // Build the partial validation result for response and broadcast.
    let partial = PartialSerializableAggregateViolations::from(
        partial_validation_results,
        Some(serializable_id.clone()),
    );

    let partial_serializable_validations: PartialSerializableValidations =
        HashMap::from([(ResourceName::from(R::TYPE.name()), partial.clone())]);

    // Broadcast the PARTIAL validation result to all registered WebSocket connections.
    // When empty: Entity known to be valid (again) by listeners.
    // When not empty: Entity known to be invalid (again) by listeners.
    collaboration::broadcast_partial_validation_result(&context, partial_serializable_validations)
        .await;

    // Run after_update hook.
    let _hook_data = R::Lifetime::after_update(
        &update_model,
        &result,
        &update_request,
        &context.res_context,
        request,
        hook_data,
    )
    .await
    .map_err(CrudError::from)?;

    // Inform all users that the entity was updated.
    collaboration::broadcast_updated_event(&context, serializable_id, has_violations).await;

    // Trigger global validation to check system-wide consistency.
    run_global_validation::<R>(&context).await;

    Ok(Saved {
        entity: result,
        violations: partial,
    })
}
