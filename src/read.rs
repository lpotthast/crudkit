use crate::{CrudResource, ReadCount, ReadMany, ReadOne};
use crud_shared_types::CrudError;
use sea_orm::{JsonValue, PaginatorTrait};
use serde_json::json;
use std::sync::Arc;

use crate::{query::build_select_query, CrudController};

pub async fn read_count<R: CrudResource>(
    controller: Arc<CrudController>,
    body: ReadCount,
) -> Result<JsonValue, CrudError> {
    let count = build_select_query::<R::Entity, R::Model, R::Column, R::CrudColumn>(
        None,
        None,
        None,
        body.condition,
    )?
    .count(controller.db.as_ref())
    .await
    .map_err(|err| CrudError::DbError(err.to_string()))?;
    Ok(json!(count))
}

pub async fn read_one<R: CrudResource>(
    controller: Arc<CrudController>,
    body: ReadOne<R>,
) -> Result<JsonValue, CrudError> {
    let db = controller.db.as_ref();
    let data = build_select_query::<R::Entity, R::Model, R::Column, R::CrudColumn>(
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
    body: ReadMany<R>,
) -> Result<JsonValue, CrudError> {
    let data = build_select_query::<R::Entity, R::Model, R::Column, R::CrudColumn>(
        body.limit,
        body.skip,
        body.order_by,
        body.condition,
    )?
    .all(controller.db.as_ref())
    .await
    .unwrap();
    Ok(json!(data))
}
