use crate::{CrudResource, UpdateActiveModelTrait, UpdateOne};
use crud_shared_types::CrudError;
use sea_orm::{ActiveModelTrait, JsonValue};
use serde_json::json;
use std::sync::Arc;

use crate::{query::build_select_query, CrudController};

pub async fn update_one<R: CrudResource>(
    controller: Arc<CrudController>,
    body: UpdateOne,
) -> Result<JsonValue, CrudError> {
    let db = controller.db.as_ref();
    let model = build_select_query::<R::Entity, R::Model, R::Column, R::CrudColumn>(
        None,
        None,
        None,
        body.condition,
    )?
    .one(db)
    .await
    .map_err(|err| CrudError::DbError(err.to_string()))?
    .ok_or_else(|| CrudError::ReadOneFoundNone)?;

    // Use the "CreateModel" to deserialize the given JSON. Some not required members are allowed to be missing.
    let update = serde_json::from_str::<R::CreateModel>(body.entity.get()).map_err(|err| {
        CrudError::UnableToParseAsEntity(body.entity.get().to_owned(), err.to_string())
    })?;

    // Update the persisted active_model!
    let mut active_model: R::ActiveModel = model.into();
    active_model.update_with(update);

    let result = active_model
        .update(db)
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;

    Ok(json!(result))
}
