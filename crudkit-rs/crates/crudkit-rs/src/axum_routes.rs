use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use crudkit_validation::PartialSerializableAggregateViolations;
use serde_json::json;
use utoipa::ToSchema;

use crate::error::CrudError;

/// Error type for Axum HTTP responses.
///
/// Maps `CrudError` variants to appropriate HTTP status codes.
#[derive(Debug, ToSchema)]
pub enum AxumCrudError {
    /// Permission denied (HTTP 403 Forbidden)
    Forbidden { reason: String },

    /// Business logic/validation rejection (HTTP 422 Unprocessable Entity)
    UnprocessableEntity { reason: String },

    /// Critical validation errors prevent the operation (HTTP 422 Unprocessable Entity)
    CriticalValidationErrors {
        reason: String,
        #[schema(value_type = Object)]
        violations: PartialSerializableAggregateViolations,
    },

    /// Entity not found (HTTP 404 Not Found)
    NotFound { reason: String },

    /// Invalid query parameters (HTTP 400 Bad Request)
    BadRequest { reason: String },

    /// Authentication required (HTTP 401 Unauthorized)
    Unauthorized { reason: String },

    /// Repository/database error (HTTP 500 Internal Server Error)
    Repository { reason: String },

    /// Lifecycle hook internal error (HTTP 500 Internal Server Error)
    LifecycleError { reason: String },

    /// Could not save validations (HTTP 500 Internal Server Error)
    SaveValidations { reason: String },

    /// Could not delete validations (HTTP 500 Internal Server Error)
    DeleteValidations { reason: String },
}

impl From<CrudError> for AxumCrudError {
    fn from(value: CrudError) -> Self {
        match value {
            /*
            User-facing errors: pass through the reason as-is.
            */
            CrudError::Forbidden { reason } => Self::Forbidden { reason },
            CrudError::UnprocessableEntity { reason } => Self::UnprocessableEntity { reason },
            CrudError::CriticalValidationErrors { violations } => Self::CriticalValidationErrors {
                reason: "Critical validation errors prevent the operation.".into(),
                violations,
            },
            CrudError::NotFound => Self::NotFound {
                reason: "Not found".into(),
            },
            CrudError::IntoCondition { .. } => Self::BadRequest {
                reason: "Invalid query parameters".into(),
            },

            /*
            Server errors: use minimal generic messages. Full details are in the original
            CrudError and should be logged via `Debug` format before conversion.
            */
            CrudError::Repository { .. } => Self::Repository {
                reason: "Repository error.".into(),
            },
            CrudError::LifecycleHookError { .. } => Self::LifecycleError {
                reason: "Lifecycle error.".into(),
            },
            CrudError::SaveValidations { .. } => Self::SaveValidations {
                reason: "Could not save validations.".into(),
            },
            CrudError::DeleteValidations { .. } => Self::DeleteValidations {
                reason: "Could not delete validations.".into(),
            },
        }
    }
}

// TODO: Use reporting mechanism (https://docs.rs/snafu/latest/snafu/attr.report.html)
impl IntoResponse for AxumCrudError {
    fn into_response(self) -> Response {
        match self {
            // ValidationFailed includes violations in the response body
            Self::CriticalValidationErrors { reason, violations } => {
                let body = Json(json!({
                    "error": reason,
                    "violations": violations,
                }));
                (StatusCode::UNPROCESSABLE_ENTITY, body).into_response()
            }

            // Client errors
            Self::Forbidden { reason } => {
                (StatusCode::FORBIDDEN, Json(json!({"error": reason}))).into_response()
            }
            Self::UnprocessableEntity { reason } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": reason})),
            )
                .into_response(),
            Self::NotFound { reason } => {
                (StatusCode::NOT_FOUND, Json(json!({"error": reason}))).into_response()
            }
            Self::BadRequest { reason } => {
                (StatusCode::BAD_REQUEST, Json(json!({"error": reason}))).into_response()
            }
            Self::Unauthorized { reason } => {
                (StatusCode::UNAUTHORIZED, Json(json!({"error": reason}))).into_response()
            }

            // Server errors
            Self::Repository { reason } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": reason})),
            )
                .into_response(),
            Self::LifecycleError { reason } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": reason})),
            )
                .into_response(),
            Self::SaveValidations { reason } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": reason})),
            )
                .into_response(),
            Self::DeleteValidations { reason } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": reason})),
            )
                .into_response(),
        }
    }
}

// TODO: On error, e REPORT must be generated, containing all error sources!

/// Macro to generate Axum CRUD routes for a resource.
///
/// # Parameters
///
/// - `$resource_type`: The type implementing `CrudResource`
/// - `$name`: Identifier for the generated module name
///
/// # Authentication/Authorization Behavior
///
/// Authorization is determined by the `CrudResource::AuthPolicy` associated type,
/// which specifies requirements per operation via [`CrudAuthPolicy`]:
///
/// - [`AuthRequirement::None`]: No authentication required for the operation (public access).
/// - [`AuthRequirement::Authenticated`]: Authentication required, state must be present
///  (automatic 401 response if `axum` `Extension` for that state is missing).
///
/// For fine-grained authorization logic (role-based access, ownership verification), implement
/// checks in your [`CrudLifetime`] hooks.
///
/// ## Built-in Policies
///
/// - [`OpenAuthPolicy`]: All operations are public.
/// - [`DefaultAuthPolicy`]: Reads public, writes require authentication.
/// - [`RestrictedAuthPolicy`]: All operations require authentication.
///
/// # Example
///
/// ```ignore
/// impl_add_crud_routes!(Article, article);
/// impl_add_crud_routes!(Comment, comment);
/// ```
///
/// You can then create a router with:
///
/// ```ignore
/// pub fn crud(root: &str) -> Router {
///     let mut router = Router::new();
///     router = super::axum_article_crud_routes::add_crud_routes(root, router);
///     router = super::axum_comment_crud_routes::add_crud_routes(root, router);
///     // ...
///     router
/// }
/// ```
#[macro_export]
macro_rules! impl_add_crud_routes {
    ($resource_type:ty, $name:ident) => {
        paste::item! {
            pub mod [< axum_ $name _crud_routes >] {
                use std::sync::Arc;
                use crudkit_rs::prelude::*;
                use crudkit_rs::auth::{AuthRequirement, CrudAuthPolicy, RequestContext};
                use crudkit_core::{DeletedMany, Deleted, Saved};
                use axum::{
                    http::StatusCode,
                    response::{IntoResponse, Response},
                    routing::post,
                    Extension, Json, Router,
                };
                use sea_orm::JsonValue;

                // We define this 'ResourceType' use statement, as `$resource_type` can not be used
                // in the `utoipa` block below...
                use $resource_type as ResourceType;
                type Auth = <$resource_type as CrudResource>::Auth;
                type Policy = <$resource_type as CrudResource>::AuthPolicy;
                type ReadViewModel = <$resource_type as CrudResource>::ReadViewModel;
                type CreateModel = <$resource_type as CrudResource>::CreateModel;
                type Model = <$resource_type as CrudResource>::Model;
                type UpdateModel = <$resource_type as CrudResource>::UpdateModel;

                /// Check the authorization requirement and build a RequestContext.
                ///
                /// Returns `Ok(RequestContext)` if the requirement is satisfied,
                /// or `Err(AxumCrudError)` with the Unauthorized variant.
                fn check_auth_requirement(
                    auth_requirement: AuthRequirement,
                    auth: Option<Extension<Auth>>,
                ) -> Result<RequestContext<Auth>, AxumCrudError> {
                    match auth_requirement {
                        AuthRequirement::None => {
                            match auth {
                                Some(Extension(a)) => Ok(RequestContext::authenticated(a)),
                                None => Ok(RequestContext::unauthenticated()),
                            }
                        }
                        AuthRequirement::Authenticated => {
                            match auth {
                                Some(Extension(a)) => Ok(RequestContext::authenticated(a)),
                                None => Err(AxumCrudError::Unauthorized {
                                    reason: "Authentication required".into(),
                                }),
                            }
                        }
                    }
                }

                /// Add all routes (create,read,update,delete) for this resource to `router`.
                ///
                /// The `root` parameter is prepended to all route paths, allowing you to use
                /// any custom api prefix like `/my-api`. Use the empty string if no prefix is
                /// required.
                pub fn add_crud_routes(
                    root: &str,
                    mut router: Router,
                ) -> Router {
                    use crudkit_rs::resource::ResourceType;
                    let resource: &'static str = <$resource_type as CrudResource>::TYPE.name();

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
                    auth: Option<Extension<Auth>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<ReadCount>,
                ) -> Response {
                    let request_context = match check_auth_requirement(Policy::read_requirement(), auth) {
                        Ok(ctx) => ctx,
                        Err(err) => return err.into_response(),
                    };
                    let result: Result<u64, AxumCrudError> = crudkit_rs::read::read_count::<$resource_type>(request_context, context.clone(), body)
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
                    auth: Option<Extension<Auth>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<ReadOne<$resource_type>>,
                ) -> Response {
                    let request_context = match check_auth_requirement(Policy::read_requirement(), auth) {
                        Ok(ctx) => ctx,
                        Err(err) => return err.into_response(),
                    };
                    let result: Result<ReadViewModel, AxumCrudError> = crudkit_rs::read::read_one::<$resource_type>(request_context, context.clone(), body)
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
                    auth: Option<Extension<Auth>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<ReadMany<$resource_type>>,
                ) -> Response {
                    let request_context = match check_auth_requirement(Policy::read_requirement(), auth) {
                        Ok(ctx) => ctx,
                        Err(err) => return err.into_response(),
                    };
                    let result: Result<Vec<ReadViewModel>, AxumCrudError> = crudkit_rs::read::read_many::<$resource_type>(request_context, context.clone(), body)
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
                    //    (status = 200, description = "entity was created", body = Saved<Model>),
                    //    (status = 422, description = "validation failed or business logic rejection", body = AxumCrudError),
                    //    (status = 500, description = "entity could not be created", body = AxumCrudError),
                    //),
                )]
                #[axum_macros::debug_handler]
                async fn create_one(
                    auth: Option<Extension<Auth>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<CreateOne<CreateModel>>,
                ) -> Response {
                    let request_context = match check_auth_requirement(Policy::create_requirement(), auth) {
                        Ok(ctx) => ctx,
                        Err(err) => return err.into_response(),
                    };
                    let result: Result<Saved<Model>, AxumCrudError> = crudkit_rs::create::create_one::<$resource_type>(request_context, context.clone(), body)
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
                    //    (status = 200, description = "entity was updated", body = Saved<Model>),
                    //    (status = 422, description = "validation failed or business logic rejection", body = AxumCrudError),
                    //    (status = 500, description = "entity could not be updated", body = String),
                    //),
                )]
                #[axum_macros::debug_handler]
                async fn update_one(
                    auth: Option<Extension<Auth>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<UpdateOne<UpdateModel>>,
                ) -> Response {
                    let request_context = match check_auth_requirement(Policy::update_requirement(), auth) {
                        Ok(ctx) => ctx,
                        Err(err) => return err.into_response(),
                    };
                    let result: Result<Saved<Model>, AxumCrudError> = crudkit_rs::update::update_one::<$resource_type>(request_context, context.clone(), body)
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
                    //    (status = 200, description = "entity was deleted", body = Deleted),
                    //    (status = 422, description = "validation failed or business logic rejection", body = AxumCrudError),
                    //    (status = 500, description = "entity could not be deleted", body = String),
                    //),
                )]
                #[axum_macros::debug_handler]
                async fn delete_by_id(
                    auth: Option<Extension<Auth>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<DeleteById>,
                ) -> Response {
                    let request_context = match check_auth_requirement(Policy::delete_requirement(), auth) {
                        Ok(ctx) => ctx,
                        Err(err) => return err.into_response(),
                    };
                    let result: Result<Deleted, AxumCrudError> = crudkit_rs::delete::delete_by_id::<$resource_type>(request_context, context.clone(), body)
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
                    //    (status = 200, description = "entity was deleted", body = Deleted),
                    //    (status = 422, description = "validation failed or business logic rejection", body = AxumCrudError),
                    //    (status = 500, description = "entity could not be deleted", body = String),
                    //),
                )]
                #[axum_macros::debug_handler]
                async fn delete_one(
                    auth: Option<Extension<Auth>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<DeleteOne<$resource_type>>,
                ) -> Response {
                    let request_context = match check_auth_requirement(Policy::delete_requirement(), auth) {
                        Ok(ctx) => ctx,
                        Err(err) => return err.into_response(),
                    };
                    let result: Result<Deleted, AxumCrudError> = crudkit_rs::delete::delete_one::<$resource_type>(request_context, context.clone(), body)
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
                /// Returns detailed results including which entities were deleted successfully
                /// and which failed for various reasons.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/delete-many",
                    request_body = DeleteMany,
                    //responses(
                    //    (status = 200, description = "Deletion results (may include partial failures)", body = DeletedMany),
                    //    (status = 422, description = "validation failed or business logic rejection", body = AxumCrudError),
                    //    (status = 500, description = "entities could not be deleted", body = String),
                    //),
                )]
                #[axum_macros::debug_handler]
                async fn delete_many(
                    auth: Option<Extension<Auth>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    Json(body): Json<DeleteMany>,
                ) -> Response {
                    let request_context = match check_auth_requirement(Policy::delete_requirement(), auth) {
                        Ok(ctx) => ctx,
                        Err(err) => return err.into_response(),
                    };
                    let result: Result<DeletedMany, AxumCrudError> = crudkit_rs::delete::delete_many::<$resource_type>(request_context, context.clone(), body)
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
                        schemas(crudkit_core::Deleted),
                        schemas(crudkit_core::DeletedMany),
                        schemas(crudkit_core::Saved<Model>),
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
