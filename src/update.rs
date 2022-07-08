use crate::prelude::*;
use crud_shared_types::{Condition, CrudError};
use sea_orm::{ActiveModelTrait, JsonValue};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct UpdateOne {
    pub condition: Option<Condition>,
    pub entity: Box<serde_json::value::RawValue>,
}

pub async fn update_one<R: CrudResource>(
    controller: Arc<CrudController>,
    context: Arc<CrudContext<R>>,
    body: UpdateOne,
) -> Result<JsonValue, CrudError> {
    let model = build_select_query::<R::Entity, R::Model, R::Column, R::CrudColumn>(
        None,
        None,
        None,
        body.condition,
    )?
    .one(controller.get_database_connection())
    .await
    .map_err(|err| CrudError::DbError(err.to_string()))?
    .ok_or_else(|| CrudError::ReadOneFoundNone)?;

    // Use the "CreateModel" to deserialize the given JSON. Some not required members are allowed to be missing.
    let update = serde_json::from_str::<R::CreateModel>(body.entity.get()).map_err(|err| {
        CrudError::UnableToParseAsEntity(body.entity.get().to_owned(), err.to_string())
    })?;

    // Convert the model into an ActiveModel, allowing mutations.
    let mut active_model: R::ActiveModel = model.into();

    // Update the persisted active_model!
    active_model.update_with(update);

    // Run validations ON THE NEW STATE(!) but before updating the entity in the database.
    let validation_results: EntityValidations = context.validator.validate_single(&active_model);
    if validation_results.has_violations() {
        // TODO: pass violations to frontend.
        log::info!("Validation errors: {:?}", validation_results);
        let persistable = validation_results.into();
        context.validation_result_repository.save_all(persistable).await;
        return Err(CrudError::ValidationErrors);
    }

    let result = active_model
        .update(controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;

    Ok(json!(result))
}
