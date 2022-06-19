use crate::{CrudColumns, CrudResource, DeleteById, DeleteMany, DeleteOne};
use crud_shared_types::{
    Condition, ConditionClause, ConditionClauseValue, ConditionElement, CrudError, Operator,
};
use sea_orm::{JsonValue, ModelTrait};
use serde_json::json;
use std::sync::Arc;

use crate::{
    query::{build_delete_many_query, build_select_query},
    CrudController,
};

pub async fn delete_by_id<R: CrudResource>(
    controller: Arc<CrudController>,
    body: DeleteById,
) -> Result<JsonValue, CrudError> {
    let select = build_select_query::<R::Entity, R::Model, R::Column, R::CrudColumn>(
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
        .one(controller.db.as_ref())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?
        .ok_or_else(|| CrudError::ReadOneFoundNone)?;
    let delete_result = R::Model::delete(data, controller.db.as_ref())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;
    Ok(json!(delete_result.rows_affected))
}

pub async fn delete_one<R: CrudResource>(
    controller: Arc<CrudController>,
    body: DeleteOne<R>,
) -> Result<JsonValue, CrudError> {
    let select = build_select_query::<R::Entity, R::Model, R::Column, R::CrudColumn>(
        None,
        body.skip,
        body.order_by,
        body.condition,
    )?;
    let data = select
        .one(controller.db.as_ref())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?
        .ok_or_else(|| CrudError::ReadOneFoundNone)?;
    let delete_result = R::Model::delete(data, controller.db.as_ref())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;
    Ok(json!(delete_result.rows_affected))
}

pub async fn delete_many<R: CrudResource>(
    controller: Arc<CrudController>,
    body: DeleteMany,
) -> Result<JsonValue, CrudError> {
    let delete_many = build_delete_many_query::<R::Entity>(body.condition)?;
    let delete_result = delete_many
        .exec(controller.db.as_ref())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;
    Ok(json!(delete_result.rows_affected))
}
