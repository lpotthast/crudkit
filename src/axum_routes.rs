use std::sync::Arc;

use crate::{
    CreateOne, CrudController, CrudResource, DeleteById, DeleteMany, DeleteOne, ReadCount,
    ReadMany, ReadOne, UpdateOne,
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Extension, Json, Router,
};
use crud_shared_types::CrudError;
use sea_orm::JsonValue;
use serde_json::json;

// https://github.com/tokio-rs/axum/discussions/358
// states which requirements R must meet in order for this to compile!
pub fn add_crud_routes<R: CrudResource + Sync + Send + 'static>(
    root: &str,
    resource: &'static str,
    router: Router,
) -> Router {
    router
        .route(
            format!("{root}/{resource}/crud/read-count").as_str(),
            post(read_count::<R>),
        )
        .route(
            format!("{root}/{resource}/crud/read-one").as_str(),
            post(read_one::<R>),
        )
        .route(
            format!("{root}/{resource}/crud/read-many").as_str(),
            post(read_many::<R>),
        )
        .route(
            format!("{root}/{resource}/crud/create-one").as_str(),
            post(create_one::<R>),
        )
        .route(
            format!("{root}/{resource}/crud/update-one").as_str(),
            post(update_one::<R>),
        )
        .route(
            format!("{root}/{resource}/crud/delete-by-id").as_str(),
            post(delete_by_id::<R>),
        )
        .route(
            format!("{root}/{resource}/crud/delete-one").as_str(),
            post(delete_one::<R>),
        )
        .route(
            format!("{root}/{resource}/crud/delete-many").as_str(),
            post(delete_many::<R>),
        )
}

struct AxumCrudError(CrudError);

impl IntoResponse for AxumCrudError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self.0 {
            CrudError::UnknownColumnSpecified(column) => (
                StatusCode::BAD_REQUEST,
                format!("Column \"{}\" not found", column),
            ),
            CrudError::UnableToParseValueAsColType(column, error) => (
                StatusCode::BAD_REQUEST,
                format!(
                    "Could not parse value for column \"{}\" to column type: {}",
                    column, error
                ),
            ),
            CrudError::UnableToParseAsEntity(json, error) => (
                StatusCode::BAD_REQUEST,
                format!("JSON \"{}\" could not be parsed as entity: {}", json, error),
            ),
            CrudError::DbError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("A database error occurred: {}", error),
            ),
            CrudError::ReadOneFoundNone => (StatusCode::NOT_FOUND, format!("Entity not found")),
            CrudError::ValidationErrors(errors) => (
                StatusCode::BAD_REQUEST,
                format!("Validation errors occurred: {:?}", errors),
            ),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

async fn read_count<R: CrudResource>(
    Extension(ref controller): Extension<Arc<CrudController>>,
    Json(body): Json<ReadCount>,
) -> Result<Json<JsonValue>, AxumCrudError> {
    super::read::read_count::<R>(controller.clone(), body)
        .await
        .map(|res| Json(res))
        .map_err(|err| AxumCrudError(err))
}

async fn read_one<R: CrudResource>(
    Extension(ref controller): Extension<Arc<CrudController>>,
    Json(body): Json<ReadOne<R>>,
) -> Result<Json<JsonValue>, AxumCrudError> {
    super::read::read_one::<R>(controller.clone(), body)
        .await
        .map(|res| Json(res))
        .map_err(|err| AxumCrudError(err))
}

async fn read_many<R: CrudResource>(
    Extension(ref controller): Extension<Arc<CrudController>>,
    Json(body): Json<ReadMany<R>>,
) -> Result<Json<JsonValue>, AxumCrudError> {
    super::read::read_many::<R>(controller.clone(), body)
        .await
        .map(|res| Json(res))
        .map_err(|err| AxumCrudError(err))
}

async fn create_one<R: CrudResource>(
    Extension(ref controller): Extension<Arc<CrudController>>,
    Json(body): Json<CreateOne>,
) -> Result<Json<JsonValue>, AxumCrudError> {
    super::create::create_one::<R>(controller.clone(), body)
        .await
        .map(|res| Json(res))
        .map_err(|err| AxumCrudError(err))
}

async fn update_one<R: CrudResource>(
    Extension(ref controller): Extension<Arc<CrudController>>,
    Json(body): Json<UpdateOne>,
) -> Result<Json<JsonValue>, AxumCrudError> {
    super::update::update_one::<R>(controller.clone(), body)
        .await
        .map(|res| Json(res))
        .map_err(|err| AxumCrudError(err))
}

async fn delete_by_id<R: CrudResource>(
    Extension(ref controller): Extension<Arc<CrudController>>,
    Json(body): Json<DeleteById>,
) -> Result<Json<JsonValue>, AxumCrudError> {
    super::delete::delete_by_id::<R>(controller.clone(), body)
        .await
        .map(|res| Json(res))
        .map_err(|err| AxumCrudError(err))
}

async fn delete_one<R: CrudResource>(
    Extension(ref controller): Extension<Arc<CrudController>>,
    Json(body): Json<DeleteOne<R>>,
) -> Result<Json<JsonValue>, AxumCrudError> {
    super::delete::delete_one::<R>(controller.clone(), body)
        .await
        .map(|res| Json(res))
        .map_err(|err| AxumCrudError(err))
}

async fn delete_many<R: CrudResource>(
    Extension(ref controller): Extension<Arc<CrudController>>,
    Json(body): Json<DeleteMany>,
) -> Result<Json<JsonValue>, AxumCrudError> {
    super::delete::delete_many::<R>(controller.clone(), body)
        .await
        .map(|res| Json(res))
        .map_err(|err| AxumCrudError(err))
}
