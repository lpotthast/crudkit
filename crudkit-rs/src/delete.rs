//! Delete operations for CRUD resources.
//!
//! Supports three modes:
//! - `delete_by_id`: Delete a single entity by its ID.
//! - `delete_one`: Delete a single entity matching a condition (first match).
//! - `delete_many`: Delete multiple entities matching a condition (all matches).

use std::sync::Arc;

use crate::data::HasId;
use crate::validate::{run_global_validation, run_model_validation};
use crate::{
    auth::RequestContext,
    collaboration,
    error::CrudError,
    lifetime::{CrudLifetime, DeleteOperation, DeleteRequest, HookError},
    prelude::*,
    validation::{CrudAction, ValidationContext, ValidationTrigger, When},
};
use crudkit_core::condition::{TryIntoAllEqualCondition, merge_conditions};
use crudkit_core::id::{Id, SerializableId};
use crudkit_core::request::{DeleteById, DeleteMany, DeleteOne};
use crudkit_core::validation::PartialSerializableAggregateViolations;
use crudkit_core::{Deleted, DeletedMany};

/// Maximum memory budget per batch (in bytes).
const BATCH_MEMORY_BUDGET: usize = 50_000_000;

/// Multiplier to estimate total memory including heap allocations.
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
    if batch_size < MIN_BATCH_SIZE {
        MIN_BATCH_SIZE
    } else if batch_size > MAX_BATCH_SIZE {
        MAX_BATCH_SIZE
    } else {
        batch_size
    }
}

/// Delete a single entity by its ID, within the request's optional condition.
///
/// # Errors
///
/// Returns `CrudError::IntoCondition` if the ID cannot be turned into a condition,
/// `CrudError::NotFound` if no entity with that ID matches the condition, `CrudError::Repository` if fetching or deleting
/// fails, `CrudError::Forbidden`, `CrudError::UnprocessableEntity`, or
/// `CrudError::LifecycleHookError` if a lifecycle hook rejects the operation or fails, or
/// `CrudError::CriticalValidationErrors` if validation blocks the deletion.
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
    let condition = merge_conditions(Some(id_condition), body.condition);

    let model = context
        .repository
        .fetch_one(None, None, None, condition.as_ref())
        .await
        .map_err(|err| CrudError::Repository {
            reason: Arc::new(err),
        })?
        .ok_or(CrudError::NotFound)?;

    let delete_request = DeleteRequest {
        operation: DeleteOperation::ById,
        skip: None,
        order_by: None,
        condition,
    };

    execute_single_delete(model, &delete_request, &context, &request).await?;

    run_global_validation::<R>(&context).await;

    Ok(Deleted {
        entities_affected: 1,
    })
}

/// Delete a single entity matching a condition.
///
/// # Errors
///
/// Returns `CrudError::NotFound` if no entity matches the condition, `CrudError::Repository` if
/// fetching or deleting fails, `CrudError::Forbidden`, `CrudError::UnprocessableEntity`, or
/// `CrudError::LifecycleHookError` if a lifecycle hook rejects the operation or fails, or
/// `CrudError::CriticalValidationErrors` if validation blocks the deletion.
#[tracing::instrument(level = "info", skip(context, request))]
pub async fn delete_one<R: CrudResource>(
    request: RequestContext<R::Auth>,
    context: Arc<CrudContext<R>>,
    body: DeleteOne<R::ModelField>,
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

    run_global_validation::<R>(&context).await;

    Ok(Deleted {
        entities_affected: 1,
    })
}

/// Error type for single entity deletion, used internally to distinguish failure modes.
enum SingleDeleteError {
    HookRejected(CrudError),
    ValidationFailed(CrudError),
    RepositoryError(CrudError),
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

/// Execute deletion for a single entity.
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

    let entity_id = model.id();
    let serializable_id = entity_id.to_serializable_id();

    // Validate the entity to check if deletion should be blocked.
    let trigger = ValidationTrigger::CrudAction(ValidationContext {
        action: CrudAction::Delete,
        when: When::Before,
    });
    let partial_validation_results =
        run_model_validation::<R>(&context.validators, &model, trigger);

    if partial_validation_results.has_critical_violations() {
        return Err(SingleDeleteError::ValidationFailed(
            CrudError::CriticalValidationErrors {
                violations: PartialSerializableAggregateViolations::from(
                    partial_validation_results,
                    Some(&serializable_id),
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
/// # Errors
///
/// Returns `CrudError::Repository` if fetching a batch of entities fails. Per-entity hook
/// rejections, validation failures, and delete errors do not fail the operation; they are reported
/// in the returned `DeletedMany`.
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
    loop {
        let models = context
            .repository
            .fetch_many(Some(batch_size), None, None, body.condition.as_ref())
            .await
            .map_err(|err| CrudError::Repository {
                reason: Arc::new(err),
            })?;

        if models.is_empty() {
            break;
        }

        let already_deleted = result.deleted_count;

        for model in models {
            delete_one_of_many(model, &delete_request, &context, &request, &mut result).await;
        }

        // If no entities were deleted in this batch, stop to avoid infinite loop.
        if result.deleted_count == already_deleted {
            break;
        }
    }

    run_global_validation::<R>(&context).await;

    Ok(result)
}

/// Deletes `model` as part of a `delete_many` operation, recording the outcome in `result`.
async fn delete_one_of_many<R: CrudResource>(
    model: R::Model,
    delete_request: &DeleteRequest<R>,
    context: &Arc<CrudContext<R>>,
    request: &RequestContext<R::Auth>,
    result: &mut DeletedMany,
) {
    let entity_id = model.id();
    let serializable_id = entity_id.to_serializable_id();

    // Run before_delete hook.
    let hook_data = R::HookData::default();
    let hook_data = match R::Lifetime::before_delete(
        &model,
        delete_request,
        &context.res_context,
        request.clone(),
        hook_data,
    )
    .await
    {
        Ok(data) => data,
        Err(HookError::Forbidden { reason } | HookError::UnprocessableEntity { reason }) => {
            result.aborted.push((serializable_id.clone(), reason));
            return;
        }
        Err(HookError::Internal(err)) => {
            result
                .errors
                .push((serializable_id.clone(), format!("Lifecycle error: {err}")));
            return;
        }
    };

    // Validate the entity.
    let trigger = ValidationTrigger::CrudAction(ValidationContext {
        action: CrudAction::Delete,
        when: When::Before,
    });
    let partial_validation_results =
        run_model_validation::<R>(&context.validators, &model, trigger);

    if partial_validation_results.has_critical_violations() {
        result.validation_failed.push(serializable_id.clone());
        return;
    }

    // Delete the entity.
    let deleted_model = model.clone();
    match context.repository.delete(model).await {
        Ok(_delete_result) => {
            let _ = R::Lifetime::after_delete(
                &deleted_model,
                delete_request,
                &context.res_context,
                request.clone(),
                hook_data,
            )
            .await;

            if let Err(e) = context
                .validation_result_repository
                .delete_all_of_entity(R::TYPE.name(), &entity_id)
                .await
            {
                tracing::warn!(
                    "Failed to delete validation results for entity {entity_id:?}: {e:?}"
                );
            }

            collaboration::broadcast_deletion_event(context, serializable_id.clone()).await;

            result.deleted_count += 1;
            result.deleted_ids.push(serializable_id.clone());
        }
        Err(err) => {
            result
                .errors
                .push((serializable_id.clone(), format!("Delete error: {err:?}")));
        }
    }
}
