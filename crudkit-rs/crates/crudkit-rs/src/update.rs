use axum_keycloak_auth::{decode::KeycloakToken, role::Role};
use serde::Deserialize;
use snafu::{Backtrace, GenerateImplicitData};
use std::{collections::HashMap, sync::Arc};
use tracing::error;
use utoipa::ToSchema;

use crudkit_condition::Condition;
use crudkit_id::Id;
use crudkit_shared::{SaveResult, Saved};
use crudkit_validation::PartialSerializableValidations;
use crudkit_websocket::{CkWsMessage, EntityUpdated};

use crate::{
    error::CrudError,
    lifetime::{Abort, CrudLifetime},
    prelude::*,
    validation::{CrudAction, ValidationContext, ValidationTrigger, When, into_persistable},
};

#[derive(Debug, ToSchema, Deserialize)]
pub struct UpdateOne<T> {
    pub condition: Option<Condition>,
    pub entity: T,
}

#[tracing::instrument(level = "info", skip(context))]
pub async fn update_one<R: CrudResource, Ro: Role>(
    keycloak_token: KeycloakToken<Ro>,
    context: Arc<CrudContext<R>>,
    body: UpdateOne<R::UpdateModel>,
) -> Result<SaveResult<R::Model>, CrudError> {
    let model = context
        .repository
        .fetch_one(None, None, None, body.condition.as_ref())
        .await
        .map_err(|err| CrudError::Repository {
            reason: Arc::new(err),
            backtrace: Backtrace::generate(),
        })?
        .ok_or_else(|| CrudError::ReadOneFoundNone {
            backtrace: Backtrace::generate(),
        })?;

    // Convert the model into an ActiveModel, allowing mutations.
    let mut active_model: R::ActiveModel = model.into();

    let update_model = body.entity;

    let hook_data = R::HookData::default();

    // Before update
    let (abort, hook_data) = R::Lifetime::before_update(
        &update_model,
        &mut active_model,
        &context.res_context,
        hook_data,
    )
    .await
    .expect("before_update to not error");

    if let Abort::Yes { reason } = abort {
        return Ok(SaveResult::Aborted { reason });
    }

    // Update the persisted active_model!
    active_model.update_with(update_model.clone()); // Clone required becase we later reference the update_model in after_update. Could be optimized away when using NoopLifetimeHooks.

    // TODO: Just like model.get_id(), provide an active_model.get_id() implementation...?
    let entity_id = R::CrudColumn::get_id_active(&active_model)
        .expect("Updatable entities must be stored and therefor have an ID!");

    let serializable_id = entity_id.into_serializable_id();

    // Run validations ON THE NEW STATE(!) but before updating the entity in the database.
    let trigger = ValidationTrigger::CrudAction(ValidationContext {
        action: CrudAction::Update,
        when: When::Before,
    });

    let partial_validation_results = context.validator.validate_single(&active_model, trigger);

    let has_violations = partial_validation_results.has_violations();

    if has_violations {
        let has_critical_violations =
            partial_validation_results.has_violation_of_type(ValidationViolationType::Critical);

        // Broadcast the PARTIAL validation result to all registered WebSocket connections.
        let partial_serializable_validations: PartialSerializableValidations = HashMap::from([(
            String::from(R::TYPE.into()),
            partial_validation_results.clone().into(),
        )]);

        context
            .ws_controller
            .broadcast_json(CkWsMessage::PartialValidationResult(
                partial_serializable_validations,
            ));

        // Persist the validation results for later access/use.
        let persistable = into_persistable(partial_validation_results);
        context
            .validation_result_repository
            .save_all(persistable)
            .await
            .map_err(|err| CrudError::SaveValidations {
                reason: Arc::new(err),
                backtrace: Backtrace::generate(),
            })?;

        // The existence of CRITICAL violations must block a save!
        if has_critical_violations {
            // SAFETY: Calling unwrap is safe, as the if above assigns a Some variant and runs with the same condition as this code.
            return Ok(SaveResult::CriticalValidationErrors);
        }
    } else {
        // We know that the entity is valid and therefor need to delete all previously stored violations for this entity.
        // The active_model might not have an id, thou that is unlikely when doing an "update".
        match R::CrudColumn::get_id_active(&active_model) {
            Ok(id) => {
                context
                    .validation_result_repository
                    .delete_all_for(&id)
                    .await
                    .map_err(|err| CrudError::DeleteValidations {
                        reason: Arc::new(err),
                        backtrace: Backtrace::generate(),
                    })?;
            }
            Err(err) => {
                error!("Could not extract ID from active_model {active_model:?}. Error was: {err}")
            }
        }

        // Inform the websocket listeners.
        let partial_serializable_validations: PartialSerializableValidations = HashMap::from([(
            String::from(R::TYPE.into()),
            partial_validation_results.clone().into(),
        )]);

        context
            .ws_controller
            .broadcast_json(CkWsMessage::PartialValidationResult(
                partial_serializable_validations,
            ));
    }

    // Update the entry.
    let result = context
        .repository
        .update(active_model)
        .await
        .map_err(|err| CrudError::Repository {
            reason: Arc::new(err),
            backtrace: Backtrace::generate(),
        })?;

    // After update
    let _hook_data =
        R::Lifetime::after_update(&update_model, &result, &context.res_context, hook_data)
            .await
            .expect("after_update to not error");

    // Inform all participants that the entity was updated.
    // TODO: Exclude the current user!
    context
        .ws_controller
        .broadcast_json(CkWsMessage::EntityUpdated(EntityUpdated {
            aggregate_name: R::TYPE.into().to_owned(),
            entity_id: serializable_id,
            with_validation_errors: has_violations,
        }));

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
    Ok(SaveResult::Saved(Saved {
        entity: result,
        with_validation_errors: has_violations,
    }))
}

// TODO: update_one_and_read_back() which updates and returns a ReadModel instead of an UpdateModel.
