//! Axum route generation and error types.
//! TODO: Extract to own crate.

use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use crudkit_core::validation::PartialSerializableAggregateViolations;
use crudkit_wire_format::v1::{ErrorResponseV1, ErrorV1, WireFormatVersionV1};

use crate::error::CrudError;

/// Error type for Axum HTTP responses.
///
/// Maps `CrudError` variants to appropriate HTTP status codes.
#[derive(Debug)]
pub enum AxumCrudError {
    /// Permission denied (HTTP 403 Forbidden).
    Forbidden { reason: String },

    /// Business logic/validation rejection (HTTP 422 Unprocessable Entity).
    UnprocessableEntity { reason: String },

    /// Critical validation errors prevent the operation (HTTP 422 Unprocessable Entity).
    CriticalValidationErrors {
        reason: String,
        violations: PartialSerializableAggregateViolations,
    },

    /// Entity not found (HTTP 404 Not Found).
    NotFound { reason: String },

    /// Invalid query parameters (HTTP 400 Bad Request).
    BadRequest { reason: String },

    /// Authentication required (HTTP 401 Unauthorized).
    Unauthorized { reason: String },

    /// Repository/database error (HTTP 500 Internal Server Error).
    Repository { reason: String },

    /// Lifecycle hook internal error (HTTP 500 Internal Server Error).
    LifecycleError { reason: String },

    /// Could not save validations (HTTP 500 Internal Server Error).
    SaveValidations { reason: String },

    /// Could not delete validations (HTTP 500 Internal Server Error).
    DeleteValidations { reason: String },
}

impl From<CrudError> for AxumCrudError {
    fn from(value: CrudError) -> Self {
        match value {
            // User-facing errors: pass through the reason as-is.
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

            // Server errors: use minimal generic messages.
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

impl AxumCrudError {
    /// Returns the HTTP status and the wire error of this failure.
    #[must_use]
    pub fn into_status_and_error(self) -> (StatusCode, ErrorV1) {
        match self {
            Self::BadRequest { reason } => (
                StatusCode::BAD_REQUEST,
                ErrorV1::BadRequest { message: reason },
            ),
            Self::Unauthorized { reason } => (
                StatusCode::UNAUTHORIZED,
                ErrorV1::Unauthorized { message: reason },
            ),
            Self::Forbidden { reason } => (
                StatusCode::FORBIDDEN,
                ErrorV1::Forbidden { message: reason },
            ),
            Self::NotFound { reason } => {
                (StatusCode::NOT_FOUND, ErrorV1::NotFound { message: reason })
            }
            Self::UnprocessableEntity { reason } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorV1::UnprocessableEntity { message: reason },
            ),
            Self::CriticalValidationErrors { reason, violations } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorV1::CriticalValidationErrors {
                    message: reason,
                    violations: violations.into(),
                },
            ),
            Self::Repository { reason }
            | Self::LifecycleError { reason }
            | Self::SaveValidations { reason }
            | Self::DeleteValidations { reason } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorV1::InternalServerError { message: reason },
            ),
        }
    }
}

impl From<JsonRejection> for AxumCrudError {
    /// A request body that is not valid JSON of the expected version 1 shape is malformed.
    fn from(rejection: JsonRejection) -> Self {
        Self::BadRequest {
            reason: rejection.body_text(),
        }
    }
}

impl IntoResponse for AxumCrudError {
    fn into_response(self) -> Response {
        let (status, error) = self.into_status_and_error();
        let body = ErrorResponseV1 {
            wire_format_version: WireFormatVersionV1,
            error,
        };
        (status, Json(body)).into_response()
    }
}

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
///   (automatic 401 response if `axum` `Extension` for that state is missing).
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

                type Auth = <$resource_type as CrudResource>::Auth;
                type Policy = <$resource_type as CrudResource>::AuthPolicy;
                type ReadModel = <$resource_type as CrudResource>::ReadModel;
                type CreateModel = <$resource_type as CrudResource>::CreateModel;
                type Model = <$resource_type as CrudResource>::Model;
                type UpdateModel = <$resource_type as CrudResource>::UpdateModel;

                type ReadModelField = <$resource_type as CrudResource>::ReadModelField;
                type ModelField = <$resource_type as CrudResource>::ModelField;
                use crudkit_rs::crudkit_wire_format::v1::{
                    CreateOneV1, DeleteByIdV1, DeleteManyV1, DeleteOneV1, DeletedManyV1, DeletedV1,
                    ErrorResponseV1, ReadCountResponseV1, ReadCountV1, ReadManyResponseV1, ReadManyV1,
                    ReadOneResponseV1, ReadOneV1, SavedV1, UpdateOneV1, WireFormatVersionV1,
                };
                use crudkit_rs::data::FieldLookup;

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
                    let version = crudkit_rs::crudkit_wire_format::WireFormatVersion::V1.path_segment();

                    let path = format!("{root}/{resource}/crud/{version}/read-count");
                    tracing::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(read_count));

                    let path = format!("{root}/{resource}/crud/{version}/read-one");
                    tracing::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(read_one));

                    let path = format!("{root}/{resource}/crud/{version}/read-many");
                    tracing::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(read_many));

                    let path = format!("{root}/{resource}/crud/{version}/create-one");
                    tracing::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(create_one));

                    let path = format!("{root}/{resource}/crud/{version}/update-one");
                    tracing::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(update_one));

                    let path = format!("{root}/{resource}/crud/{version}/delete-by-id");
                    tracing::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(delete_by_id));

                    let path = format!("{root}/{resource}/crud/{version}/delete-one");
                    tracing::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(delete_one));

                    let path = format!("{root}/{resource}/crud/{version}/delete-many");
                    tracing::debug!("Adding route: {}", path);
                    router = router.route(path.as_str(), post(delete_many));

                    router
                }

                /// Retrieve the amount of entities available.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/v1/read-count",
                    request_body = ReadCountV1,
                    responses(
                        (status = 200, body = ReadCountResponseV1),
                        (status = "default", body = ErrorResponseV1, description = "The request failed."),
                    ),
                )]
                #[axum_macros::debug_handler]
                async fn read_count(
                    auth: Option<Extension<Auth>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    body: Result<Json<ReadCountV1>, axum::extract::rejection::JsonRejection>,
                ) -> Response {
                    let request_context = match check_auth_requirement(Policy::read_requirement(), auth) {
                        Ok(ctx) => ctx,
                        Err(err) => return err.into_response(),
                    };
                    let Json(body) = match body {
                        Ok(body) => body,
                        Err(rejection) => return AxumCrudError::from(rejection).into_response(),
                    };
                    let result: Result<u64, AxumCrudError> = crudkit_rs::read::read_count::<$resource_type>(request_context, context.clone(), body.into())
                        .await
                        .map_err(Into::into);
                    match result {
                        Ok(count) => (
                            StatusCode::OK,
                            Json(ReadCountResponseV1 { wire_format_version: WireFormatVersionV1, count }),
                        )
                            .into_response(),
                        Err(err) => {
                            tracing::error!(?err, "Could not perform CRUD operation: read count.");
                            err.into_response()
                        },
                    }
                }

                /// Retrieve one entity.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/v1/read-one",
                    request_body = ReadOneV1,
                    responses(
                        (status = 200, body = ReadOneResponseV1<ReadModel>),
                        (status = "default", body = ErrorResponseV1, description = "The request failed."),
                    ),
                )]
                #[axum_macros::debug_handler]
                async fn read_one(
                    auth: Option<Extension<Auth>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    body: Result<Json<ReadOneV1>, axum::extract::rejection::JsonRejection>,
                ) -> Response {
                    let request_context = match check_auth_requirement(Policy::read_requirement(), auth) {
                        Ok(ctx) => ctx,
                        Err(err) => return err.into_response(),
                    };
                    let Json(body) = match body {
                        Ok(body) => body,
                        Err(rejection) => return AxumCrudError::from(rejection).into_response(),
                    };
                    let body = match ReadOne::try_from_v1(body, <ReadModelField as FieldLookup>::from_name) {
                        Ok(body) => body,
                        Err(err) => return AxumCrudError::BadRequest { reason: err.to_string() }.into_response(),
                    };
                    let result: Result<ReadModel, AxumCrudError> = crudkit_rs::read::read_one::<$resource_type>(request_context, context.clone(), body)
                        .await
                        .map_err(Into::into);
                    match result {
                        Ok(data) => (StatusCode::OK, Json(ReadOneResponseV1 { wire_format_version: WireFormatVersionV1, entity: Some(data) })).into_response(),
                        Err(err) => {
                            tracing::error!(?err, "Could not perform CRUD operation: read one.");
                            err.into_response()
                        },
                    }
                }

                /// Retrieve many entities.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/v1/read-many",
                    request_body = ReadManyV1,
                    responses(
                        (status = 200, body = ReadManyResponseV1<ReadModel>),
                        (status = "default", body = ErrorResponseV1, description = "The request failed."),
                    ),
                )]
                #[axum_macros::debug_handler]
                async fn read_many(
                    auth: Option<Extension<Auth>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    body: Result<Json<ReadManyV1>, axum::extract::rejection::JsonRejection>,
                ) -> Response {
                    let request_context = match check_auth_requirement(Policy::read_requirement(), auth) {
                        Ok(ctx) => ctx,
                        Err(err) => return err.into_response(),
                    };
                    let Json(body) = match body {
                        Ok(body) => body,
                        Err(rejection) => return AxumCrudError::from(rejection).into_response(),
                    };
                    let body = match ReadMany::try_from_v1(body, <ReadModelField as FieldLookup>::from_name) {
                        Ok(body) => body,
                        Err(err) => return AxumCrudError::BadRequest { reason: err.to_string() }.into_response(),
                    };
                    let result: Result<Vec<ReadModel>, AxumCrudError> = crudkit_rs::read::read_many::<$resource_type>(request_context, context.clone(), body)
                        .await
                        .map_err(Into::into);
                    match result {
                        Ok(data) => (StatusCode::OK, Json(ReadManyResponseV1 { wire_format_version: WireFormatVersionV1, entities: data })).into_response(),
                        Err(err) => {
                            tracing::error!(?err, "Could not perform CRUD operation: read many.");
                            err.into_response()
                        },
                    }
                }

                /// Create one entity.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/v1/create-one",
                    request_body = CreateOneV1<CreateModel>,
                    responses(
                        (status = 200, body = SavedV1<Model>),
                        (status = "default", body = ErrorResponseV1, description = "The request failed."),
                    ),
                )]
                #[axum_macros::debug_handler]
                async fn create_one(
                    auth: Option<Extension<Auth>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    body: Result<Json<CreateOneV1<CreateModel>>, axum::extract::rejection::JsonRejection>,
                ) -> Response {
                    let request_context = match check_auth_requirement(Policy::create_requirement(), auth) {
                        Ok(ctx) => ctx,
                        Err(err) => return err.into_response(),
                    };
                    let Json(body) = match body {
                        Ok(body) => body,
                        Err(rejection) => return AxumCrudError::from(rejection).into_response(),
                    };
                    let result: Result<Saved<Model>, AxumCrudError> = crudkit_rs::create::create_one::<$resource_type>(request_context, context.clone(), body.into())
                        .await
                        .map_err(Into::into);
                    match result {
                        Ok(data) => (StatusCode::OK, Json(SavedV1::from(data))).into_response(),
                        Err(err) => {
                            tracing::error!(?err, "Could not perform CRUD operation: create one.");
                            err.into_response()
                        },
                    }
                }

                /// Update one entity.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/v1/update-one",
                    request_body = UpdateOneV1<UpdateModel>,
                    responses(
                        (status = 200, body = SavedV1<Model>),
                        (status = "default", body = ErrorResponseV1, description = "The request failed."),
                    ),
                )]
                #[axum_macros::debug_handler]
                async fn update_one(
                    auth: Option<Extension<Auth>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    body: Result<Json<UpdateOneV1<UpdateModel>>, axum::extract::rejection::JsonRejection>,
                ) -> Response {
                    let request_context = match check_auth_requirement(Policy::update_requirement(), auth) {
                        Ok(ctx) => ctx,
                        Err(err) => return err.into_response(),
                    };
                    let Json(body) = match body {
                        Ok(body) => body,
                        Err(rejection) => return AxumCrudError::from(rejection).into_response(),
                    };
                    let result: Result<Saved<Model>, AxumCrudError> = crudkit_rs::update::update_one::<$resource_type>(request_context, context.clone(), body.into())
                        .await
                        .map_err(Into::into);
                    match result {
                        Ok(data) => (StatusCode::OK, Json(SavedV1::from(data))).into_response(),
                        Err(err) => {
                            tracing::error!(?err, "Could not perform CRUD operation: update.");
                            err.into_response()
                        },
                    }
                }

                /// Delete one entity by id.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/v1/delete-by-id",
                    request_body = DeleteByIdV1,
                    responses(
                        (status = 200, body = DeletedV1),
                        (status = "default", body = ErrorResponseV1, description = "The request failed."),
                    ),
                )]
                #[axum_macros::debug_handler]
                async fn delete_by_id(
                    auth: Option<Extension<Auth>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    body: Result<Json<DeleteByIdV1>, axum::extract::rejection::JsonRejection>,
                ) -> Response {
                    let request_context = match check_auth_requirement(Policy::delete_requirement(), auth) {
                        Ok(ctx) => ctx,
                        Err(err) => return err.into_response(),
                    };
                    let Json(body) = match body {
                        Ok(body) => body,
                        Err(rejection) => return AxumCrudError::from(rejection).into_response(),
                    };
                    let result: Result<Deleted, AxumCrudError> = crudkit_rs::delete::delete_by_id::<$resource_type>(request_context, context.clone(), body.into())
                        .await
                        .map_err(Into::into);
                    match result {
                        Ok(data) => (StatusCode::OK, Json(DeletedV1::from(data))).into_response(),
                        Err(err) => {
                            tracing::error!(?err, "Could not perform CRUD operation: delete by id.");
                            err.into_response()
                        },
                    }
                }

                /// Delete one entity using a standard filter query.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/v1/delete-one",
                    request_body = DeleteOneV1,
                    responses(
                        (status = 200, body = DeletedV1),
                        (status = "default", body = ErrorResponseV1, description = "The request failed."),
                    ),
                )]
                #[axum_macros::debug_handler]
                async fn delete_one(
                    auth: Option<Extension<Auth>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    body: Result<Json<DeleteOneV1>, axum::extract::rejection::JsonRejection>,
                ) -> Response {
                    let request_context = match check_auth_requirement(Policy::delete_requirement(), auth) {
                        Ok(ctx) => ctx,
                        Err(err) => return err.into_response(),
                    };
                    let Json(body) = match body {
                        Ok(body) => body,
                        Err(rejection) => return AxumCrudError::from(rejection).into_response(),
                    };
                    let body = match DeleteOne::try_from_v1(body, <ModelField as FieldLookup>::from_name) {
                        Ok(body) => body,
                        Err(err) => return AxumCrudError::BadRequest { reason: err.to_string() }.into_response(),
                    };
                    let result: Result<Deleted, AxumCrudError> = crudkit_rs::delete::delete_one::<$resource_type>(request_context, context.clone(), body)
                        .await
                        .map_err(Into::into);
                    match result {
                        Ok(data) => (StatusCode::OK, Json(DeletedV1::from(data))).into_response(),
                        Err(err) => {
                            tracing::error!(?err, "Could not perform CRUD operation: delete one.");
                            err.into_response()
                        },
                    }
                }

                /// Delete many entities using a standard filter query.
                #[utoipa::path(
                    post,
                    path = "/" $name "/crud/v1/delete-many",
                    request_body = DeleteManyV1,
                    responses(
                        (status = 200, body = DeletedManyV1),
                        (status = "default", body = ErrorResponseV1, description = "The request failed."),
                    ),
                )]
                #[axum_macros::debug_handler]
                async fn delete_many(
                    auth: Option<Extension<Auth>>,
                    Extension(context): Extension<Arc<CrudContext<$resource_type>>>,
                    body: Result<Json<DeleteManyV1>, axum::extract::rejection::JsonRejection>,
                ) -> Response {
                    let request_context = match check_auth_requirement(Policy::delete_requirement(), auth) {
                        Ok(ctx) => ctx,
                        Err(err) => return err.into_response(),
                    };
                    let Json(body) = match body {
                        Ok(body) => body,
                        Err(rejection) => return AxumCrudError::from(rejection).into_response(),
                    };
                    let result: Result<DeletedMany, AxumCrudError> = crudkit_rs::delete::delete_many::<$resource_type>(request_context, context.clone(), body.into())
                        .await
                        .map_err(Into::into);
                    match result {
                        Ok(data) => (StatusCode::OK, Json(DeletedManyV1::from(data))).into_response(),
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
                        schemas(crudkit_rs::crudkit_wire_format::v1::DeletedV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::DeletedManyV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::SavedV1<Model>),
                        schemas(crudkit_rs::crudkit_wire_format::v1::ErrorResponseV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::ConditionV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::ConditionElementV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::ConditionClauseV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::ConditionClauseValueV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::OperatorV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::OrderV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::SerializableIdV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::CreateOneV1<CreateModel>),
                        schemas(crudkit_rs::crudkit_wire_format::v1::ReadCountV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::ReadOneV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::ReadManyV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::ReadCountResponseV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::ReadOneResponseV1<ReadModel>),
                        schemas(crudkit_rs::crudkit_wire_format::v1::ReadManyResponseV1<ReadModel>),
                        schemas(crudkit_rs::crudkit_wire_format::v1::WireFormatVersionV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::UpdateOneV1<UpdateModel>),
                        schemas(crudkit_rs::crudkit_wire_format::v1::DeleteByIdV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::DeleteOneV1),
                        schemas(crudkit_rs::crudkit_wire_format::v1::DeleteManyV1),
                    ),
                )]
                pub struct ApiDoc;
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crudkit_core::id::{IdValue, SerializableId, SerializableIdEntry};
    use crudkit_core::validation::violation::{Violation, Violations};

    #[test]
    fn errors_map_to_their_status_and_wire_kind() {
        let reason = || "reason".to_owned();
        let cases = [
            (
                AxumCrudError::BadRequest { reason: reason() },
                400,
                "bad_request",
            ),
            (
                AxumCrudError::Unauthorized { reason: reason() },
                401,
                "unauthorized",
            ),
            (
                AxumCrudError::Forbidden { reason: reason() },
                403,
                "forbidden",
            ),
            (
                AxumCrudError::NotFound { reason: reason() },
                404,
                "not_found",
            ),
            (
                AxumCrudError::UnprocessableEntity { reason: reason() },
                422,
                "unprocessable_entity",
            ),
            (
                AxumCrudError::Repository { reason: reason() },
                500,
                "internal_server_error",
            ),
            (
                AxumCrudError::LifecycleError { reason: reason() },
                500,
                "internal_server_error",
            ),
            (
                AxumCrudError::SaveValidations { reason: reason() },
                500,
                "internal_server_error",
            ),
            (
                AxumCrudError::DeleteValidations { reason: reason() },
                500,
                "internal_server_error",
            ),
        ];
        for (error, status, kind) in cases {
            let (actual_status, error) = error.into_status_and_error();
            let json = serde_json::to_value(&error).expect("error should serialize");
            assert_eq!(actual_status.as_u16(), status);
            assert_eq!(json["kind"], kind);
            assert_eq!(json["message"], "reason");
        }
    }

    #[test]
    fn critical_validation_errors_carry_their_violations() {
        let id = SerializableId(vec![SerializableIdEntry {
            field_name: "id".to_owned(),
            value: IdValue::I64(7),
        }]);
        let error = AxumCrudError::CriticalValidationErrors {
            reason: "Critical validation errors prevent the operation.".to_owned(),
            violations: PartialSerializableAggregateViolations {
                general: None,
                create: None,
                by_entity: vec![(
                    id,
                    Violations {
                        violations: vec![Violation::critical("Too many Seekers.")],
                    },
                )],
            },
        };

        let (status, error) = error.into_status_and_error();
        let json = serde_json::to_value(&error).expect("error should serialize");
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json["kind"], "critical_validation_errors");
        assert_eq!(
            json["violations"]["by_entity"][0]["violations"][0],
            serde_json::json!({ "severity": "critical", "message": "Too many Seekers." })
        );
    }
}
