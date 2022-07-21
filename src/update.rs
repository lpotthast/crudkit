use crate::{prelude::*, validation::into_persistable};
use axum::extract::ws::Message;
use axum_websockets::TypedMessage;
use crud_shared_types::{
    validation::{EntityValidations, SerializableValidations, StrictOwnedEntityInfo},
    Condition, CrudError, SaveResult,
};
use sea_orm::{ActiveModelTrait, JsonValue};
use serde::Deserialize;
use serde_json::json;
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
) -> Result<JsonValue, CrudError> {
    let model =
        build_select_query::<R::Entity, R::Model, R::ActiveModel, R::Column, R::CrudColumn>(
            None,
            None,
            None,
            body.condition,
        )?
        .one(controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?
        .ok_or_else(|| CrudError::ReadOneFoundNone)?;

    // Use the "CreateModel" to deserialize the given JSON. Some not required members are allowed to be missing.
    let update = serde_json::from_str::<R::CreateModel>(body.entity.get()).map_err(|err| {
        CrudError::UnableToParseAsEntity(body.entity.get().to_owned(), err.to_string())
    })?;

    // Convert the model into an ActiveModel, allowing mutations.
    let mut active_model: R::ActiveModel = model.into();

    // Update the persisted active_model!
    active_model.update_with(update);

    // Run validations ON THE NEW STATE(!) but before updating the entity in the database.
    let partial_validation_results: EntityValidations =
        context.validator.validate_single(&active_model);

    let has_violations = partial_validation_results.has_violations();

    if has_violations {
        let has_critical_violations =
            partial_validation_results.has_violation_of_type(ValidationViolationType::Critical);

        // log::info!("Validation errors: {:?}", partial_validation_results);

        // Broadcast the PARTIAL validation result to all registered WebSocket connections.
        let msg: TypedMessage<SerializableValidations> = TypedMessage {
            message_type: String::from("partial_validation_result"),
            data: &(&partial_validation_results).into(), // TODO: reference not strictly needed, as we clone in the into() impl anyway...
        };
        let serialized_msg = match serde_json::to_string(&msg) {
            Ok(string) => string,
            Err(err) => {
                let err_msg = format!("Unable to serialize partial validation result: {err:?}");
                log::error!("{err_msg}");
                err_msg
            }
        };
        controller
            .get_websocket_controller()
            .broadcast_message(Message::Text(serialized_msg));

        // Persist the validation results for later access/use.
        let persistable = into_persistable(partial_validation_results);
        context
            .validation_result_repository
            .save_all(persistable)
            .await;

        // The existence of CRITICAL violations must block a save!
        if has_critical_violations {
            return Err(CrudError::ValidationErrors);
        }
    } else {
        // We know that the entity is valid and therefor need to delete all previously stored violations for this entity.
        // The active_model might not have an id, thou that is unlikely when doing an "update".
        if let Some(id) = R::CrudColumn::get_id(&active_model) {
            context
                .validation_result_repository
                .delete_for(StrictOwnedEntityInfo {
                    entity_name: String::from(R::TYPE.into()),
                    entity_id: id,
                })
                .await;
        }

        // TODO: Do we need to inform the websocket listeners?
    }

    let result = active_model
        .update(controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;

    Ok(json!(SaveResult {
        entity: result,
        with_validation_errors: has_violations,
    }))
}
