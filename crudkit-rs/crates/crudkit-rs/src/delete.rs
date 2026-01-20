use indexmap::IndexMap;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;

use crate::validate::{run_entity_validation, run_global_validation};
use crate::{
    auth::RequestContext,
    collaboration,
    error::CrudError,
    lifetime::{CrudLifetime, DeleteOperation, DeleteRequest, HookError},
    prelude::*,
    validation::{CrudAction, ValidationContext, ValidationTrigger, When},
    GetIdFromModel,
};
use crudkit_condition::{Condition, TryIntoAllEqualCondition};
use crudkit_core::{Deleted, DeletedMany, Order};
use crudkit_id::{Id, SerializableId};
use crudkit_validation::PartialSerializableAggregateViolations;

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
        .clone()
        .into_entries()
        .try_into_all_equal_condition()
        .map_err(|err| CrudError::IntoCondition { source: err })?;

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

    execute_single_delete(model, &delete_request, &context, &request).await?;

    // Trigger global validation to check system-wide consistency.
    run_global_validation::<R>(&context).await;

    Ok(Deleted {
        entities_affected: 1,
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

    execute_single_delete(model, &delete_request, &context, &request).await?;

    // Trigger global validation to check system-wide consistency.
    run_global_validation::<R>(&context).await;

    Ok(Deleted {
        entities_affected: 1,
    })
}

/// Helper to convert SerializableId to JSON value for result reporting.
fn id_to_json(id: &SerializableId) -> serde_json::Value {
    serde_json::to_value(id).unwrap_or(serde_json::Value::Null)
}

/// Error type for single entity deletion, used internally to distinguish failure modes.
enum SingleDeleteError {
    /// Lifecycle hook rejected the deletion.
    HookRejected(CrudError),
    /// Validation failed with critical violations.
    ValidationFailed(CrudError),
    /// Repository operation failed.
    RepositoryError(CrudError),
    /// After-delete hook failed (entity was deleted but hook errored).
    AfterHookFailed(CrudError),
}

impl From<SingleDeleteError> for CrudError {
    fn from(err: SingleDeleteError) -> Self {
        match err {
            SingleDeleteError::HookRejected(e)
            | SingleDeleteError::ValidationFailed(e)
            | SingleDeleteError::RepositoryError(e)
            | SingleDeleteError::AfterHookFailed(e) => e,
        }
    }
}

/// Execute deletion for a single entity after it has been fetched.
///
/// This helper handles the common deletion flow:
/// 1. Run before_delete hook
/// 2. Validate the entity
/// 3. Delete from database
/// 4. Run after_delete hook
/// 5. Clear validation results
/// 6. Broadcast deletion via WebSocket
///
/// Returns the serializable ID on success for result reporting.
async fn execute_single_delete<R: CrudResource>(
    model: R::Model,
    delete_request: &DeleteRequest<R>,
    context: &Arc<CrudContext<R>>,
    request: &RequestContext<R::Auth>,
) -> Result<SerializableId, SingleDeleteError> {
    let hook_data = R::HookData::default();

    let hook_data = R::Lifetime::before_delete(
        &model,
        delete_request,
        &context.res_context,
        request.clone(),
        hook_data,
    )
    .await
    .map_err(|e| SingleDeleteError::HookRejected(CrudError::from(e)))?;

    let entity_id = model.get_id();
    let serializable_id = entity_id.to_serializable_id();

    let active_model: R::ActiveModel = model.clone().into();

    // Validate the entity to check if deletion should be blocked.
    let trigger = ValidationTrigger::CrudAction(ValidationContext {
        action: CrudAction::Delete,
        when: When::Before,
    });
    let partial_validation_results =
        run_entity_validation::<R>(&context.validators, &active_model, trigger);

    if partial_validation_results.has_critical_violations() {
        return Err(SingleDeleteError::ValidationFailed(
            CrudError::CriticalValidationErrors {
                violations: PartialSerializableAggregateViolations::from(
                    partial_validation_results,
                    Some(serializable_id.clone()),
                ),
            },
        ));
    }

    // Delete the entity from the database.
    let deleted_model = model.clone();
    context.repository.delete(model).await.map_err(|err| {
        SingleDeleteError::RepositoryError(CrudError::Repository {
            reason: Arc::new(err),
        })
    })?;

    // Run after_delete hook.
    R::Lifetime::after_delete(
        &deleted_model,
        delete_request,
        &context.res_context,
        request.clone(),
        hook_data,
    )
    .await
    .map_err(|e| SingleDeleteError::AfterHookFailed(CrudError::from(e)))?;

    // Clear validation results for this entity.
    if let Err(e) = context
        .validation_result_repository
        .delete_all_of_entity(R::TYPE.name(), &entity_id)
        .await
    {
        tracing::warn!("Failed to delete validation results for entity {entity_id:?}: {e:?}");
    }

    // Broadcast deletion via WebSocket.
    collaboration::broadcast_deletion_event(context, serializable_id.clone()).await;

    Ok(serializable_id)
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
                run_entity_validation::<R>(&context.validators, &active_model, trigger);

            // Check for critical validation errors.
            // Critical validation errors are returned synchronously in the response (validation_failed list).
            if partial_validation_results.has_critical_violations() {
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
                    if let Err(e) = context
                        .validation_result_repository
                        .delete_all_of_entity(R::TYPE.name(), &entity_id)
                        .await
                    {
                        tracing::warn!(
                            "Failed to delete validation results for entity {entity_id:?}: {e:?}"
                        );
                    }

                    // Broadcast deletion via WebSocket.
                    collaboration::broadcast_deletion_event(&context, serializable_id.clone())
                        .await;

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

    // Trigger global validation to check system-wide consistency.
    // Results are broadcast via WebSocket to all connected users.
    run_global_validation::<R>(&context).await;

    Ok(result)
}
