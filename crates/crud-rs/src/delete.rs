use crate::{
    error::CrudError,
    lifetime::{Abort, CrudLifetime},
    prelude::*,
    validation::{CrudAction, ValidationContext, ValidationTrigger, When},
    GetIdFromModel,
};
use crud_shared_types::{
    prelude::*,
    validation::PartialSerializableValidations,
    ws_messages::{CrudWsMessage, EntityDeleted},
};
use indexmap::IndexMap;
use sea_orm::ModelTrait;
use serde::Deserialize;
use snafu::{Backtrace, GenerateImplicitData};
use std::{collections::HashMap, sync::Arc};
use utoipa::ToSchema;

#[derive(Debug, ToSchema, Deserialize)]
pub struct DeleteById {
    pub id: SerializableId,
}

#[derive(Debug, ToSchema, Deserialize)]
pub struct DeleteOne<R: CrudResource> {
    pub skip: Option<u64>,
    #[schema(value_type = Option<Object>, example = json!({"id": Order::Asc}))]
    // TODO: Better type definition including Column and Order types? Example not showing in UI...
    pub order_by: Option<IndexMap<R::CrudColumn, Order>>,
    pub condition: Option<Condition>,
}

#[derive(ToSchema, Deserialize)]
pub struct DeleteMany {
    pub condition: Option<Condition>,
}

#[tracing::instrument(level = "info", skip(controller, context, res_context))]
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
        &Some(body.id.to_all_equal_condition()),
    )?;

    let model = select
        .one(controller.get_database_connection())
        .await
        .map_err(|err| CrudError::Db {
            reason: err.to_string(),
            backtrace: Backtrace::generate(),
        })?
        .ok_or(CrudError::ReadOneFoundNone {
            backtrace: Backtrace::generate(),
        })?;

    // TODO: Make sure that the user really has the right to delete this entry!!! Maybe an additional lifetime check?

    let hook_data = R::HookData::default();
    let (abort, hook_data) = R::Lifetime::before_delete(&model, &res_context, hook_data)
        .await
        .expect("before_create to no error");

    if let Abort::Yes { reason } = abort {
        return Ok(DeleteResult::Aborted { reason });
    }

    let entity_id = model.get_id();
    //.expect("Stored entity without an ID should be impossible!");

    let serializable_id = entity_id.into_serializable_id();

    let active_model = model.clone().into();

    // Validate the entity, so that we can block its deletion if validators say so.
    let trigger = ValidationTrigger::CrudAction(ValidationContext {
        action: CrudAction::Delete,
        when: When::Before,
    });
    let partial_validation_results = context.validator.validate_single(&active_model, trigger);

    // Prevent deletion on critical errors.
    if partial_validation_results.has_critical_violations() {
        // TODO: Only notify the user that issued THIS REQUEST!!!

        let partial_serializable_validations: PartialSerializableValidations = HashMap::from([(
            String::from(R::TYPE.into()),
            partial_validation_results.clone().into(),
        )]);

        controller.get_websocket_controller().broadcast_json(
            &CrudWsMessage::PartialValidationResult(partial_serializable_validations),
        );

        // NOTE: Validations done before a deletion are only there to prevent it if necessary. Nothing must be persisted.
        return Ok(DeleteResult::CriticalValidationErrors);
    }

    // Delete the entity in the database.
    let cloned_model = model.clone();
    let delete_result = R::Model::delete(model, controller.get_database_connection())
        .await
        .map_err(|err| CrudError::Db {
            reason: err.to_string(),
            backtrace: Backtrace::generate(),
        })?;

    let _hook_data = R::Lifetime::after_delete(&cloned_model, &res_context, hook_data).await;

    // Deleting the entity could have introduced new validation errors in other parts ot the system.
    // TODO: let validation run again...

    // All previous validations regarding this entity must be deleted!
    context
        .validation_result_repository
        .delete_all_for(&entity_id) // String::from(R::TYPE.into()),
        .await;

    // Inform all participants that the entity was deleted.
    // TODO: Exclude the current user!
    controller
        .get_websocket_controller()
        .broadcast_json(&CrudWsMessage::EntityDeleted(EntityDeleted {
            aggregate_name: R::TYPE.into().to_owned(),
            entity_id: serializable_id,
        }));

    Ok(DeleteResult::Deleted(delete_result.rows_affected))
}

#[tracing::instrument(level = "info", skip(controller, context, res_context))]
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
        .map_err(|err| CrudError::Db {
            reason: err.to_string(),
            backtrace: Backtrace::generate(),
        })?
        .ok_or(CrudError::ReadOneFoundNone {
            backtrace: Backtrace::generate(),
        })?;

    let hook_data = R::HookData::default();
    let (abort, hook_data) = R::Lifetime::before_delete(&model, &res_context, hook_data)
        .await
        .expect("before_create to no error");

    if let Abort::Yes { reason } = abort {
        return Ok(DeleteResult::Aborted { reason });
    }

    let entity_id = model.get_id();
    //.expect("Stored entity without an ID should be impossible!");

    let serializable_id = entity_id.into_serializable_id();

    let active_model = model.clone().into();

    // Validate the entity, so that we can block its deletion if validators say so.
    let trigger = ValidationTrigger::CrudAction(ValidationContext {
        action: CrudAction::Delete,
        when: When::Before,
    });
    let partial_validation_results = context.validator.validate_single(&active_model, trigger);

    // Prevent deletion on critical errors.
    if partial_validation_results.has_critical_violations() {
        // TODO: Only notify the user that issued THIS REQUEST!!!

        let partial_serializable_validations: PartialSerializableValidations = HashMap::from([(
            String::from(R::TYPE.into()),
            partial_validation_results.clone().into(),
        )]);

        controller.get_websocket_controller().broadcast_json(
            &CrudWsMessage::PartialValidationResult(partial_serializable_validations),
        );

        // NOTE: Validations done before a deletion are only there to prevent it if necessary. Nothing must be persisted.
        return Ok(DeleteResult::CriticalValidationErrors);
    }

    // Delete the entity in the database.
    let cloned_model = model.clone();
    let delete_result = R::Model::delete(model, controller.get_database_connection())
        .await
        .map_err(|err| CrudError::Db {
            reason: err.to_string(),
            backtrace: Backtrace::generate(),
        })?;

    let _hook_data = R::Lifetime::after_delete(&cloned_model, &res_context, hook_data).await;

    // All previous validations regarding this entity must be deleted!
    context
        .validation_result_repository
        .delete_all_for(&entity_id) // String::from(R::TYPE.into()),
        .await;

    // Inform all participants that the entity was deleted.
    // TODO: Exclude the current user!
    controller
        .get_websocket_controller()
        .broadcast_json(&CrudWsMessage::EntityDeleted(EntityDeleted {
            aggregate_name: R::TYPE.into().to_owned(),
            entity_id: serializable_id,
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
        .map_err(|err| CrudError::Db {
            reason: err.to_string(),
            backtrace: Backtrace::generate(),
        })?;
    Ok(DeleteResult::Deleted(delete_result.rows_affected))
}
