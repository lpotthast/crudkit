//! Where an instance reads and changes its resource: the data source contract, the parents of its
//! entities, the requests and outcomes of its operations, and CrudKit's REST routes as the default
//! data source.
//!
//! See [Data Sources](https://github.com/lpotthast/crudkit/blob/main/design/data-sources.md).

mod deletion;
mod entity;
mod page;
mod rest;
mod scope;

pub use deletion::{CrudDeleteManyRequest, CrudDeleteRequest, CrudDeleted, CrudDeletedMany};
pub use entity::{CrudCreateRequest, CrudReadRequest, CrudSaved, CrudUpdateRequest};
pub use page::{CrudCountRequest, CrudCursor, CrudListRequest, CrudPage, CrudPaging};
pub use rest::CrudRestDataSource;
pub use scope::CrudScope;

/// The macro with which implementations of [`CrudDataSource`] are written, as
/// `#[async_trait(?Send)]`.
pub use async_trait::async_trait;

use crate::http::RequestError;
use crate::model::{DynReadField, DynReadModel};
use std::fmt::Debug;

/// Reads and changes the entities of one resource for the instances showing it.
///
/// An instance holds its data source behind an `Arc` and passes the [`CrudScope`] of its entities
/// to every operation, so one data source serves every instance of its resource, below every
/// parent. [`CrudRestDataSource`] talks to CrudKit's REST routes. Other data sources adapt another
/// API, e.g. a typed client, and convert between its types and the resource's models.
///
/// Every operation takes a request and answers with an outcome of its own. Both are
/// `#[non_exhaustive]` and built with their constructors, so that they can gain fields without
/// breaking data sources.
///
/// The futures of the operations need not be `Send`, because instances load and save in the
/// browser, which runs them on one thread. Implement the trait with `#[async_trait(?Send)]`.
///
/// Every failure is a [`RequestError`]. A data source that cannot honor a part of a request
/// refuses the request with [`RequestError::InvalidRequest`] instead of ignoring that part.
#[async_trait(?Send)]
pub trait CrudDataSource: Debug + Send + Sync {
    /// The name of the resource, e.g. `clubs`. Instances name it as the origin of their
    /// notifications.
    fn resource_name(&self) -> &str;

    /// Returns whether [`Self::list`] can order the entities by `field`.
    ///
    /// Tables offer ordering only by these fields, and instances order only by them.
    fn can_order_by(&self, field: &DynReadField) -> bool;

    /// Reads one page of the entities in `scope`.
    ///
    /// The page says how the list continues: by offset, or by the cursor of the next page.
    async fn list(
        &self,
        scope: &CrudScope,
        request: CrudListRequest,
    ) -> Result<CrudPage, RequestError>;

    /// Counts all entities in `scope`, for a list whose pages follow each other by offset.
    ///
    /// Instances count again only when the scope changes or the list reloads, not for every
    /// page. Returns `None` if the source cannot count, as by default, e.g. because it pages by
    /// cursor. A list paged by offset then offers no page after the current one.
    async fn count(
        &self,
        scope: &CrudScope,
        request: CrudCountRequest,
    ) -> Result<Option<u64>, RequestError> {
        let _ = (scope, request);
        Ok(None)
    }

    /// Reads one entity in `scope`. Returns `None` if no entity has the ID in `scope`.
    async fn read(
        &self,
        scope: &CrudScope,
        request: CrudReadRequest,
    ) -> Result<Option<DynReadModel>, RequestError>;

    /// Creates an entity below the parents of `scope`, and returns it as stored.
    async fn create(
        &self,
        scope: &CrudScope,
        request: CrudCreateRequest,
    ) -> Result<CrudSaved, RequestError>;

    /// Changes an entity in `scope`, and returns it as stored.
    async fn update(
        &self,
        scope: &CrudScope,
        request: CrudUpdateRequest,
    ) -> Result<CrudSaved, RequestError>;

    /// Deletes an entity in `scope`.
    async fn delete(
        &self,
        scope: &CrudScope,
        request: CrudDeleteRequest,
    ) -> Result<CrudDeleted, RequestError>;

    /// Deletes many entities in `scope`, and reports the outcome for each.
    ///
    /// By default, it deletes one entity after the other with [`Self::delete`]. It goes on after
    /// a failure that concerns only one entity, such as a refusal, and skips the remaining
    /// entities after any other failure, such as a lost connection. Override it when the source
    /// can delete many entities at once.
    async fn delete_many(
        &self,
        scope: &CrudScope,
        request: CrudDeleteManyRequest,
    ) -> Result<CrudDeletedMany, RequestError> {
        Ok(deletion::delete_one_after_another(self, scope, request).await)
    }
}
