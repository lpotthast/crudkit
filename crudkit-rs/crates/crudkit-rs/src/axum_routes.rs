use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use utoipa::ToSchema;

use crate::error::CrudError;

#[derive(Debug, ToSchema)]
pub enum AxumCrudError {
    Repository { reason: String },
    ReadOneFoundNone { reason: String },
    SaveValidations { reason: String },
    DeleteValidations { reason: String },
}

impl From<CrudError> for AxumCrudError {
    fn from(value: CrudError) -> Self {
        match value {
            err @ CrudError::Repository {
                reason: _,
                backtrace: _,
            } => Self::Repository {
                reason: err.to_string(),
            },
            err @ CrudError::ReadOneFoundNone { backtrace: _ } => Self::ReadOneFoundNone {
                reason: err.to_string(),
            },
            err @ CrudError::SaveValidations {
                reason: _,
                backtrace: _,
            } => Self::SaveValidations {
                reason: err.to_string(),
            },
            err @ CrudError::DeleteValidations {
                reason: _,
                backtrace: _,
            } => Self::DeleteValidations {
                reason: err.to_string(),
            },
        }
    }
}

// TODO: Use reporting mechanism (https://docs.rs/snafu/latest/snafu/attr.report.html)
impl IntoResponse for AxumCrudError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::Repository { reason } => (StatusCode::INTERNAL_SERVER_ERROR, reason),
            Self::ReadOneFoundNone { reason } => (StatusCode::NOT_FOUND, reason),
            Self::SaveValidations { reason } => (StatusCode::INTERNAL_SERVER_ERROR, reason),
            Self::DeleteValidations { reason } => (StatusCode::INTERNAL_SERVER_ERROR, reason),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

// TODO: On error, e REPORT must be generated, containing all error sources!

#[macro_export]
macro_rules! impl_add_crud_routes {

    ($resource_type:ty, $role_type:ty, $name:ident) => {
        paste::item! {
            pub mod [< axum_ $name _crud_routes >] {
                use std::sync::Arc;
                use crudkit_rs::prelude::*;
                use crudkit_shared::{DeleteResult, SaveResult}; // TODO: Rely on crudkit_rs::prelude use?
                use axum::{
                    http::StatusCode,
                    response::{IntoResponse, Response},
                    routing::post,
                    Extension, Json, Router,
                };
                use axum_keycloak_auth::decode::KeycloakToken;
                use sea_orm::JsonValue;
                use serde_json::json;

                // We define this 'ResourceType' use statement, as `$resource_type` can not be used in the utoipa block below...
                use $resource_type as ResourceType;
                type Context = <$resource_type as CrudResource>::Context;
                type ReadViewModel = <$resource_type as CrudResource>::ReadViewModel;
                type CreateModel = <$resource_type as CrudResource>::CreateModel;
                type Model = <$resource_type as CrudResource>::Model;
                type UpdateModel = <$resource_type as CrudResource>::UpdateModel;

                // https://github.com/tokio-rs/axum/discussions/358
                // states which requirements R must meet in order for this to compile!
                pub fn add_crud_routes(
                    root: &str,
                    mut router: Router,
                ) -> Router {
                    let resource: &'static str = <$resource_type as CrudResource>::TYPE.into();

                    let path = format!("{root}/{resource}/crud/read-count");
                    tracing::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(read_count));

                    let path = format!("{root}/{resource}/crud/read-one");
                    tracing::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(read_one));

                    let path = format!("{root}/{resource}/crud/read-many");
                    tracing::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(read_many));

                    let path = format!("{root}/{resource}/crud/create-one");
                    tracing::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(create_one));

                    let path = format!("{root}/{resource}/crud/update-one");
                    tracing::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(update_one));

                    let path = format!("{root}/{resource}/crud/delete-by-id");
                    tracing::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(delete_by_id));

                    let path = format!("{root}/{resource}/crud/delete-one");
                    tracing::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(delete_one));

                    let path = format!("{root}/{resource}/crud/delete-many");
                    tracing::debug!("Adding route: {}", path);
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
                    //responses(
                    //    (status = 200, description = "count was successfully calculated", body = u64),
                    //    (status = 500, description = "count could not be calculated", body = String),
                    //),
                )]
                #[axum_macros::debug_handler]
                async fn read_count(
                    Extension(keycloak_token): Extension<KeycloakToken<$role_type>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<ReadCount>,
                ) -> Response {
                    let result: Result<u64, AxumCrudError> = crudkit_rs::read::read_count::<$resource_type, $role_type>(keycloak_token, context.clone(), body)
                        .await
                        .map_err(Into::into);
                    match result {
                        Ok(count) => (StatusCode::OK, Json(count)).into_response(),
                        Err(err) => {
                            tracing::error!(?err, "Could not perform CRUD operation: read count.");
                            err.into_response()
                        },
                    }
                }

                /// Retrieve one entity.
                ///
                /// Retrieve one entity based on the given [crate::read::ReadOne] body.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/read-one",
                    request_body = ReadOne<$resource_type>,
                    //responses(
                    //    (status = 200, description = "one entity was read", body = ReadViewModel),
                    //    (status = 500, description = "entity could not be read", body = AxumCrudError),
                    //),
                )]
                #[axum_macros::debug_handler]
                async fn read_one(
                    Extension(keycloak_token): Extension<KeycloakToken<$role_type>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<ReadOne<$resource_type>>,
                ) -> Response {
                    let result: Result<ReadViewModel, AxumCrudError> = crudkit_rs::read::read_one::<$resource_type, $role_type>(keycloak_token, context.clone(), body)
                        .await
                        .map_err(Into::into);
                    match result {
                        Ok(date) => (StatusCode::OK, Json(date)).into_response(),
                        Err(err) => {
                            tracing::error!(?err, "Could not perform CRUD operation: read one.");
                            err.into_response()
                        },
                    }
                }

                /// Retrieve many entities.
                ///
                /// Retrieve many entities based on the given [crate::read::ReadMany] body.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/read-many",
                    request_body = ReadMany<$resource_type>,
                    //responses(
                    //    (status = 200, description = "entities were read", body = Vec<ReadViewModel>),
                    //    (status = 500, description = "entities could not be read", body = AxumCrudError),
                    //),
                )]
                #[axum_macros::debug_handler]
                async fn read_many(
                    Extension(keycloak_token): Extension<KeycloakToken<$role_type>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<ReadMany<$resource_type>>,
                ) -> Response {
                    let result: Result<Vec<ReadViewModel>, AxumCrudError> = crudkit_rs::read::read_many::<$resource_type, $role_type>(keycloak_token, context.clone(), body)
                        .await
                        .map_err(Into::into);
                    match result {
                        Ok(date) => (StatusCode::OK, Json(date)).into_response(),
                        Err(err) => {
                            tracing::error!(?err, "Could not perform CRUD operation: read many.");
                            err.into_response()
                        },
                    }
                }

                /// Create one entity.
                ///
                /// Create one entity based on the given [crate::create::CreateOne] body.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/create-one",
                    request_body = CreateOne<CreateModel>,
                    //responses(
                    //    (status = 200, description = "entity was created", body = SaveResult<Model>),
                    //    (status = 500, description = "entity could not be created", body = AxumCrudError),
                    //),
                )]
                #[axum_macros::debug_handler]
                async fn create_one(
                    Extension(keycloak_token): Extension<KeycloakToken<$role_type>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    Extension(res_context): Extension<Arc<Context>>,
                    Json(body): Json<CreateOne<CreateModel>>,
                ) -> Response {
                    let result: Result<SaveResult<Model>, AxumCrudError> = crudkit_rs::create::create_one::<$resource_type, $role_type>(keycloak_token, context.clone(), res_context.clone(), body)
                        .await
                        .map_err(Into::into);
                    match result {
                        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
                        Err(err) => {
                            tracing::error!(?err, "Could not perform CRUD operation: create one.");
                            err.into_response()
                        },
                    }
                }

                /// Update one entity.
                ///
                /// Update one entity based on the given [crate::create::UpdateOne] body.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/update-one",
                    request_body = UpdateOne<UpdateModel>,
                    //responses(
                    //    (status = 200, description = "entity was updated", body = SaveResult<Model>),
                    //    (status = 500, description = "entity could not be updated", body = String),
                    //),
                )]
                #[axum_macros::debug_handler]
                async fn update_one(
                    Extension(keycloak_token): Extension<KeycloakToken<$role_type>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    Extension(res_context): Extension<Arc<Context>>,
                    Json(body): Json<UpdateOne<UpdateModel>>,
                ) -> Response {
                    let result: Result<SaveResult<Model>, AxumCrudError> = crudkit_rs::update::update_one::<$resource_type, $role_type>(keycloak_token, context.clone(), res_context.clone(), body)
                        .await
                        .map_err(Into::into);
                    match result {
                        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
                        Err(err) => {
                            tracing::error!(?err, "Could not perform CRUD operation: update.");
                            err.into_response()
                        },
                    }
                }

                /// Delete one entity by id.
                ///
                /// Delete one entity based on the given [crate::delete::DeleteById] body.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/delete-by-id",
                    request_body = DeleteById,
                    //responses(
                    //    (status = 200, description = "entity was deleted", body = DeleteResult),
                    //    (status = 500, description = "entity could not be deleted", body = String),
                    //),
                )]
                #[axum_macros::debug_handler]
                async fn delete_by_id(
                    Extension(keycloak_token): Extension<KeycloakToken<$role_type>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    Extension(res_context): Extension<Arc<Context>>,
                    Json(body): Json<DeleteById>,
                ) -> Response {
                    let result: Result<DeleteResult, AxumCrudError> = crudkit_rs::delete::delete_by_id::<$resource_type, $role_type>(keycloak_token, context.clone(), res_context.clone(), body)
                        .await
                        .map_err(Into::into);
                    match result {
                        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
                        Err(err) => {
                            tracing::error!(?err, "Could not perform CRUD operation: delete by id.");
                            err.into_response()
                        },
                    }
                }

                /// Delete one entity using a standard filter query.
                ///
                /// Delete one entity based on the given [crate::delete::DeleteOne] body.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/delete-one",
                    request_body = DeleteOne<$resource_type>,
                    //responses(
                    //    (status = 200, description = "entity was deleted", body = DeleteResult),
                    //    (status = 500, description = "entity could not be deleted", body = String),
                    //),
                )]
                #[axum_macros::debug_handler]
                async fn delete_one(
                    Extension(keycloak_token): Extension<KeycloakToken<$role_type>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    Extension(res_context): Extension<Arc<Context>>,
                    Json(body): Json<DeleteOne<$resource_type>>,
                ) -> Response {
                    let result: Result<DeleteResult, AxumCrudError> = crudkit_rs::delete::delete_one::<$resource_type, $role_type>(keycloak_token, context.clone(), res_context.clone(), body)
                        .await
                        .map_err(Into::into);
                    match result {
                        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
                        Err(err) => {
                            tracing::error!(?err, "Could not perform CRUD operation: delete one.");
                            err.into_response()
                        },
                    }
                }

                /// Delete many entities using a standard filter query.
                ///
                /// Delete many entities based on the given [crate::delete::DeleteMany] body.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/delete-many",
                    request_body = DeleteMany,
                    //responses(
                    //    (status = 200, description = "entities were deleted", body = DeleteResult),
                    //    (status = 500, description = "entities could not be deleted", body = String),
                    //),
                )]
                #[axum_macros::debug_handler]
                async fn delete_many(
                    Extension(keycloak_token): Extension<KeycloakToken<$role_type>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<DeleteMany>,
                ) -> Response {
                    let result: Result<DeleteResult, AxumCrudError> = crudkit_rs::delete::delete_many::<$resource_type, $role_type>(keycloak_token, context.clone(), body)
                        .await
                        .map_err(Into::into);
                    match result {
                        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
                        Err(err) => {
                            tracing::error!(?err, "Could not perform CRUD operation: delete many.");
                            err.into_response()
                        },
                    }
                }

                #[derive(utoipa::OpenApi)]
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
                        schemas(crudkit_shared::DeleteResult),
                        schemas(crudkit_shared::SaveResult<Model>),
                        schemas(crudkit_shared::Saved<Model>),
                        schemas(crudkit_condition::Condition),
                        schemas(crudkit_condition::ConditionElement),
                        schemas(crudkit_condition::ConditionClause),
                        schemas(crudkit_condition::ConditionClauseValue),
                        schemas(crudkit_condition::Operator),
                        schemas(crudkit_id::SerializableId),
                        schemas(crudkit_rs::create::CreateOne<CreateModel>),
                        schemas(crudkit_rs::read::ReadCount),
                        schemas(crudkit_rs::read::ReadOne<ResourceType>),
                        schemas(crudkit_rs::read::ReadMany<ResourceType>),
                        schemas(crudkit_rs::update::UpdateOne<ResourceType>),
                        schemas(crudkit_rs::delete::DeleteById),
                        schemas(crudkit_rs::delete::DeleteOne<ResourceType>),
                        schemas(crudkit_rs::delete::DeleteMany),
                    ),
                )]
                pub struct ApiDoc; // We just use `ApiDoc` instead of `[< $name _ApiDoc >]` as we are already in a named module.
            }
        }
    };
}
