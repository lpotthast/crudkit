use crate::{error::CrudError, prelude::*};
use crud_shared_types::{condition::Condition, Order};
use indexmap::IndexMap;
use serde::Deserialize;
use snafu::{Backtrace, GenerateImplicitData};
use std::sync::Arc;
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

#[tracing::instrument(level = "info", skip(controller, context))]
pub async fn read_count<R: CrudResource>(
    controller: Arc<CrudController>,
    context: Arc<CrudContext<R>>,
    body: ReadCount,
) -> Result<u64, CrudError> {
    context
        .repository
        .count(None, None, None, body.condition.as_ref())
        .await
        .map_err(|err| CrudError::Repository {
            reason: Arc::new(err),
            backtrace: Backtrace::generate(),
        })
}

#[tracing::instrument(level = "info", skip(controller, context))]
pub async fn read_one<R: CrudResource>(
    controller: Arc<CrudController>,
    context: Arc<CrudContext<R>>,
    body: ReadOne<R>,
) -> Result<R::ReadViewModel, CrudError> {
    context
        .repository
        .read_one(None, body.skip, body.order_by, body.condition.as_ref())
        .await
        .map_err(|err| CrudError::Repository {
            reason: Arc::new(err),
            backtrace: Backtrace::generate(),
        })?
        .ok_or_else(|| CrudError::ReadOneFoundNone {
            backtrace: Backtrace::generate(),
        })
}

#[tracing::instrument(level = "info", skip(controller, context))]
pub async fn read_many<R: CrudResource>(
    controller: Arc<CrudController>,
    context: Arc<CrudContext<R>>,
    body: ReadMany<R>,
) -> Result<Vec<R::ReadViewModel>, CrudError> {
    context
        .repository
        .read_many(
            body.limit,
            body.skip,
            body.order_by,
            body.condition.as_ref(),
        )
        .await
        .map_err(|err| CrudError::Repository {
            reason: Arc::new(err),
            backtrace: Backtrace::generate(),
        })
}
