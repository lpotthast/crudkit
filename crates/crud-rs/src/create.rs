use crate::{prelude::*, validation::{into_persistable, ValidationTrigger, ValidationContext, CrudAction, When}};
use crud_shared_types::{
    validation::PartialSerializableValidations,
    ws_messages::{CrudWsMessage, EntityCreated},
    CrudError, SaveResult, Saved,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CreateOne {
    pub entity: Box<serde_json::value::RawValue>,
}

pub async fn create_one<R: CrudResource>(
    controller: Arc<CrudController>,
    context: Arc<CrudContext<R>>,
    res_context: Arc<R::Context>,
    body: CreateOne,
) -> Result<SaveResult<R::Model>, CrudError> {
    let entity_json: &str = body.entity.get();

    // Use the "CreateModel" to deserialize the given JSON. Some not required members are allowed to be missing.
    let create_model: R::CreateModel = serde_json::from_str::<R::CreateModel>(entity_json)
        .map_err(|err| CrudError::UnableToParseAsEntity(entity_json.to_owned(), err.to_string()))?;

    // Directly create an "ActiveModel" from the "CreateModel", ready to be persisted.
    // By not going through the Model -> ActiveModel conversion, we give the user to exactly specify the data that should be Set/Unset.
    let mut active_entity: R::ActiveModel = create_model.into_active_model(res_context.as_ref());

    // We might have accidentally set attributes on the "ActiveModel" that we must not have in order to run validations.
    prune_active_model::<R>(&mut active_entity);

    // Run validations before inserting the entity. If critical violations are present, prevent the creation!
    // NOTE: All violations created here can not have an ID, as the entity was not yet saved!
    // OPTIMIZATION: We are only interested in CRITICAL violations. Can this be used to make this more efficient?
    let trigger = ValidationTrigger::CrudAction(ValidationContext {
        action: CrudAction::Create,
        when: When::Before,
    });
    let partial_validation_results = context.validator.validate_single(&active_entity, trigger);
    if partial_validation_results.has_critical_violations() {
        // TODO: Only notify the user that issued THIS REQUEST!!!
        // Broadcast the PARTIAL validation result to all registered WebSocket connections.
        controller.get_websocket_controller().broadcast_json(
            &CrudWsMessage::PartialValidationResult(partial_validation_results.clone().into()),
        );

        // NOTE: Nothing must be persisted, as the entity is not yet created!
        return Ok(SaveResult::CriticalValidationErrors);
    }

    // The entity to insert has no critical violations. The entity can be inserted!
    let insert_query = build_insert_query::<R>(active_entity)?;

    let inserted_entity: R::Model = insert_query
        .exec_with_returning(controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;

    let active_inserted_entity: R::ActiveModel = inserted_entity.clone().into();
    let entity_id = R::CrudColumn::get_id(&active_inserted_entity)
        .expect("Already inserted entities must have an ID!");

    // Reevaluate the entity for violations and broadcast all of them if some exist.
    let trigger = ValidationTrigger::CrudAction(ValidationContext {
        action: CrudAction::Create,
        when: When::After,
    });
    let partial_validation_results = context.validator.validate_single(&active_inserted_entity, trigger);
    let with_validation_errors = partial_validation_results.has_violations();
    if with_validation_errors {
        // Broadcast the PARTIAL validation result to all registered WebSocket connections.
        let mut serializable: PartialSerializableValidations =
            partial_validation_results.clone().into();

        // We successfully created the entry at this point. To delete any leftover "create" violations in the frontend, set create to Some empty vector!
        serializable
            .entry(R::TYPE.into().to_owned())
            .and_modify(|s| {
                s.create = Some(Vec::new());
            });
        controller
            .get_websocket_controller()
            .broadcast_json(&CrudWsMessage::PartialValidationResult(serializable));

        // Persist the validation results for later access/use.
        let persistable = into_persistable(partial_validation_results);
        context
            .validation_result_repository
            .save_all(persistable)
            .await;
    }

    // Inform all participants that the entity was updated.
    // TODO: Exclude the current user!
    controller
        .get_websocket_controller()
        .broadcast_json(&CrudWsMessage::EntityCreated(EntityCreated {
            aggregate_name: R::TYPE.into().to_owned(),
            entity_id,
            with_validation_errors,
        }));

    Ok(SaveResult::Saved(Saved {
        entity: inserted_entity,
        with_validation_errors,
    }))
}

// TODO: update_one_and_read_back() which updates and returns a ReadModel instead of an UpdateModel.
