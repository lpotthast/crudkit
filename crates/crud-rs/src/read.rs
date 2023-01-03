use crate::prelude::*;
use crud_shared_types::{condition::Condition, error::CrudError, Order};
use indexmap::IndexMap;
use sea_orm::PaginatorTrait;
use serde::Deserialize;
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

#[tracing::instrument(level = "info", skip(controller, _context))]
pub async fn read_count<R: CrudResource>(
    controller: Arc<CrudController>,
    _context: Arc<CrudContext<R>>,
    body: ReadCount,
) -> Result<u64, CrudError> {
    let count =
        build_select_query::<R::Entity, R::Model, R::ActiveModel, R::Column, R::CrudColumn>(
            None,
            None,
            None,
            &body.condition,
        )?
        .count(controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;
    Ok(count)
}

#[tracing::instrument(level = "info", skip(controller, _context))]
pub async fn read_one<R: CrudResource>(
    controller: Arc<CrudController>,
    _context: Arc<CrudContext<R>>,
    body: ReadOne<R>,
) -> Result<R::ReadViewModel, CrudError> {
    let db = controller.get_database_connection();
    let data = build_select_query::<
        R::ReadViewEntity,
        R::ReadViewModel,
        R::ReadViewActiveModel,
        R::ReadViewColumn,
        R::ReadViewCrudColumn,
    >(None, body.skip, body.order_by, &body.condition)?
    .one(db)
    .await
    .map_err(|err| CrudError::DbError(err.to_string()))?
    .ok_or(CrudError::ReadOneFoundNone)?;
    Ok(data)
}

#[tracing::instrument(level = "info", skip(controller, _context))]
pub async fn read_many<R: CrudResource>(
    controller: Arc<CrudController>,
    _context: Arc<CrudContext<R>>,
    body: ReadMany<R>,
) -> Result<Vec<R::ReadViewModel>, CrudError> {
    let data = build_select_query::<
        R::ReadViewEntity,
        R::ReadViewModel,
        R::ReadViewActiveModel,
        R::ReadViewColumn,
        R::ReadViewCrudColumn,
    >(body.limit, body.skip, body.order_by, &body.condition)?
    .all(controller.get_database_connection())
    .await
    .unwrap();
    Ok(data)
}
