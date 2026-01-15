use crate::{
    auth::RequestContext,
    error::CrudError,
    lifetime::{Abort, CrudLifetime, ReadOperation, ReadRequest, ReadResult},
    prelude::*,
};

use crudkit_condition::Condition;
use crudkit_core::Order;

use indexmap::IndexMap;
use serde::Deserialize;
use snafu::{Backtrace, GenerateImplicitData};
use std::sync::Arc;
use tracing::error;
use utoipa::ToSchema;

#[derive(Debug, ToSchema, Deserialize)]
pub struct ReadCount {
    pub condition: Option<Condition>,
}

#[derive(Debug, ToSchema, Deserialize)]
pub struct ReadOne<R: CrudResource> {
    pub skip: Option<u64>,
    #[serde(bound = "")]
    #[schema(value_type = Option<Object>, example = json!({"id": Order::Asc}))]
    pub order_by: Option<IndexMap<R::ReadViewCrudColumn, Order>>,
    pub condition: Option<Condition>,
}

#[derive(Debug, ToSchema, Deserialize)]
pub struct ReadMany<R: CrudResource> {
    pub limit: Option<u64>,
    pub skip: Option<u64>,
    #[serde(bound = "")]
    #[schema(value_type = Option<Object>, example = json!({"id": Order::Asc}))]
    pub order_by: Option<IndexMap<R::ReadViewCrudColumn, Order>>,
    pub condition: Option<Condition>,
}

#[tracing::instrument(level = "info", skip(context, request))]
pub async fn read_count<R: CrudResource>(
    request: RequestContext<R::Auth>,
    context: Arc<CrudContext<R>>,
    body: ReadCount,
) -> Result<u64, CrudError> {
    // Build ReadRequest from body
    let mut read_request = ReadRequest {
        operation: ReadOperation::Count,
        limit: None,
        skip: None,
        order_by: None,
        condition: body.condition,
    };

    // Initialize hook data and call before_read
    let hook_data = R::HookData::default();

    // TODO: Do not ignore error! What to do? Should an error roll back a transaction. Should the error tell us that!?
    let (abort, hook_data) = R::Lifetime::before_read(
        &mut read_request,
        &context.res_context,
        request.clone(),
        hook_data,
    )
    .await
    .expect("before_read to not error");

    if let Abort::Yes { reason } = abort {
        return Err(CrudError::ReadAborted {
            reason,
            backtrace: Backtrace::generate(),
        });
    }

    // Execute query with potentially modified condition
    let count = context
        .repository
        .count(None, None, None, read_request.condition.as_ref())
        .await
        .map_err(|err| CrudError::Repository {
            reason: Arc::new(err),
            backtrace: Backtrace::generate(),
        })?;

    // Call after_read hook
    let mut read_result = ReadResult::Count(count);
    // TODO: Do not ignore error! What to do? Should an error roll back a transaction. Should the error tell us that!?
    let _hook_data = R::Lifetime::after_read(
        &read_request,
        &mut read_result,
        &context.res_context,
        request,
        hook_data,
    )
    .await
    .expect("after_read to not error");

    // Extract final count
    match read_result {
        ReadResult::Count(n) => Ok(n),
        _ => unreachable!("after_read should not change result type"),
    }
}

#[tracing::instrument(level = "info", skip(context, request))]
pub async fn read_one<R: CrudResource>(
    request: RequestContext<R::Auth>,
    context: Arc<CrudContext<R>>,
    body: ReadOne<R>,
) -> Result<R::ReadViewModel, CrudError> {
    // Build ReadRequest from body
    let mut read_request = ReadRequest {
        operation: ReadOperation::One,
        limit: None,
        skip: body.skip,
        order_by: body.order_by,
        condition: body.condition,
    };

    // Initialize hook data and call before_read
    let hook_data = R::HookData::default();
    // TODO: Do not ignore error! What to do? Should an error roll back a transaction. Should the error tell us that!?
    let (abort, hook_data) = R::Lifetime::before_read(
        &mut read_request,
        &context.res_context,
        request.clone(),
        hook_data,
    )
    .await
    .expect("before_read to not error");

    if let Abort::Yes { reason } = abort {
        return Err(CrudError::ReadAborted {
            reason,
            backtrace: Backtrace::generate(),
        });
    }

    // Execute query with potentially modified parameters
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
            backtrace: Backtrace::generate(),
        })?
        .ok_or_else(|| CrudError::ReadOneFoundNone {
            backtrace: Backtrace::generate(),
        })?;

    // Call after_read hook
    let mut read_result = ReadResult::One(entity);
    // TODO: Do not ignore error! What to do? Should an error roll back a transaction. Should the error tell us that!?
    let _hook_data = R::Lifetime::after_read(
        &read_request,
        &mut read_result,
        &context.res_context,
        request,
        hook_data,
    )
    .await
    .expect("after_read to not error");

    // Extract final entity
    match read_result {
        ReadResult::One(entity) => Ok(entity),
        _ => unreachable!("after_read should not change result type"),
    }
}

#[tracing::instrument(level = "info", skip(context, request))]
pub async fn read_many<R: CrudResource>(
    request: RequestContext<R::Auth>,
    context: Arc<CrudContext<R>>,
    body: ReadMany<R>,
) -> Result<Vec<R::ReadViewModel>, CrudError> {
    // Build ReadRequest from body
    let mut read_request = ReadRequest {
        operation: ReadOperation::Many,
        limit: body.limit,
        skip: body.skip,
        order_by: body.order_by,
        condition: body.condition,
    };

    // Initialize hook data and call before_read
    let hook_data = R::HookData::default();
    // TODO: Do not ignore error! What to do? Should an error roll back a transaction. Should the error tell us that!?
    let (abort, hook_data) = R::Lifetime::before_read(
        &mut read_request,
        &context.res_context,
        request.clone(),
        hook_data,
    )
    .await
    .expect("before_read to not error");

    if let Abort::Yes { reason } = abort {
        return Err(CrudError::ReadAborted {
            reason,
            backtrace: Backtrace::generate(),
        });
    }

    // Execute query with potentially modified parameters
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
            backtrace: Backtrace::generate(),
        });

    if let Err(err) = &entities {
        error!(resource = ?R::TYPE, "{err}");
    }

    let entities = entities?;

    // Call after_read hook
    let mut read_result = ReadResult::Many(entities);
    // TODO: Do not ignore error! What to do? Should an error roll back a transaction. Should the error tell us that!?
    let _hook_data = R::Lifetime::after_read(
        &read_request,
        &mut read_result,
        &context.res_context,
        request,
        hook_data,
    )
    .await
    .expect("after_read to not error");

    // Extract final entities
    match read_result {
        ReadResult::Many(entities) => Ok(entities),
        _ => unreachable!("after_read should not change result type"),
    }
}
