use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use crud_shared_types::error::CrudError;
use serde_json::json;

// Note: We do not derive ToSchema, but instead use CrudError directly.
#[derive(Debug)]
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
            CrudError::DbError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("A database error occurred: {}", error),
            ),
            CrudError::ReadOneFoundNone => (StatusCode::NOT_FOUND, "Entity not found".to_owned()),
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
            pub mod [< axum_ $name _crud_routes >] {
                use std::sync::Arc;
                use crud_rs::prelude::*;
                use crud_rs::axum_routes::AxumCrudError;
                use axum::{
                    http::StatusCode,
                    response::{IntoResponse, Response},
                    routing::post,
                    Extension, Json, Router,
                };
                use crud_shared_types::error::CrudError;
                use crud_shared_types::{DeleteResult, SaveResult};
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

                /// Retrieve the amount of entities available.
                ///
                /// Counts the number of entities based on the provided condition.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/read-count",
                    request_body = ReadCount,
                    responses(
                        (status = 200, description = "count was successfully calculated", body = u64),
                        (status = 500, description = "count could not be calculated", body = CrudError),
                    ),
                )]
                async fn read_count(
                    Extension(ref controller): Extension<Arc<CrudController>>,
                    Extension(ref context): Extension<std::sync::Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<ReadCount>,
                ) -> Result<Json<u64>, AxumCrudError> {
                    crud_rs::read::read_count::<$resource_type>(controller.clone(), context.clone(), body)
                        .await
                        .map(|res| Json(res))
                        .map_err(|err| AxumCrudError(err))
                }

                /// Retrieve one entity.
                ///
                /// Retrieve one entity based on the given [crate::read::ReadOne] body.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/read-one",
                    request_body = ReadOne<$resource_type>,
                    responses(
                        (status = 200, description = "one entity was read", body = <$resource_type as CrudResource>::ReadViewModel),
                        (status = 500, description = "entity could not be read", body = CrudError),
                    ),
                )]
                async fn read_one(
                    Extension(ref controller): Extension<Arc<CrudController>>,
                    Extension(ref context): Extension<std::sync::Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<ReadOne<$resource_type>>,
                ) -> Result<Json<<$resource_type as CrudResource>::ReadViewModel>, AxumCrudError> {
                    crud_rs::read::read_one::<$resource_type>(controller.clone(), context.clone(), body)
                        .await
                        .map(|res| Json(res))
                        .map_err(|err| AxumCrudError(err))
                }

                /// Retrieve many entities.
                ///
                /// Retrieve many entities based on the given [crate::read::ReadMany] body.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/read-many",
                    request_body = ReadMany<$resource_type>,
                    responses(
                        (status = 200, description = "entities were read", body = Vec<<$resource_type as CrudResource>::ReadViewModel>),
                        (status = 500, description = "entities could not be read", body = CrudError),
                    ),
                )]
                async fn read_many(
                    Extension(ref controller): Extension<Arc<CrudController>>,
                    Extension(ref context): Extension<std::sync::Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<ReadMany<$resource_type>>,
                ) -> Result<Json<Vec<<$resource_type as CrudResource>::ReadViewModel>>, AxumCrudError> {
                    crud_rs::read::read_many::<$resource_type>(controller.clone(), context.clone(), body)
                        .await
                        .map(|res| Json(res))
                        .map_err(|err| AxumCrudError(err))
                }

                /// Create one entity.
                ///
                /// Create one entity based on the given [crate::create::CreateOne] body.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/create-one",
                    request_body = CreateOne<<$resource_type as CrudResource>::CreateModel>,
                    responses(
                        (status = 200, description = "entity was created", body = SaveResult<<$resource_type as CrudResource>::Model>),
                        (status = 500, description = "entity could not be created", body = CrudError),
                    ),
                )]
                async fn create_one(
                    Extension(ref controller): Extension<std::sync::Arc<CrudController>>,
                    Extension(ref context): Extension<std::sync::Arc<CrudContext<$resource_type>>>,
                    Extension(ref res_context): Extension<std::sync::Arc<<$resource_type as CrudResource>::Context>>,
                    Json(body): Json<CreateOne<<$resource_type as CrudResource>::CreateModel>>,
                ) -> Result<Json<SaveResult<<$resource_type as CrudResource>::Model>>, AxumCrudError> {
                    crud_rs::create::create_one::<$resource_type>(controller.clone(), context.clone(), res_context.clone(), body)
                        .await
                        .map(|res| Json(res))
                        .map_err(|err| AxumCrudError(err))
                }

                /// Update one entity.
                ///
                /// Update one entity based on the given [crate::create::UpdateOne] body.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/update-one",
                    request_body = UpdateOne<$resource_type>,
                    responses(
                        (status = 200, description = "entity was updated", body = SaveResult<<$resource_type as CrudResource>::Model>),
                        (status = 500, description = "entity could not be updated", body = CrudError),
                    ),
                )]
                async fn update_one(
                    Extension(ref controller): Extension<Arc<CrudController>>,
                    Extension(ref context): Extension<std::sync::Arc<CrudContext<$resource_type>>>,
                    Extension(ref res_context): Extension<std::sync::Arc<<$resource_type as CrudResource>::Context>>,
                    Json(body): Json<UpdateOne<$resource_type>>,
                ) -> Result<Json<SaveResult<<$resource_type as CrudResource>::Model>>, AxumCrudError> {
                    crud_rs::update::update_one::<$resource_type>(controller.clone(), context.clone(), res_context.clone(), body)
                        .await
                        .map(|res| Json(res))
                        .map_err(|err| AxumCrudError(err))
                }

                /// Delete one entity by id.
                ///
                /// Delete one entity based on the given [crate::delete::DeleteById] body.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/delete-by-id",
                    request_body = DeleteById,
                    responses(
                        (status = 200, description = "entity was deleted", body = DeleteResult),
                        (status = 500, description = "entity could not be deleted", body = CrudError),
                    ),
                )]
                async fn delete_by_id(
                    Extension(ref controller): Extension<Arc<CrudController>>,
                    Extension(ref context): Extension<std::sync::Arc<CrudContext<$resource_type>>>,
                    Extension(ref res_context): Extension<std::sync::Arc<<$resource_type as CrudResource>::Context>>,
                    Json(body): Json<DeleteById>,
                ) -> Result<Json<DeleteResult>, AxumCrudError> {
                    crud_rs::delete::delete_by_id::<$resource_type>(controller.clone(), context.clone(), res_context.clone(), body)
                        .await
                        .map(|res| Json(res))
                        .map_err(|err| AxumCrudError(err))
                }

                /// Delete one entity using a standard filter query.
                ///
                /// Delete one entity based on the given [crate::delete::DeleteOne] body.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/delete-one",
                    request_body = DeleteOne,
                    responses(
                        (status = 200, description = "entity was deleted", body = DeleteResult),
                        (status = 500, description = "entity could not be deleted", body = CrudError),
                    ),
                )]
                async fn delete_one(
                    Extension(ref controller): Extension<Arc<CrudController>>,
                    Extension(ref context): Extension<std::sync::Arc<CrudContext<$resource_type>>>,
                    Extension(ref res_context): Extension<std::sync::Arc<<$resource_type as CrudResource>::Context>>,
                    Json(body): Json<DeleteOne<$resource_type>>,
                ) -> Result<Json<DeleteResult>, AxumCrudError> {
                    crud_rs::delete::delete_one::<$resource_type>(controller.clone(), context.clone(), res_context.clone(), body)
                        .await
                        .map(|res| Json(res))
                        .map_err(|err| AxumCrudError(err))
                }

                /// Delete many entities using a standard filter query.
                ///
                /// Delete many entities based on the given [crate::delete::DeleteMany] body.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/delete-many",
                    request_body = DeleteMany,
                    responses(
                        (status = 200, description = "entities were deleted", body = DeleteResult),
                        (status = 500, description = "entities could not be deleted", body = CrudError),
                    ),
                )]
                async fn delete_many(
                    Extension(ref controller): Extension<Arc<CrudController>>,
                    Extension(ref context): Extension<std::sync::Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<DeleteMany>,
                ) -> Result<Json<DeleteResult>, AxumCrudError> {
                    crud_rs::delete::delete_many::<$resource_type>(controller.clone(), context.clone(), body)
                        .await
                        .map(|res| Json(res))
                        .map_err(|err| AxumCrudError(err))
                }

                use utoipa::OpenApi;

                // We define this Crt ('CrudResourceType') use statement, as `$resource_type` can not be used in the utoipa block below...
                use $resource_type as Crt;
                type CrtModel = <$resource_type as CrudResource>::Model;
                type CrtReadModel = <$resource_type as CrudResource>::ReadViewModel;
                type CrtCreateModel = <$resource_type as CrudResource>::CreateModel;

                #[derive(OpenApi)]
                #[openapi(
                    paths(
                        read_count,
                        read_one,
                        read_many,
                        create_one,
                        update_one,
                        delete_by_id,
                        delete_one,
                        delete_many,
                    ),
                    components(
                        schemas(<$resource_type as CrudResource>::Model as CrtModel),
                        schemas(<$resource_type as CrudResource>::ReadViewModel as CrtReadModel),
                        schemas(<$resource_type as CrudResource>::CreateModel as CrtCreateModel),
                        schemas(crud_shared_types::error::CrudError),
                        schemas(crud_shared_types::DeleteResult),
                        schemas(crud_shared_types::SaveResult<CrtModel>),
                        schemas(crud_shared_types::Saved<CrtModel>),
                        schemas(crud_shared_types::condition::Condition),
                        schemas(crud_shared_types::condition::ConditionElement),
                        schemas(crud_shared_types::condition::ConditionClause),
                        schemas(crud_shared_types::condition::ConditionClauseValue),
                        schemas(crud_shared_types::condition::Operator),
                        schemas(crud_shared_types::id::SerializableId),
                        schemas(crud_rs::create::CreateOne<CrtCreateModel>),
                        schemas(crud_rs::read::ReadCount),
                        schemas(crud_rs::read::ReadOne<Crt>),
                        schemas(crud_rs::read::ReadMany<Crt>),
                        schemas(crud_rs::update::UpdateOne<Crt>),
                        schemas(crud_rs::delete::DeleteById),
                        schemas(crud_rs::delete::DeleteOne<Crt>),
                        schemas(crud_rs::delete::DeleteMany),
                        schemas(chrono_utc_date_time::UtcDateTime),
                    )
                )]
                pub struct ApiDoc; // We just use `ApiDoc` instead of `[< $name _ApiDoc >]` as we are already in a named module.
            }
        }
    };
}
