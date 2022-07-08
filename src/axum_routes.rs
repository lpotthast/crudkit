use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use crud_shared_types::CrudError;
use serde_json::json;

pub struct AxumCrudError(pub CrudError);

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
            CrudError::ValidationErrors => (
                StatusCode::BAD_REQUEST,
                format!("Validation errors occurred"),
            ),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

#[macro_export]
macro_rules! impl_add_crud_routes {

    ($resource_type:ty, $name:ident) => {
        paste::item! {
            mod [< axum_ $name _crud_routes >] {
                use std::sync::Arc;
                use crud::prelude::*;
                use axum::{
                    http::StatusCode,
                    response::{IntoResponse, Response},
                    routing::post,
                    Extension, Json, Router,
                };
                use crud_shared_types::CrudError;
                use crud::axum_routes::AxumCrudError;
                use sea_orm::JsonValue;
                use serde_json::json;

                // https://github.com/tokio-rs/axum/discussions/358
                // states which requirements R must meet in order for this to compile!
                pub fn add_crud_routes(
                    root: &str,
                    mut router: Router,
                ) -> Router {
                    let resource: &'static str = <$resource_type as CrudResource>::TYPE.into();

                    let path = format!("{root}/{resource}/crud/read-count");
                    log::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(read_count));

                    let path = format!("{root}/{resource}/crud/read-one");
                    log::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(read_one));

                    let path = format!("{root}/{resource}/crud/read-many");
                    log::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(read_many));

                    let path = format!("{root}/{resource}/crud/create-one");
                    log::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(create_one));

                    let path = format!("{root}/{resource}/crud/update-one");
                    log::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(update_one));

                    let path = format!("{root}/{resource}/crud/delete-by-id");
                    log::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(delete_by_id));

                    let path = format!("{root}/{resource}/crud/delete-one");
                    log::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(delete_one));

                    let path = format!("{root}/{resource}/crud/delete-many");
                    log::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(delete_many));

                    router
                }

                async fn read_count(
                    Extension(ref controller): Extension<Arc<CrudController>>,
                    Extension(ref context): Extension<std::sync::Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<ReadCount>,
                ) -> Result<Json<JsonValue>, AxumCrudError> {
                    crud::read::read_count::<$resource_type>(controller.clone(), context.clone(), body)
                        .await
                        .map(|res| Json(res))
                        .map_err(|err| AxumCrudError(err))
                }

                async fn read_one(
                    Extension(ref controller): Extension<Arc<CrudController>>,
                    Extension(ref context): Extension<std::sync::Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<ReadOne<$resource_type>>,
                ) -> Result<Json<JsonValue>, AxumCrudError> {
                    crud::read::read_one::<$resource_type>(controller.clone(), context.clone(), body)
                        .await
                        .map(|res| Json(res))
                        .map_err(|err| AxumCrudError(err))
                }

                async fn read_many(
                    Extension(ref controller): Extension<Arc<CrudController>>,
                    Extension(ref context): Extension<std::sync::Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<ReadMany<$resource_type>>,
                ) -> Result<Json<JsonValue>, AxumCrudError> {
                    crud::read::read_many::<$resource_type>(controller.clone(), context.clone(), body)
                        .await
                        .map(|res| Json(res))
                        .map_err(|err| AxumCrudError(err))
                }

                async fn create_one(
                    Extension(ref controller): Extension<std::sync::Arc<CrudController>>,
                    Extension(ref context): Extension<std::sync::Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<CreateOne>,
                ) -> Result<Json<JsonValue>, AxumCrudError> {
                    crud::create::create_one::<$resource_type>(controller.clone(), context.clone(), body)
                        .await
                        .map(|res| Json(res))
                        .map_err(|err| AxumCrudError(err))
                }

                async fn update_one(
                    Extension(ref controller): Extension<Arc<CrudController>>,
                    Extension(ref context): Extension<std::sync::Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<UpdateOne>,
                ) -> Result<Json<JsonValue>, AxumCrudError> {
                    crud::update::update_one::<$resource_type>(controller.clone(), context.clone(), body)
                        .await
                        .map(|res| Json(res))
                        .map_err(|err| AxumCrudError(err))
                }

                async fn delete_by_id(
                    Extension(ref controller): Extension<Arc<CrudController>>,
                    Extension(ref context): Extension<std::sync::Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<DeleteById>,
                ) -> Result<Json<JsonValue>, AxumCrudError> {
                    crud::delete::delete_by_id::<$resource_type>(controller.clone(), context.clone(), body)
                        .await
                        .map(|res| Json(res))
                        .map_err(|err| AxumCrudError(err))
                }

                async fn delete_one(
                    Extension(ref controller): Extension<Arc<CrudController>>,
                    Extension(ref context): Extension<std::sync::Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<DeleteOne<$resource_type>>,
                ) -> Result<Json<JsonValue>, AxumCrudError> {
                    crud::delete::delete_one::<$resource_type>(controller.clone(), context.clone(), body)
                        .await
                        .map(|res| Json(res))
                        .map_err(|err| AxumCrudError(err))
                }

                async fn delete_many(
                    Extension(ref controller): Extension<Arc<CrudController>>,
                    Extension(ref context): Extension<std::sync::Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<DeleteMany>,
                ) -> Result<Json<JsonValue>, AxumCrudError> {
                    crud::delete::delete_many::<$resource_type>(controller.clone(), context.clone(), body)
                        .await
                        .map(|res| Json(res))
                        .map_err(|err| AxumCrudError(err))
                }
            }
        }
    };
}
