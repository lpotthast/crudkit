use crate::prelude::*;
use crud_shared_types::CrudError;
use sea_orm::JsonValue;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CreateOne {
    pub entity: Box<serde_json::value::RawValue>,
}

pub async fn create_one<R: CrudResource>(
    controller: Arc<CrudController>,
    context: Arc<CrudContext<R>>,
    body: CreateOne,
) -> Result<JsonValue, CrudError> {
    let entity_json: &str = body.entity.get();

    // Use the "CreateModel" to deserialize the given JSON. Some not required members are allowed to be missing.
    let create_model: R::CreateModel = serde_json::from_str::<R::CreateModel>(entity_json)
        .map_err(|err| CrudError::UnableToParseAsEntity(entity_json.to_owned(), err.to_string()))?;

    // Convert the "CreateModel" into the actual "Model"
    let model: R::Model = create_model.into();

    // Create the "Model" into an "ActiveModel" ready to be persisted.
    let mut active_entity: R::ActiveModel = model.into();

    // We might have accidentally set attributes on the "ActiveModel" that we must not have in order to run validations.
    prune_active_model::<R>(&mut active_entity);

    // Run validations before inserting the entity.
    let validation_results: EntityValidations = context.validator.validate_single(&active_entity);
    if validation_results.has_violations() {
        // TODO: pass violations to frontend.
        log::info!("Validation errors: {:?}", validation_results);
        let persistable = validation_results.into();
        context.validation_result_repository.save_all(persistable).await;
        return Err(CrudError::ValidationErrors);
    }

    let insert_query = build_insert_query::<R>(active_entity)?;

    let inserted_entity = insert_query
        .exec_with_returning(controller.get_database_connection())
        .await
        .map_err(|err| CrudError::DbError(err.to_string()))?;

    Ok(json! {inserted_entity})
}
