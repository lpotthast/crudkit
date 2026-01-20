use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use utoipa::ToSchema;

use crudkit_condition::Condition;
use crudkit_core::Saved;
use crudkit_id::Id;
use crudkit_resource::ResourceName;
use crudkit_validation::{
    PartialSerializableAggregateViolations, PartialSerializableValidations, ViolationsByEntity,
};

use crate::validate::{run_delta_validation, run_global_validation};
use crate::validation::{CrudAction, ValidationContext, ValidationTrigger, When};
use crate::{
    auth::RequestContext,
    collaboration,
    error::CrudError,
    lifetime::{CrudLifetime, UpdateRequest},
    prelude::*,
};

#[derive(Debug, ToSchema, Deserialize)]
pub struct UpdateOne<T> {
    pub condition: Option<Condition>,
    pub entity: T,
}

// TODO(stretch): UpdateMany? Supporting this would require a huge change for our leptos frontend. It would also require a `PartialUpdateModel`, allowing for unobserved / unchanged fields. This could(?) also be helpful as another UpdateOne variant, (more geared towards programmatic updates which dont want to read in the entity before).

#[tracing::instrument(level = "info", skip(context, request))]
pub async fn update_one<R: CrudResource>(
    request: RequestContext<R::Auth>,
    context: Arc<CrudContext<R>>,
    body: UpdateOne<R::UpdateModel>,
) -> Result<Saved<R::Model>, CrudError> {
    let model = context
        .repository
        .fetch_one(None, None, None, body.condition.as_ref())
        .await
        .map_err(|err| CrudError::Repository {
            reason: Arc::new(err),
        })?
        .ok_or(CrudError::NotFound)?;

    // Convert the model into an ActiveModel, allowing mutations.
    let mut active_model: R::ActiveModel = model.into();

    // Keep a copy of the old state for delta validation
    let old_active_model = active_model.clone();

    let update_model = body.entity;

    let update_request = UpdateRequest {
        condition: body.condition,
    };

    let hook_data = R::HookData::default();

    let hook_data = R::Lifetime::before_update(
        &update_model,
        &mut active_model,
        &update_request,
        &context.res_context,
        request.clone(),
        hook_data,
    )
    .await
    .map_err(CrudError::from)?;

    // Update the persisted active_model!
    active_model.update_with(update_model.clone()); // Clone required because we later reference the update_model in after_update. Could be optimized away when using NoopLifetimeHooks.

    // TODO: Just like model.get_id(), provide an active_model.get_id() implementation...?
    let entity_id = R::CrudColumn::get_id_active(&active_model)
        .expect("Updatable entities must be stored and therefor have an ID!");

    let serializable_id = entity_id.to_serializable_id();

    // Run delta validations comparing old and new state before updating the entity in the database.
    // This allows validators to check for violations related to the specific change being made.
    let trigger = ValidationTrigger::CrudAction(ValidationContext {
        action: CrudAction::Update,
        when: When::Before,
    });

    let mut partial_validation_results = run_delta_validation::<R>(
        &context.validators,
        &old_active_model,
        &active_model,
        trigger,
    );

    // Critical violations must block the save immediately. No persist/broadcast needed
    // since the update won't happen and the entity state hasn't changed.
    if partial_validation_results.has_critical_violations() {
        return Err(CrudError::CriticalValidationErrors {
            violations: PartialSerializableAggregateViolations::from(
                partial_validation_results,
                Some(serializable_id.clone()),
            ),
        });
    }

    // After this point, all critical violations are no longer of interest and can be dropped.
    partial_validation_results.drop_critical();

    // Update the entity using the user-provided repository impl.
    // Note: This might have unexpected effects on the data being saved. We shall load the entity
    //  again to make sure we do not miss any of them.
    let result = context
        .repository
        .update(active_model.clone())
        .await
        .map_err(|err| CrudError::Repository {
            reason: Arc::new(err),
        })?;

    // Delete all previously stored violations for this entity.
    // TODO: Should this be done in a transaction together with the later save, creating a safe swap?
    match R::CrudColumn::get_id_active(&active_model) {
        Ok(id) => {
            context
                .validation_result_repository
                .delete_all_of_entity(R::TYPE.name(), &id)
                .await
                .map_err(|err| CrudError::DeleteValidations {
                    reason: Arc::new(err),
                })?;
        }
        Err(err) => {
            tracing::error!(
                "Could not extract ID from active_model {active_model:?}. Error was: {err}"
            )
        }
    }

    // TODO: Should we validate again after the insert, catching potential changes made in the (unknown to us) repository (or even hooked up database)?
    // TODO: Follow up with a load op!

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

    // Now that the update succeeded, handle validation result persistence and broadcasting.
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

    // We read the entity again, to get an up-to-date instance of the "ReadModel".
    /*let read = build_select_query::<R::ReadViewEntity, R::ReadViewModel, R::ReadViewActiveModel, R::ReadViewColumn, R::ReadViewCrudColumn>(
        None,
        None,
        None,
        &body.condition,
    )?
    .one(controller.get_database_connection())
    .await
    .map_err(|err| CrudError::DbError(err.to_string()))?
    .ok_or_else(|| CrudError::ReadOneFoundNone)?;*/

    // Trigger global validation to check system-wide consistency.
    // Results are broadcast via WebSocket to all connected users.
    run_global_validation::<R>(&context).await;

    Ok(Saved {
        entity: result,
        violations: partial,
    })
}

// TODO: update_one_and_read_back() which updates and returns a ReadModel instead of an UpdateModel.
