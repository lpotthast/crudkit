use crate::{prelude::*, validation::into_persistable};
use crud_shared_types::{
    validation::{EntityViolations, StrictOwnedEntityInfo},
    ws_messages::{CrudWsMessage, EntityUpdated},
    Condition, CrudError, SaveResult, Saved,
};
use sea_orm::ActiveModelTrait;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct UpdateOne {
    pub condition: Option<Condition>,
    pub entity: Box<serde_json::value::RawValue>,
}

pub async fn update_one<R: CrudResource>(
    controller: Arc<CrudController>,
    context: Arc<CrudContext<R>>,
    body: UpdateOne,
) -> Result<SaveResult<R::Model>, CrudError> {
    let model =
        build_select_query::<R::Entity, R::Model, R::ActiveModel, R::Column, R::CrudColumn>(
            None,
            None,
            None,
            &body.condition,
        )?
        .one(controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?
        .ok_or(CrudError::ReadOneFoundNone)?;

    // Use the "CreateModel" to deserialize the given JSON. Some not required members are allowed to be missing.
    let update = serde_json::from_str::<R::CreateModel>(body.entity.get()).map_err(|err| {
        CrudError::UnableToParseAsEntity(body.entity.get().to_owned(), err.to_string())
    })?;

    // Convert the model into an ActiveModel, allowing mutations.
    let mut active_model: R::ActiveModel = model.into();

    // Update the persisted active_model!
    active_model.update_with(update);
    let entity_id = R::CrudColumn::get_id(&active_model).expect("Updatable entities must be stored and therefor have an ID!");

    // Run validations ON THE NEW STATE(!) but before updating the entity in the database.
    let partial_validation_results: EntityViolations =
        context.validator.validate_single(&active_model);

    let has_violations = partial_validation_results.has_violations();

    if has_violations {
        let has_critical_violations =
            partial_validation_results.has_violation_of_type(ValidationViolationType::Critical);

        // Broadcast the PARTIAL validation result to all registered WebSocket connections.
        controller.get_websocket_controller().broadcast_json(
            &CrudWsMessage::PartialValidationResult(partial_validation_results.clone().into()),
        );

        // Persist the validation results for later access/use.
        let persistable = into_persistable(partial_validation_results);
        context
            .validation_result_repository
            .save_all(persistable)
            .await;

        // The existence of CRITICAL violations must block a save!
        if has_critical_violations {
            // SAFETY: Calling unwrap is safe, as the if above assigns a Some variant and runs with the same condition as this code.
            return Ok(SaveResult::CriticalValidationErrors);
        }
    } else {
        // We know that the entity is valid and therefor need to delete all previously stored violations for this entity.
        // The active_model might not have an id, thou that is unlikely when doing an "update".
        if let Some(id) = R::CrudColumn::get_id(&active_model) {
            context
                .validation_result_repository
                .delete_for(StrictOwnedEntityInfo {
                    aggregate_name: String::from(R::TYPE.into()),
                    entity_id: id,
                })
                .await;
        }

        // Inform the websocket listeners.
        controller.get_websocket_controller().broadcast_json(
            &CrudWsMessage::PartialValidationResult(partial_validation_results.clone().into()),
        );
    }

    let result = active_model
        .update(controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;

    // Inform all participants that the entity was updated.
    // TODO: Exclude the current user!
    controller
        .get_websocket_controller()
        .broadcast_json(&CrudWsMessage::EntityUpdated(EntityUpdated {
            aggregate_name: R::TYPE.into().to_owned(),
            entity_id,
            with_validation_errors: has_violations
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
