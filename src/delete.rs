use crate::prelude::*;
use crud_shared_types::{
    Condition, ConditionClause, ConditionClauseValue, ConditionElement, CrudError, Operator, Order,
};
use indexmap::IndexMap;
use sea_orm::{JsonValue, ModelTrait};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct DeleteById {
    pub id: u32,
}

#[derive(Deserialize)]
pub struct DeleteOne<R: CrudResource> {
    pub skip: Option<u64>,
    pub order_by: Option<IndexMap<R::CrudColumn, Order>>,
    pub condition: Option<Condition>,
}

#[derive(Deserialize)]
pub struct DeleteMany {
    pub condition: Option<Condition>,
}

pub async fn delete_by_id<R: CrudResource>(
    controller: Arc<CrudController>,
    _context: Arc<CrudContext<R>>,
    body: DeleteById,
) -> Result<JsonValue, CrudError> {
    let select = build_select_query::<R::Entity, R::Model, R::ActiveModel, R::Column, R::CrudColumn>(
        None,
        None,
        None,
        Some(Condition::All(vec![ConditionElement::Clause(
            ConditionClause {
                column_name: R::CrudColumn::get_id_field_name(),
                operator: Operator::Equal,
                value: ConditionClauseValue::I32(body.id.try_into().unwrap()),
            },
        )])),
    )?;
    let data = select
        .one(controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?
        .ok_or_else(|| CrudError::ReadOneFoundNone)?;
    let delete_result = R::Model::delete(data, controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;
    Ok(json!(delete_result.rows_affected))
}

pub async fn delete_one<R: CrudResource>(
    controller: Arc<CrudController>,
    _context: Arc<CrudContext<R>>,
    body: DeleteOne<R>,
) -> Result<JsonValue, CrudError> {
    let select = build_select_query::<R::Entity, R::Model, R::ActiveModel, R::Column, R::CrudColumn>(
        None,
        body.skip,
        body.order_by,
        body.condition,
    )?;
    let data = select
        .one(controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?
        .ok_or_else(|| CrudError::ReadOneFoundNone)?;
    let delete_result = R::Model::delete(data, controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;
    Ok(json!(delete_result.rows_affected))
}

pub async fn delete_many<R: CrudResource>(
    controller: Arc<CrudController>,
    _context: Arc<CrudContext<R>>,
    body: DeleteMany,
) -> Result<JsonValue, CrudError> {
    let delete_many = build_delete_many_query::<R::Entity>(body.condition)?;
    let delete_result = delete_many
        .exec(controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;
    Ok(json!(delete_result.rows_affected))
}
