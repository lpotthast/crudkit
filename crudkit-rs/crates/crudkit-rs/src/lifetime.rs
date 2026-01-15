use crudkit_condition::Condition;
use crudkit_core::Order;
use indexmap::IndexMap;
use snafu::Snafu;
use utoipa::ToSchema;

use crate::{auth::RequestContext, resource::CrudResource};

// TODO: Document why "graceful" aborting of some actions should be supported/allowed. Why cant we handle this through standard error systems?
#[derive(Debug, ToSchema)]
pub enum Abort {
    Yes { reason: String },
    No,
}

// =============================================================================
// Read Hook Types
// =============================================================================

/// Discriminant for read operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
pub enum ReadOperation {
    /// Counting entities (read_count endpoint)
    Count,
    /// Reading a single entity (read_one endpoint)
    One,
    /// Reading multiple entities (read_many endpoint)
    Many,
}

/// Request context for read hooks.
///
/// Passed to [`CrudLifetime::before_read`] (mutable) and [`CrudLifetime::after_read`] (immutable).
///
/// Uses `R::ReadViewCrudColumn` for ordering - the column enum for the read view entity.
///
/// # Example: Row-Level Security
///
/// ```ignore
/// async fn before_read(
///     read_request: &mut ReadRequest<Article>,
///     _context: &ArticleContext,
///     request: RequestContext<KeycloakToken<Role>>,
///     data: HookData,
/// ) -> Result<(Abort, HookData), Self::Error> {
///     // Add tenant filter to all reads
///     if let Some(auth) = &request.auth {
///         let tenant_filter = Condition::all().eq("tenant_id", auth.tenant_id.clone());
///         read_request.condition = match read_request.condition.take() {
///             Some(existing) => Some(existing.and(tenant_filter)),
///             None => Some(tenant_filter),
///         };
///     }
///     Ok((Abort::No, data))
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
    pub order_by: Option<IndexMap<R::ReadViewCrudColumn, Order>>,
    /// Filter condition for the query.
    ///
    /// In `before_read`, this can be modified to implement row-level security
    /// by adding additional conditions (e.g., filtering by tenant_id or user ownership).
    pub condition: Option<Condition>,
}

/// Result passed to [`CrudLifetime::after_read`].
///
/// Uses `R::ReadViewModel` - the model returned from read operations.
///
/// The result is mutable to allow field masking or result filtering in `after_read`.
#[derive(Debug)]
pub enum ReadResult<R: CrudResource> {
    /// Result of a count operation.
    Count(u64),
    /// Result of a read_one operation.
    One(R::ReadViewModel),
    /// Result of a read_many operation.
    Many(Vec<R::ReadViewModel>),
}

// =============================================================================
// Delete Hook Types
// =============================================================================

/// Discriminant for delete operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
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
/// Uses `R::CrudColumn` for ordering - the column enum for the main entity.
#[derive(Debug, Clone)]
pub struct DeleteRequest<R: CrudResource> {
    /// The type of delete operation being performed.
    pub operation: DeleteOperation,
    /// Number of entities to skip (for delete_one).
    pub skip: Option<u64>,
    /// Ordering specification (for delete_one).
    pub order_by: Option<IndexMap<R::CrudColumn, Order>>,
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

    // =========================================================================
    // Read Hooks
    // =========================================================================

    /// Called before any read operation (read_count, read_one, read_many).
    ///
    /// # Use Cases
    /// - **Row-level security**: Modify `read_request.condition` to add tenant/user filters
    /// - **Audit logging**: Log read attempts with request context
    /// - **Access control**: Abort reads based on auth context
    ///
    /// # Mutation Pattern
    /// The `read_request` is passed by mutable reference, allowing modification of:
    /// - `condition`: Add security filters (AND with existing conditions)
    /// - `limit`: Enforce maximum page sizes
    /// - `skip`: Adjust pagination
    async fn before_read(
        read_request: &mut ReadRequest<R>,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<(Abort, R::HookData), Self::Error>;

    /// Called after any read operation completes successfully.
    ///
    /// # Use Cases
    /// - **Audit logging**: Log successful reads with result metadata
    /// - **Result filtering**: Remove sensitive fields from results (field masking)
    /// - **Metrics**: Track read patterns
    ///
    /// # Note on Result Modification
    /// The `read_result` is passed by mutable reference to allow field masking
    /// or result modification.
    async fn after_read(
        read_request: &ReadRequest<R>,
        read_result: &mut ReadResult<R>,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, Self::Error>;

    // =========================================================================
    // Create Hooks
    // =========================================================================

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

    // =========================================================================
    // Update Hooks
    // =========================================================================

    async fn before_update(
        update_model: &R::UpdateModel,
        active_model: &mut R::ActiveModel,
        update_request: &UpdateRequest,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<(Abort, R::HookData), Self::Error>;

    async fn after_update(
        update_model: &R::UpdateModel,
        model: &R::Model,
        update_request: &UpdateRequest,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, Self::Error>;

    // =========================================================================
    // Delete Hooks
    // =========================================================================

    async fn before_delete(
        model: &R::Model,
        delete_request: &DeleteRequest<R>,
        context: &R::Context,
        request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<(Abort, R::HookData), Self::Error>;

    async fn after_delete(
        model: &R::Model,
        delete_request: &DeleteRequest<R>,
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

    // =========================================================================
    // Read Hooks
    // =========================================================================

    async fn before_read(
        _read_request: &mut ReadRequest<R>,
        _context: &R::Context,
        _request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<(Abort, R::HookData), Self::Error> {
        Ok((Abort::No, data))
    }

    async fn after_read(
        _read_request: &ReadRequest<R>,
        _read_result: &mut ReadResult<R>,
        _context: &R::Context,
        _request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, Self::Error> {
        Ok(data)
    }

    // =========================================================================
    // Create Hooks
    // =========================================================================

    async fn before_create(
        _create_model: &R::CreateModel,
        _active_model: &mut R::ActiveModel,
        _context: &R::Context,
        _request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<(Abort, R::HookData), Self::Error> {
        Ok((Abort::No, data))
    }

    async fn after_create(
        _create_model: &R::CreateModel,
        _model: &R::Model,
        _context: &R::Context,
        _request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, Self::Error> {
        Ok(data)
    }

    // =========================================================================
    // Update Hooks
    // =========================================================================

    async fn before_update(
        _update_model: &R::UpdateModel,
        _active_model: &mut R::ActiveModel,
        _update_request: &UpdateRequest,
        _context: &R::Context,
        _request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<(Abort, R::HookData), Self::Error> {
        Ok((Abort::No, data))
    }

    async fn after_update(
        _update_model: &R::UpdateModel,
        _model: &R::Model,
        _update_request: &UpdateRequest,
        _context: &R::Context,
        _request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, Self::Error> {
        Ok(data)
    }

    // =========================================================================
    // Delete Hooks
    // =========================================================================

    async fn before_delete(
        _model: &R::Model,
        _delete_request: &DeleteRequest<R>,
        _context: &R::Context,
        _request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<(Abort, R::HookData), Self::Error> {
        Ok((Abort::No, data))
    }

    async fn after_delete(
        _model: &R::Model,
        _delete_request: &DeleteRequest<R>,
        _context: &R::Context,
        _request: RequestContext<R::Auth>,
        data: R::HookData,
    ) -> Result<R::HookData, Self::Error> {
        Ok(data)
    }
}
