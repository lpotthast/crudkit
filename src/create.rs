use crate::{CreateOne, CrudResource};
use crud_shared_types::CrudError;
use sea_orm::JsonValue;
use serde_json::json;
use std::sync::Arc;

use crate::{query::build_insert_query, CrudController};

pub async fn create_one<R: CrudResource>(
    controller: Arc<CrudController>,
    body: CreateOne,
) -> Result<JsonValue, CrudError> {
    let inserted_entity =
        build_insert_query::<R::CreateModel, R::Model, R::ActiveModel>(body.entity.get())?
            .exec_with_returning(controller.db.as_ref())
            .await
            .map_err(|err| CrudError::DbError(err.to_string()))?;
    Ok(json! {inserted_entity})
}
