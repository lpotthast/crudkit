use snafu::Snafu;
use utoipa::ToSchema;

use crate::{auth::RequestContext, resource::CrudResource};

#[derive(Debug, ToSchema)]
pub enum Abort {
    Yes { reason: String },
    No,
}

/// Lifecycle hooks for CRUD operations.
///
/// Implement this trait to add custom logic before and after create, read, update, and delete
/// operations.
/// These hooks are called within the transaction, so any failure will roll back the operation.
///
/// # Authentication in Hooks
///
/// The `request` ([`RequestContext`]) parameter contains optional authentication data.
/// The `request.auth` field is `Option<A>` because whether authentication is present
/// depends on the resource's [`AuthPolicy`](crate::auth::CrudAuthPolicy) for the current operation:
///
/// - If the policy allows public access (`AuthRequirement::None`), `request.auth` may be `None`.
/// - If the policy requires authentication, `request.auth` will be `Some(...)`.
///
/// For ownership checks or other auth-dependent logic, check if auth is present or use
/// `expect("present")`:
///
/// ```ignore
/// async fn before_delete(
///     model: &Article,
///     context: &ArticleContext,
///     request: RequestContext<KeycloakToken<Role>>,
///     data: HookData,
/// ) -> Result<(Abort, HookData), Self::Error> {
///     if let Some(auth) = &request.auth {
///         if model.creator_subject != auth.subject {
///             return Ok((Abort::Yes { reason: "Only creator can delete".into() }, data));
///         }
///     }
///     Ok((Abort::No, data))
/// }
/// ```
pub trait CrudLifetime<R: CrudResource> {
    type Error: std::error::Error; // TODO: Only when the "snafu" feature is activated. Otherwise use core or thiserror.

    // TODO: hook_data as &mut, do not force return of hook data

    async fn before_create(
        create_model: &R::CreateModel,
        active_model: &mut R::ActiveModel,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<(Abort, R::HookData), Self::Error>;

    async fn after_create(
        create_model: &R::CreateModel,
        model: &R::Model,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, Self::Error>;

    async fn before_update(
        update_model: &R::UpdateModel,
        active_model: &mut R::ActiveModel,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<(Abort, R::HookData), Self::Error>;

    async fn after_update(
        update_model: &R::UpdateModel,
        model: &R::Model,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, Self::Error>;

    async fn before_delete(
        model: &R::Model,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<(Abort, R::HookData), Self::Error>;

    async fn after_delete(
        model: &R::Model,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, Self::Error>;
}

#[derive(Debug)]
pub struct NoopLifetimeHooks {}

#[derive(Debug, Snafu)]
pub enum NoopError {}

impl<R: CrudResource> CrudLifetime<R> for NoopLifetimeHooks {
    type Error = NoopError;

    async fn before_create(
        _create_model: &<R as CrudResource>::CreateModel,
        _active_model: &mut <R as CrudResource>::ActiveModel,
        _context: &<R as CrudResource>::Context,
        _request: RequestContext<<R as CrudResource>::Auth>,
        data: <R as CrudResource>::HookData,
    ) -> Result<(Abort, <R as CrudResource>::HookData), Self::Error> {
        Ok((Abort::No, data))
    }

    async fn after_create(
        _create_model: &<R as CrudResource>::CreateModel,
        _model: &<R as CrudResource>::Model,
        _context: &<R as CrudResource>::Context,
        _request: RequestContext<<R as CrudResource>::Auth>,
        data: <R as CrudResource>::HookData,
    ) -> Result<<R as CrudResource>::HookData, Self::Error> {
        Ok(data)
    }

    async fn before_update(
        _update_model: &<R as CrudResource>::UpdateModel,
        _active_model: &mut <R as CrudResource>::ActiveModel,
        _context: &<R as CrudResource>::Context,
        _request: RequestContext<<R as CrudResource>::Auth>,
        data: <R as CrudResource>::HookData,
    ) -> Result<(Abort, <R as CrudResource>::HookData), Self::Error> {
        Ok((Abort::No, data))
    }

    async fn after_update(
        _update_model: &<R as CrudResource>::UpdateModel,
        _model: &<R as CrudResource>::Model,
        _context: &<R as CrudResource>::Context,
        _request: RequestContext<<R as CrudResource>::Auth>,
        data: <R as CrudResource>::HookData,
    ) -> Result<<R as CrudResource>::HookData, Self::Error> {
        Ok(data)
    }

    async fn before_delete(
        _model: &<R as CrudResource>::Model,
        _context: &<R as CrudResource>::Context,
        _request: RequestContext<<R as CrudResource>::Auth>,
        data: <R as CrudResource>::HookData,
    ) -> Result<(Abort, <R as CrudResource>::HookData), Self::Error> {
        Ok((Abort::No, data))
    }

    async fn after_delete(
        _model: &<R as CrudResource>::Model,
        _context: &<R as CrudResource>::Context,
        _request: RequestContext<<R as CrudResource>::Auth>,
        data: <R as CrudResource>::HookData,
    ) -> Result<<R as CrudResource>::HookData, Self::Error> {
        Ok(data)
    }
}
