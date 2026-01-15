use indexmap::IndexMap;
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use utoipa::ToSchema;

use crudkit_condition::{Condition, TryIntoAllEqualCondition};
use crudkit_core::{Deleted, DeletedMany, Order};
use crudkit_id::{Id, SerializableId};
use crudkit_validation::PartialSerializableValidations;
use crudkit_websocket::{CkWsMessage, EntityDeleted};

use crate::{
    auth::RequestContext,
    error::CrudError,
    lifetime::{CrudLifetime, DeleteOperation, DeleteRequest, HookError},
    prelude::*,
    validation::{CrudAction, ValidationContext, ValidationTrigger, When},
    GetIdFromModel,
};

/// Maximum memory budget per batch (in bytes).
///
/// This limits how much memory a single batch of entities can consume.
/// Default: 50 MB. Adjust based on available system memory.
const BATCH_MEMORY_BUDGET: usize = 50_000_000;

/// Multiplier to estimate total memory including heap allocations.
///
/// **Important:** `std::mem::size_of::<T>()` returns the *stack* size of a type,
/// which is NOT the same as total memory usage! Heap-allocated data is not included.
///
/// Example: `String` always reports 24 bytes (pointer + len + capacity), but a
/// 1000-ASCII-character string actually uses ~1024 bytes (24 stack + 1000 heap).
///
/// This multiplier compensates for heap allocations in fields like:
/// - `String`, `Vec<T>`, `Box<T>`
/// - `serde_json::Value`
/// - `Option<T>` containing heap types
///
/// Tuning guide:
/// - Set to 1 if models contain mostly primitive fields (i32, bool, Uuid, etc.)
/// - Set to 2-3 if models have several String/Vec fields
/// - Set to 4+ if models contain large JSON blobs or binary data
const HEAP_OVERHEAD_MULTIPLIER: usize = 3;

/// Minimum batch size to ensure progress even for large models.
const MIN_BATCH_SIZE: u64 = 10;

/// Maximum batch size to limit individual query load.
const MAX_BATCH_SIZE: u64 = 1000;

/// Calculate batch size based on model memory footprint.
const fn calculate_batch_size<M>() -> u64 {
    let stack_usage = size_of::<M>();
    let estimated_combined_usage = stack_usage * HEAP_OVERHEAD_MULTIPLIER;
    if estimated_combined_usage == 0 {
        return MAX_BATCH_SIZE;
    }
    let batch_size = (BATCH_MEMORY_BUDGET / estimated_combined_usage) as u64;
    // Note: clamp() is not const, so use manual bounds checking
    if (batch_size) < MIN_BATCH_SIZE {
        MIN_BATCH_SIZE
    } else if (batch_size) > MAX_BATCH_SIZE {
        MAX_BATCH_SIZE
    } else {
        batch_size
    }
}

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

#[derive(Debug, ToSchema, Deserialize)]
pub struct DeleteMany {
    pub condition: Option<Condition>,
}

#[tracing::instrument(level = "info", skip(context, request))]
pub async fn delete_by_id<R: CrudResource>(
    request: RequestContext<R::Auth>,
    context: Arc<CrudContext<R>>,
    body: DeleteById,
) -> Result<Deleted, CrudError> {
    let id_condition = body
        .id
        .0
        .iter()
        .map(|(name, value)| (name.clone(), value.clone()))
        .try_into_all_equal_condition()
        .map_err(|err| CrudError::IntoCondition { source: err })?;

    // TODO: This initially fetched Model, not ReadViewModel...
    let model = context
        .repository
        .fetch_one(None, None, None, Some(&id_condition))
        .await
        .map_err(|err| CrudError::Repository {
            reason: Arc::new(err),
        })?
        .ok_or(CrudError::NotFound)?;

    let delete_request = DeleteRequest {
        operation: DeleteOperation::ById,
        skip: None,
        order_by: None,
        condition: Some(id_condition),
    };

    let hook_data = R::HookData::default();

    let hook_data = R::Lifetime::before_delete(
        &model,
        &delete_request,
        &context.res_context,
        request.clone(),
        hook_data,
    )
    .await
    .map_err(CrudError::from)?;

    let entity_id = model.get_id();
    //.expect("Stored entity without an ID should be impossible!");

    let serializable_id = entity_id.to_serializable_id();

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

        context
            .ws_controller
            .broadcast_json(CkWsMessage::PartialValidationResult(
                partial_serializable_validations,
            ));

        // NOTE: Validations done before a deletion are only there to prevent it if necessary. Nothing must be persisted.
        return Err(CrudError::ValidationFailed);
    }

    // Delete the entity in the database.
    let deleted_model = model.clone();

    let delete_result =
        context
            .repository
            .delete(model)
            .await
            .map_err(|err| CrudError::Repository {
                reason: Arc::new(err),
            })?;

    let _hook_data = R::Lifetime::after_delete(
        &deleted_model,
        &delete_request,
        &context.res_context,
        request,
        hook_data,
    )
    .await
    .map_err(CrudError::from)?;

    // Deleting the entity could have introduced new validation errors in other parts ot the system.
    // TODO: let validation run again...

    // All previous validations regarding this entity must be deleted!
    let _ = context
        .validation_result_repository
        .delete_all_for(&entity_id) // String::from(R::TYPE.into()),
        .await;

    // Inform all participants that the entity was deleted.
    // TODO: Exclude the current user!
    context
        .ws_controller
        .broadcast_json(CkWsMessage::EntityDeleted(EntityDeleted {
            aggregate_name: R::TYPE.into().to_owned(),
            entity_id: serializable_id,
        }));

    Ok(Deleted {
        entities_affected: delete_result.entities_affected,
    })
}

#[tracing::instrument(level = "info", skip(context, request))]
pub async fn delete_one<R: CrudResource>(
    request: RequestContext<R::Auth>,
    context: Arc<CrudContext<R>>,
    body: DeleteOne<R>,
) -> Result<Deleted, CrudError> {
    let model = context
        .repository
        .fetch_one(
            None,
            body.skip,
            body.order_by.clone(),
            body.condition.as_ref(),
        )
        .await
        .map_err(|err| CrudError::Repository {
            reason: Arc::new(err),
        })?
        .ok_or(CrudError::NotFound)?;

    let delete_request = DeleteRequest {
        operation: DeleteOperation::One,
        skip: body.skip,
        order_by: body.order_by,
        condition: body.condition,
    };

    let hook_data = R::HookData::default();

    let hook_data = R::Lifetime::before_delete(
        &model,
        &delete_request,
        &context.res_context,
        request.clone(),
        hook_data,
    )
    .await
    .map_err(CrudError::from)?;

    let entity_id = model.get_id();
    //.expect("Stored entity without an ID should be impossible!");

    let serializable_id = entity_id.to_serializable_id();

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

        context
            .ws_controller
            .broadcast_json(CkWsMessage::PartialValidationResult(
                partial_serializable_validations,
            ));

        // NOTE: Validations done before a deletion are only there to prevent it if necessary. Nothing must be persisted.
        return Err(CrudError::ValidationFailed);
    }

    // Delete the entity in the database.
    let deleted_model = model.clone();

    let delete_result =
        context
            .repository
            .delete(model)
            .await
            .map_err(|err| CrudError::Repository {
                reason: Arc::new(err),
            })?;

    let _hook_data = R::Lifetime::after_delete(
        &deleted_model,
        &delete_request,
        &context.res_context,
        request,
        hook_data,
    )
    .await
    .map_err(CrudError::from)?;

    // All previous validations regarding this entity must be deleted!
    // TODO: We should not ignore the error here completely!
    let _ = context
        .validation_result_repository
        .delete_all_for(&entity_id) // String::from(R::TYPE.into()),
        .await;

    // Inform all participants that the entity was deleted.
    // TODO: Exclude the current user!
    context
        .ws_controller
        .broadcast_json(CkWsMessage::EntityDeleted(EntityDeleted {
            aggregate_name: R::TYPE.into().to_owned(),
            entity_id: serializable_id,
        }));

    Ok(Deleted {
        entities_affected: delete_result.entities_affected,
    })
}

/// Helper to convert SerializableId to JSON value for result reporting.
fn id_to_json(id: &SerializableId) -> serde_json::Value {
    serde_json::to_value(id).unwrap_or_else(|_| serde_json::Value::Null)
}

/// Delete multiple entities matching a condition.
///
/// This function fetches entities in batches to prevent OOM when deleting large numbers of entities.
/// Each entity is processed individually, running lifecycle hooks and validation.
/// Partial success is supported - if some entities fail to delete, others will still be processed.
///
/// # Parameters
///
/// * `keycloak_token` - The authentication token. Authorization checks are typically performed
///   at the route/middleware level before this function is called. The token is available here
///   for use in lifecycle hooks via the resource context if needed for entity-level permission checks.
#[tracing::instrument(level = "info", skip(context, request))]
pub async fn delete_many<R: CrudResource>(
    request: RequestContext<R::Auth>,
    context: Arc<CrudContext<R>>,
    body: DeleteMany,
) -> Result<DeletedMany, CrudError> {
    let mut result = DeletedMany {
        deleted_count: 0,
        deleted_ids: Vec::new(),
        aborted: Vec::new(),
        validation_failed: Vec::new(),
        errors: Vec::new(),
    };

    let batch_size = calculate_batch_size::<R::Model>();

    let delete_request = DeleteRequest {
        operation: DeleteOperation::Many,
        skip: None,
        order_by: None,
        condition: body.condition.clone(),
    };

    // Process entities in batches to prevent OOM issues.
    // We always fetch from offset 0 because deleted entities no longer match the condition.
    // If no deletions occur in a batch (all blocked by hooks/validation), we stop.
    loop {
        let models = context
            .repository
            .fetch_many(Some(batch_size), None, None, body.condition.as_ref())
            .await
            .map_err(|err| CrudError::Repository {
                reason: Arc::new(err),
            })?;

        // No more entities to process.
        if models.is_empty() {
            break;
        }

        let already_deleted = result.deleted_count;

        // Process each entity in the batch.
        for model in models {
            let entity_id = model.get_id();
            let serializable_id = entity_id.to_serializable_id();

            // Run before_delete hook.
            let hook_data = R::HookData::default();
            let hook_data = match R::Lifetime::before_delete(
                &model,
                &delete_request,
                &context.res_context,
                request.clone(),
                hook_data,
            )
            .await
            {
                Ok(data) => data,
                Err(HookError::Forbidden { reason })
                | Err(HookError::UnprocessableEntity { reason }) => {
                    result.aborted.push((id_to_json(&serializable_id), reason));
                    continue;
                }
                Err(HookError::Internal(err)) => {
                    result.errors.push((
                        id_to_json(&serializable_id),
                        format!("Lifecycle error: {err}"),
                    ));
                    continue;
                }
            };

            // Validate the entity.
            let active_model = model.clone().into();
            let trigger = ValidationTrigger::CrudAction(ValidationContext {
                action: CrudAction::Delete,
                when: When::Before,
            });
            let partial_validation_results =
                context.validator.validate_single(&active_model, trigger);

            // Check for critical validation errors.
            if partial_validation_results.has_critical_violations() {
                // Broadcast validation errors via WebSocket.
                let partial_serializable_validations: PartialSerializableValidations =
                    HashMap::from([(
                        String::from(R::TYPE.into()),
                        partial_validation_results.clone().into(),
                    )]);

                let _ = context
                    .ws_controller
                    .broadcast_json(CkWsMessage::PartialValidationResult(
                        partial_serializable_validations,
                    ));

                result.validation_failed.push(id_to_json(&serializable_id));
                continue;
            }

            // Delete the entity.
            let deleted_model = model.clone();
            match context.repository.delete(model).await {
                Ok(_delete_result) => {
                    // Run after_delete hook (ignore errors for batch operations).
                    let _ = R::Lifetime::after_delete(
                        &deleted_model,
                        &delete_request,
                        &context.res_context,
                        request.clone(),
                        hook_data,
                    )
                    .await;

                    // Clear validation results for this entity.
                    let _ = context
                        .validation_result_repository
                        .delete_all_for(&entity_id)
                        .await;

                    // Broadcast deletion via WebSocket.
                    let _ = context
                        .ws_controller
                        .broadcast_json(CkWsMessage::EntityDeleted(EntityDeleted {
                            aggregate_name: R::TYPE.into().to_owned(),
                            entity_id: serializable_id.clone(),
                        }));

                    result.deleted_count += 1;
                    result.deleted_ids.push(id_to_json(&serializable_id));
                }
                Err(err) => {
                    result.errors.push((
                        id_to_json(&serializable_id),
                        format!("Delete error: {err:?}"),
                    ));
                }
            }
        }

        // If no entities were deleted in this batch, all remaining entities are blocked
        // (aborted, validation failed, or errored). Stop to avoid infinite loop.
        if result.deleted_count == already_deleted {
            break;
        }
    }

    Ok(result)
}
