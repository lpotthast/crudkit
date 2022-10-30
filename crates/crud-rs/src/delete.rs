use crate::{
    prelude::*,
    validation::{CrudAction, ValidationContext, ValidationTrigger, When},
    lifetime::{CrudLifetime, Abort},
};
use crud_shared_types::{
    validation::StrictOwnedEntityInfo,
    ws_messages::{CrudWsMessage, EntityDeleted},
    Condition, ConditionClause, ConditionClauseValue, ConditionElement, CrudError, DeleteResult,
    Operator, Order,
};
use indexmap::IndexMap;
use sea_orm::ModelTrait;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct DeleteById {
    pub id: u32,
}

#[derive(Deserialize)]
pub struct DeleteOne<R: CrudResource> {
    pub skip: Option<u64>,
    pub order_by: Option<IndexMap<R::CrudColumn, Order>>,
    pub condition: Option<Condition>,
}

#[derive(Deserialize)]
pub struct DeleteMany {
    pub condition: Option<Condition>,
}

pub async fn delete_by_id<R: CrudResource>(
    controller: Arc<CrudController>,
    context: Arc<CrudContext<R>>,
    res_context: Arc<R::Context>,
    body: DeleteById,
) -> Result<DeleteResult, CrudError> {
    let select = build_select_query::<R::Entity, R::Model, R::ActiveModel, R::Column, R::CrudColumn>(
        None,
        None,
        None,
        &Some(Condition::All(vec![ConditionElement::Clause(
            ConditionClause {
                column_name: R::CrudColumn::get_id_field_name(),
                operator: Operator::Equal,
                value: ConditionClauseValue::I32(body.id.try_into().unwrap()),
            },
        )])),
    )?;

    let model = select
        .one(controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?
        .ok_or(CrudError::ReadOneFoundNone)?;

    let hook_data = R::HookData::default();
    let (abort, hook_data) = R::Lifetime::before_delete(&model, &res_context, hook_data).await.expect("before_create to no error");

    if let Abort::Yes { reason } = abort {
        return Ok(DeleteResult::Aborted { reason })
    }

    let active_model = model.clone().into();
    let entity_id = R::CrudColumn::get_id(&active_model)
        .expect("Stored entity without an ID should be impossible!");

    // Validate the entity, so that we can block its deletion if validators say so.
    let trigger = ValidationTrigger::CrudAction(ValidationContext {
        action: CrudAction::Delete,
        when: When::Before,
    });
    let partial_validation_results = context.validator.validate_single(&active_model, trigger);

    // Prevent deletion on critical errors.
    if partial_validation_results.has_critical_violations() {
        // TODO: Only notify the user that issued THIS REQUEST!!!
        controller.get_websocket_controller().broadcast_json(
            &CrudWsMessage::PartialValidationResult(partial_validation_results.clone().into()),
        );

        // NOTE: Validations done before a deletion are only there to prevent it if necessary. Nothing must be persisted.
        return Ok(DeleteResult::CriticalValidationErrors);
    }

    // Delete the entity in the database.
    let cloned_model = model.clone();
    let delete_result = R::Model::delete(model, controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;


    let _hook_data = R::Lifetime::after_delete(&cloned_model, &res_context, hook_data).await;

    // Deleting the entity could have introduced new validation errors in other parts ot the system.
    // TODO: let validation run again...

    // All previous validations regarding this entity must be deleted!
    context
        .validation_result_repository
        .delete_for(StrictOwnedEntityInfo {
            aggregate_name: String::from(R::TYPE.into()),
            entity_id,
        })
        .await;

    // Inform all participants that the entity was deleted.
    // TODO: Exclude the current user!
    controller
        .get_websocket_controller()
        .broadcast_json(&CrudWsMessage::EntityDeleted(EntityDeleted {
            aggregate_name: R::TYPE.into().to_owned(),
            entity_id,
        }));

    Ok(DeleteResult::Deleted(delete_result.rows_affected))
}

pub async fn delete_one<R: CrudResource>(
    controller: Arc<CrudController>,
    context: Arc<CrudContext<R>>,
    res_context: Arc<R::Context>,
    body: DeleteOne<R>,
) -> Result<DeleteResult, CrudError> {
    let select = build_select_query::<R::Entity, R::Model, R::ActiveModel, R::Column, R::CrudColumn>(
        None,
        body.skip,
        body.order_by,
        &body.condition,
    )?;

    let model = select
        .one(controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?
        .ok_or(CrudError::ReadOneFoundNone)?;

    let hook_data = R::HookData::default();
    let (abort, hook_data) = R::Lifetime::before_delete(&model, &res_context, hook_data).await.expect("before_create to no error");

    if let Abort::Yes { reason } = abort {
        return Ok(DeleteResult::Aborted { reason })
    }

    let active_model = model.clone().into();
    let entity_id = R::CrudColumn::get_id(&active_model)
        .expect("Stored entity without an ID should be impossible!");

    // Validate the entity, so that we can block its deletion if validators say so.
    let trigger = ValidationTrigger::CrudAction(ValidationContext {
        action: CrudAction::Delete,
        when: When::Before,
    });
    let partial_validation_results = context.validator.validate_single(&active_model, trigger);

    // Prevent deletion on critical errors.
    if partial_validation_results.has_critical_violations() {
        // TODO: Only notify the user that issued THIS REQUEST!!!
        controller.get_websocket_controller().broadcast_json(
            &CrudWsMessage::PartialValidationResult(partial_validation_results.clone().into()),
        );

        // NOTE: Validations done before a deletion are only there to prevent it if necessary. Nothing must be persisted.
        return Ok(DeleteResult::CriticalValidationErrors);
    }

    // Delete the entity in the database.
    let cloned_model = model.clone();
    let delete_result = R::Model::delete(model, controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;

    let _hook_data = R::Lifetime::after_delete(&cloned_model, &res_context, hook_data).await;

    // All previous validations regarding this entity must be deleted!
    context
        .validation_result_repository
        .delete_for(StrictOwnedEntityInfo {
            aggregate_name: String::from(R::TYPE.into()),
            entity_id,
        })
        .await;

    // Inform all participants that the entity was deleted.
    // TODO: Exclude the current user!
    controller
        .get_websocket_controller()
        .broadcast_json(&CrudWsMessage::EntityDeleted(EntityDeleted {
            aggregate_name: R::TYPE.into().to_owned(),
            entity_id,
        }));

    Ok(DeleteResult::Deleted(delete_result.rows_affected))
}

// TODO: IMPLEMENT. Match implementations above. Extract logic, reducing duplication if possible.
pub async fn delete_many<R: CrudResource>(
    controller: Arc<CrudController>,
    _context: Arc<CrudContext<R>>,
    body: DeleteMany,
) -> Result<DeleteResult, CrudError> {
    todo!();
    // TODO: Add missing validation logic to this function.
    let delete_many = build_delete_many_query::<R::Entity>(&body.condition)?;
    let delete_result = delete_many
        .exec(controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;
    Ok(DeleteResult::Deleted(delete_result.rows_affected))
}
