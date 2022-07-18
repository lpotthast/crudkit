use crate::prelude::*;
use crud_shared_types::{Condition, CrudError, Order};
use indexmap::IndexMap;
use sea_orm::{JsonValue, PaginatorTrait};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct ReadCount {
    pub condition: Option<Condition>,
}

#[derive(Deserialize)]
pub struct ReadOne<R: CrudResource> {
    pub skip: Option<u64>,
    #[serde(bound = "")]
    pub order_by: Option<IndexMap<R::ReadViewCrudColumn, Order>>,
    pub condition: Option<Condition>,
}

#[derive(Deserialize)]
pub struct ReadMany<R: CrudResource> {
    pub limit: Option<u64>,
    pub skip: Option<u64>,
    #[serde(bound = "")]
    pub order_by: Option<IndexMap<R::ReadViewCrudColumn, Order>>,
    pub condition: Option<Condition>,
}

pub async fn read_count<R: CrudResource>(
    controller: Arc<CrudController>,
    context: Arc<CrudContext<R>>,
    body: ReadCount,
) -> Result<JsonValue, CrudError> {
    let count =
        build_select_query::<R::Entity, R::Model, R::ActiveModel, R::Column, R::CrudColumn>(
            None,
            None,
            None,
            body.condition,
        )?
        .count(controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;
    Ok(json!(count))
}

pub async fn read_one<R: CrudResource>(
    controller: Arc<CrudController>,
    context: Arc<CrudContext<R>>,
    body: ReadOne<R>,
) -> Result<JsonValue, CrudError> {
    let db = controller.get_database_connection();
    let data = build_select_query::<R::ReadViewEntity, R::ReadViewModel, R::ReadViewActiveModel, R::ReadViewColumn, R::ReadViewCrudColumn>(
        None,
        body.skip,
        body.order_by,
        body.condition,
    )?
    .one(db)
    .await
    .map_err(|err| CrudError::DbError(err.to_string()))?
    .ok_or_else(|| CrudError::ReadOneFoundNone)?;
    Ok(json!(data))
}

pub async fn read_many<R: CrudResource>(
    controller: Arc<CrudController>,
    context: Arc<CrudContext<R>>,
    body: ReadMany<R>,
) -> Result<JsonValue, CrudError> {
    let data = build_select_query::<R::ReadViewEntity, R::ReadViewModel, R::ReadViewActiveModel, R::ReadViewColumn, R::ReadViewCrudColumn>(
        body.limit,
        body.skip,
        body.order_by,
        body.condition,
    )?
    .all(controller.get_database_connection())
    .await
    .unwrap();
    Ok(json!(data))
}
