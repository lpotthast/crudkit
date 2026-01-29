//! Read operations for CRUD resources.
//!
//! Read operations return the ReadModel (which may be backed by a SQL view).

use crate::{
    auth::RequestContext,
    error::CrudError,
    lifetime::{CrudLifetime, ReadOperation, ReadRequest, ReadResult},
    prelude::*,
};

use crudkit_core::condition::Condition;
use crudkit_core::Order;

use indexmap::IndexMap;
use serde::Deserialize;
use std::sync::Arc;
use tracing::error;
use utoipa::ToSchema;

/// Request body for counting entities.
#[derive(Debug, ToSchema, Deserialize)]
pub struct ReadCount {
    /// Filter condition.
    pub condition: Option<Condition>,
}

/// Request body for reading one entity.
#[derive(Debug, ToSchema, Deserialize)]
pub struct ReadOne<R: CrudResource> {
    /// Number of entities to skip.
    pub skip: Option<u64>,
    /// Ordering specification.
    #[serde(bound = "")]
    #[schema(value_type = Option<Object>, example = json!({"id": Order::Asc}))]
    pub order_by: Option<IndexMap<R::ReadModelField, Order>>,
    /// Filter condition.
    pub condition: Option<Condition>,
}

/// Request body for reading many entities.
#[derive(Debug, ToSchema, Deserialize)]
pub struct ReadMany<R: CrudResource> {
    /// Maximum number of entities to return.
    pub limit: Option<u64>,
    /// Number of entities to skip.
    pub skip: Option<u64>,
    /// Ordering specification.
    #[serde(bound = "")]
    #[schema(value_type = Option<Object>, example = json!({"id": Order::Asc}))]
    pub order_by: Option<IndexMap<R::ReadModelField, Order>>,
    /// Filter condition.
    pub condition: Option<Condition>,
}

/// Count entities matching the given condition.
#[tracing::instrument(level = "info", skip(context, request))]
pub async fn read_count<R: CrudResource>(
    request: RequestContext<R::Auth>,
    context: Arc<CrudContext<R>>,
    body: ReadCount,
) -> Result<u64, CrudError> {
    let mut read_request = ReadRequest {
        operation: ReadOperation::Count,
        limit: None,
        skip: None,
        order_by: None,
        condition: body.condition,
    };

    let hook_data = R::HookData::default();

    let hook_data = R::Lifetime::before_read(
        &mut read_request,
        &context.res_context,
        request.clone(),
        hook_data,
    )
    .await
    .map_err(CrudError::from)?;

    let count = context
        .repository
        .count(None, None, None, read_request.condition.as_ref())
        .await
        .map_err(|err| CrudError::Repository {
            reason: Arc::new(err),
        })?;

    let mut read_result = ReadResult::Count(count);

    let _hook_data = R::Lifetime::after_read(
        &read_request,
        &mut read_result,
        &context.res_context,
        request,
        hook_data,
    )
    .await
    .map_err(CrudError::from)?;

    match read_result {
        ReadResult::Count(n) => Ok(n),
        _ => unreachable!("after_read should not change result type"),
    }
}

/// Read a single entity matching the given criteria.
#[tracing::instrument(level = "info", skip(context, request))]
pub async fn read_one<R: CrudResource>(
    request: RequestContext<R::Auth>,
    context: Arc<CrudContext<R>>,
    body: ReadOne<R>,
) -> Result<R::ReadModel, CrudError> {
    let mut read_request = ReadRequest {
        operation: ReadOperation::One,
        limit: None,
        skip: body.skip,
        order_by: body.order_by,
        condition: body.condition,
    };

    let hook_data = R::HookData::default();

    let hook_data = R::Lifetime::before_read(
        &mut read_request,
        &context.res_context,
        request.clone(),
        hook_data,
    )
    .await
    .map_err(CrudError::from)?;

    let entity = context
        .repository
        .read_one(
            None,
            read_request.skip,
            read_request.order_by.clone(),
            read_request.condition.as_ref(),
        )
        .await
        .map_err(|err| CrudError::Repository {
            reason: Arc::new(err),
        })?
        .ok_or(CrudError::NotFound)?;

    let mut read_result = ReadResult::One(entity);

    let _hook_data = R::Lifetime::after_read(
        &read_request,
        &mut read_result,
        &context.res_context,
        request,
        hook_data,
    )
    .await
    .map_err(CrudError::from)?;

    match read_result {
        ReadResult::One(entity) => Ok(entity),
        _ => unreachable!("after_read should not change result type"),
    }
}

/// Read multiple entities matching the given criteria.
#[tracing::instrument(level = "info", skip(context, request))]
pub async fn read_many<R: CrudResource>(
    request: RequestContext<R::Auth>,
    context: Arc<CrudContext<R>>,
    body: ReadMany<R>,
) -> Result<Vec<R::ReadModel>, CrudError> {
    let mut read_request = ReadRequest {
        operation: ReadOperation::Many,
        limit: body.limit,
        skip: body.skip,
        order_by: body.order_by,
        condition: body.condition,
    };

    let hook_data = R::HookData::default();

    let hook_data = R::Lifetime::before_read(
        &mut read_request,
        &context.res_context,
        request.clone(),
        hook_data,
    )
    .await
    .map_err(CrudError::from)?;

    let entities = context
        .repository
        .read_many(
            read_request.limit,
            read_request.skip,
            read_request.order_by.clone(),
            read_request.condition.as_ref(),
        )
        .await
        .map_err(|err| CrudError::Repository {
            reason: Arc::new(err),
        });

    if let Err(err) = &entities {
        error!(resource = ?R::TYPE, "{err}");
    }

    let entities = entities?;

    let mut read_result = ReadResult::Many(entities);

    let _hook_data = R::Lifetime::after_read(
        &read_request,
        &mut read_result,
        &context.res_context,
        request,
        hook_data,
    )
    .await
    .map_err(CrudError::from)?;

    match read_result {
        ReadResult::Many(entities) => Ok(entities),
        _ => unreachable!("after_read should not change result type"),
    }
}
