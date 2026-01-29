//! Lifecycle hooks for CRUD operations.
//!
//! Hooks allow custom logic to run before and after create, read, update, and delete operations.

use crudkit_core::condition::Condition;
use crudkit_core::Order;
use indexmap::IndexMap;
use snafu::Snafu;
use std::fmt::Debug;

use crate::{auth::RequestContext, error::CrudError, resource::CrudResource};

// =============================================================================
// Hook Error Type
// =============================================================================

/// Error type for lifecycle hooks.
///
/// Hooks return `Result<R::HookData, HookError<Self::Error>>` to signal success or failure.
///
/// # Variants
///
/// - [`HookError::Forbidden`] - Permission/authorization rejection (HTTP 403)
/// - [`HookError::UnprocessableEntity`] - Business logic rejection (HTTP 422)
/// - [`HookError::Internal`] - Technical/internal error (HTTP 500)
///
/// # Example
///
/// ```ignore
/// async fn before_delete(
///     model: &Article,
///     delete_request: &DeleteRequest<Article>,
///     context: &ArticleContext,
///     request: RequestContext<KeycloakToken<Role>>,
///     data: HookData,
/// ) -> Result<HookData, HookError<MyError>> {
///     // Permission check
///     if let Some(auth) = &request.auth {
///         if model.creator_id != auth.user_id && !auth.is_admin {
///             return Err(HookError::Forbidden {
///                 reason: "Only the creator or admin can delete".into()
///             });
///         }
///     }
///
///     // Business rule check
///     if model.has_active_orders() {
///         return Err(HookError::UnprocessableEntity {
///             reason: "Cannot delete article with active orders".into()
///         });
///     }
///
///     Ok(data)
/// }
/// ```
#[derive(Debug, Clone)]
pub enum HookError<E> {
    /// Permission/authorization rejection.
    ///
    /// Use when: user lacks permission, ownership check fails, auth context invalid, etc...
    ///
    /// Mapped to HTTP status 403 Forbidden.
    Forbidden { reason: String },

    /// Business logic rejection.
    ///
    /// Use when: operation is semantically invalid due to business rules or data constraints.
    ///
    /// Mapped to HTTP status 422 Unprocessable Entity.
    UnprocessableEntity { reason: String },

    /// Internal/technical error.
    ///
    /// Use when: unexpected error occurred (database failed, external service error, etc.)
    ///
    /// Mapped to HTTP status 500 Internal Server Error.
    Internal(E),
}

impl<E: std::error::Error> From<HookError<E>> for CrudError {
    fn from(err: HookError<E>) -> Self {
        match err {
            HookError::Forbidden { reason } => CrudError::Forbidden { reason },
            HookError::UnprocessableEntity { reason } => CrudError::UnprocessableEntity { reason },
            HookError::Internal(e) => CrudError::LifecycleHookError {
                reason: e.to_string(),
            },
        }
    }
}

// =============================================================================
// Read Hook Types
// =============================================================================

/// Discriminant for read operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadOperation {
    /// Counting entities (`read_count` endpoint).
    Count,

    /// Reading a single entity (`read_one` endpoint).
    One,

    /// Reading multiple entities (`read_many` endpoint).
    Many,
}

/// Request context for read hooks.
///
/// Passed to [`CrudLifetime::before_read`] (mutable) and [`CrudLifetime::after_read`] (immutable).
///
/// Uses `R::ReadModelField` for ordering - the field enum for the read model.
///
/// # Example: Row-Level Security
///
/// ```ignore
/// async fn before_read(
///     read_request: &mut ReadRequest<Article>,
///     _context: &ArticleContext,
///     request: RequestContext<KeycloakToken<Role>>,
///     data: HookData,
/// ) -> Result<HookData, HookError<MyError>> {
///     // Add tenant filter to all reads
///     if let Some(auth) = &request.auth {
///         let tenant_filter = Condition::all().eq("tenant_id", auth.tenant_id.clone());
///         read_request.condition = match read_request.condition.take() {
///             Some(existing) => Some(existing.and(tenant_filter)),
///             None => Some(tenant_filter),
///         };
///     }
///     Ok(data)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ReadRequest<R: CrudResource> {
    /// The type of read operation being performed.
    pub operation: ReadOperation,
    /// Maximum number of entities to return (only for `ReadOperation::Many`).
    pub limit: Option<u64>,
    /// Number of entities to skip before returning results.
    pub skip: Option<u64>,
    /// Ordering specification for results.
    pub order_by: Option<IndexMap<R::ReadModelField, Order>>,
    /// Filter condition for the query.
    ///
    /// In `before_read`, this can be modified to implement row-level security
    /// by adding additional conditions (e.g., filtering by tenant_id or user ownership).
    pub condition: Option<Condition>,
}

/// Result passed to [`CrudLifetime::after_read`].
///
/// Uses `R::ReadModel` - the model returned from read operations.
///
/// The result is mutable to allow field masking or result filtering in `after_read`.
#[derive(Debug)]
pub enum ReadResult<R: CrudResource> {
    /// Result of a count operation.
    Count(u64),
    /// Result of a read_one operation.
    One(R::ReadModel),
    /// Result of a read_many operation.
    Many(Vec<R::ReadModel>),
}

// =============================================================================
// Delete Hook Types
// =============================================================================

/// Discriminant for delete operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteOperation {
    /// Deleting by explicit ID (delete_by_id endpoint)
    ById,
    /// Deleting one entity by query (delete_one endpoint)
    One,
    /// Deleting multiple entities by query (delete_many endpoint)
    Many,
}

/// Request context for delete hooks.
///
/// Passed to [`CrudLifetime::before_delete`] and [`CrudLifetime::after_delete`].
///
/// Uses `R::ModelField` for ordering - the field enum for the main entity.
#[derive(Debug, Clone)]
pub struct DeleteRequest<R: CrudResource> {
    /// The type of delete operation being performed.
    pub operation: DeleteOperation,
    /// Number of entities to skip (for delete_one).
    pub skip: Option<u64>,
    /// Ordering specification (for delete_one).
    pub order_by: Option<IndexMap<R::ModelField, Order>>,
    /// Filter condition used to select the entity.
    /// For delete_by_id, this is the condition derived from the ID.
    pub condition: Option<Condition>,
}

// =============================================================================
// Update Hook Types
// =============================================================================

/// Request context for update hooks.
///
/// Passed to [`CrudLifetime::before_update`] and [`CrudLifetime::after_update`].
#[derive(Debug, Clone)]
pub struct UpdateRequest {
    /// Filter condition used to select the entity for update (derived from ID).
    pub condition: Option<Condition>,
}

/// Lifecycle hooks for CRUD operations.
///
/// Implement this trait to add custom logic before and after create, read, update, and delete
/// operations. Hooks operate on storage-agnostic DTOs (CreateModel, UpdateModel, Model).
///
/// # Error Handling
///
/// Hooks return `Result<R::HookData, HookError<Self::Error>>`:
///
/// - Return `Ok(data)` to allow the operation to proceed.
/// - Return `Err(HookError::Forbidden { reason })` for permission denials (HTTP 403).
/// - Return `Err(HookError::UnprocessableEntity { reason })` for business rule violations (HTTP 422).
/// - Return `Err(HookError::Internal(error))` for technical errors (HTTP 500).
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
/// # Example
///
/// ```ignore
/// async fn before_delete(
///     model: &Article,
///     delete_request: &DeleteRequest<Article>,
///     context: &ArticleContext,
///     request: RequestContext<KeycloakToken<Role>>,
///     data: HookData,
/// ) -> Result<HookData, HookError<MyError>> {
///     if let Some(auth) = &request.auth {
///         if model.creator_subject != auth.subject {
///             return Err(HookError::Forbidden {
///                 reason: "Only creator can delete".into()
///             });
///         }
///     }
///     Ok(data)
/// }
/// ```
pub trait CrudLifetime<R: CrudResource> {
    /// User's custom error type, declaring internal errors.
    type Error: std::error::Error + Send + Sync + 'static;

    // =========================================================================
    // Read Hooks
    // =========================================================================

    /// Called before any read operation (read_count, read_one, read_many).
    ///
    /// # Use Cases
    /// - **Row-level security**: Modify `read_request.condition` to add tenant/user filters
    /// - **Audit logging**: Log read attempts with request context
    /// - **Access control**: Return error to deny reads based on auth context
    fn before_read(
        read_request: &mut ReadRequest<R>,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> impl Future<Output = Result<R::HookData, HookError<Self::Error>>> + Send;

    /// Called after any read operation completes successfully.
    ///
    /// # Use Cases
    /// - **Audit logging**: Log successful reads with result metadata
    /// - **Result filtering**: Remove sensitive fields from results (field masking)
    fn after_read(
        read_request: &ReadRequest<R>,
        read_result: &mut ReadResult<R>,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> impl Future<Output = Result<R::HookData, HookError<Self::Error>>> + Send;

    // =========================================================================
    // Create Hooks
    // =========================================================================

    /// Called before creating an entity.
    ///
    /// Called before any validation is run.
    ///
    /// The `create_model` can be modified to change fields before insertion.
    fn before_create(
        create_model: &mut R::CreateModel,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> impl Future<Output = Result<R::HookData, HookError<Self::Error>>> + Send;

    /// Called after an entity was created successfully.
    fn after_create(
        create_model: &R::CreateModel,
        model: &R::Model,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> impl Future<Output = Result<R::HookData, HookError<Self::Error>>> + Send;

    // =========================================================================
    // Update Hooks
    // =========================================================================

    /// Called before updating an entity.
    ///
    /// Receives the existing model and the update data. The `update_model` can
    /// be modified to change fields before the update occurs.
    fn before_update(
        existing: &R::Model,
        update_model: &mut R::UpdateModel,
        update_request: &UpdateRequest,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> impl Future<Output = Result<R::HookData, HookError<Self::Error>>> + Send;

    /// Called after an entity was updated successfully.
    fn after_update(
        update_model: &R::UpdateModel,
        model: &R::Model,
        update_request: &UpdateRequest,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> impl Future<Output = Result<R::HookData, HookError<Self::Error>>> + Send;

    // =========================================================================
    // Delete Hooks
    // =========================================================================

    /// Called before deleting an entity.
    fn before_delete(
        model: &R::Model,
        delete_request: &DeleteRequest<R>,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> impl Future<Output = Result<R::HookData, HookError<Self::Error>>> + Send;

    /// Called after an entity was deleted successfully.
    fn after_delete(
        model: &R::Model,
        delete_request: &DeleteRequest<R>,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> impl Future<Output = Result<R::HookData, HookError<Self::Error>>> + Send;
}

/// Default no-op implementation of lifecycle hooks.
///
/// All hooks simply return `Ok(data)`, allowing the operation to proceed.
#[derive(Debug)]
pub struct NoopLifetimeHooks {}

/// Error type for [`NoopLifetimeHooks`].
///
/// This type has no variants because noop hooks never fail.
#[derive(Debug, Snafu)]
pub enum NoopError {}

impl<R: CrudResource> CrudLifetime<R> for NoopLifetimeHooks {
    type Error = NoopError;

    async fn before_read(
        _read_request: &mut ReadRequest<R>,
        _context: &R::Context,
        _request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, HookError<Self::Error>> {
        Ok(data)
    }

    async fn after_read(
        _read_request: &ReadRequest<R>,
        _read_result: &mut ReadResult<R>,
        _context: &R::Context,
        _request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, HookError<Self::Error>> {
        Ok(data)
    }

    async fn before_create(
        _create_model: &mut R::CreateModel,
        _context: &R::Context,
        _request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, HookError<Self::Error>> {
        Ok(data)
    }

    async fn after_create(
        _create_model: &R::CreateModel,
        _model: &R::Model,
        _context: &R::Context,
        _request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, HookError<Self::Error>> {
        Ok(data)
    }

    async fn before_update(
        _existing: &R::Model,
        _update_model: &mut R::UpdateModel,
        _update_request: &UpdateRequest,
        _context: &R::Context,
        _request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, HookError<Self::Error>> {
        Ok(data)
    }

    async fn after_update(
        _update_model: &R::UpdateModel,
        _model: &R::Model,
        _update_request: &UpdateRequest,
        _context: &R::Context,
        _request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, HookError<Self::Error>> {
        Ok(data)
    }

    async fn before_delete(
        _model: &R::Model,
        _delete_request: &DeleteRequest<R>,
        _context: &R::Context,
        _request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, HookError<Self::Error>> {
        Ok(data)
    }

    async fn after_delete(
        _model: &R::Model,
        _delete_request: &DeleteRequest<R>,
        _context: &R::Context,
        _request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, HookError<Self::Error>> {
        Ok(data)
    }
}
