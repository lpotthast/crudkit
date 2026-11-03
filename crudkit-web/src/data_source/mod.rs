//! Where an instance reads and changes its resource: the data source contract, the scope of its
//! operations, the pages of lists, and CrudKit's REST routes as the default data source.
//!
//! See [Data Sources](https://github.com/lpotthast/crudkit/blob/main/design/data-sources.md).

mod page;
mod rest;
mod scope;

pub use page::{CrudCursor, CrudListRequest, CrudPage, CrudPaging};
pub use rest::CrudRestDataSource;
pub use scope::CrudScope;

/// The macro with which implementations of [`CrudDataSource`] are written, as
/// `#[async_trait(?Send)]`.
pub use async_trait::async_trait;

use crate::http::RequestError;
use crate::model::{DynCreateModel, DynReadField, DynReadModel, DynUpdateModel};
use crudkit_core::id::SerializableId;
use crudkit_core::{Deleted, DeletedMany, Saved};
use std::fmt::Debug;

/// Reads and changes the entities of one resource for the instances showing it.
///
/// An instance holds its data source behind an `Arc` and passes its [`CrudScope`] to every
/// operation, so one data source serves every instance of its resource, at every place in the
/// resource's hierarchy. [`CrudRestDataSource`] talks to CrudKit's REST routes. Other data
/// sources adapt another API, e.g. a typed client, and convert between its types and the
/// resource's models.
///
/// The futures of the operations need not be `Send`, because instances load and save in the
/// browser, which runs them on one thread. Implement the trait with `#[async_trait(?Send)]`.
///
/// Every failure is a [`RequestError`]. A data source that cannot honor a part of a request, e.g.
/// a condition it cannot filter by, refuses the request with [`RequestError::InvalidRequest`]
/// instead of ignoring that part.
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
    /// The page says how the list continues: by offset, together with the number of all entities,
    /// or by the cursor of the next page.
    async fn list(
        &self,
        scope: &CrudScope,
        request: CrudListRequest,
    ) -> Result<CrudPage, RequestError>;

    /// Reads the entity `id` in `scope`. Returns `None` if no entity has that ID in `scope`.
    async fn read(
        &self,
        scope: &CrudScope,
        id: &SerializableId,
    ) -> Result<Option<DynReadModel>, RequestError>;

    /// Creates `entity` below the parents of `scope`, and returns the created entity.
    ///
    /// The instance fills the fields of `entity` that refer to the parents of `scope`, if the
    /// create model has them.
    async fn create(
        &self,
        scope: &CrudScope,
        entity: DynCreateModel,
    ) -> Result<Saved<DynUpdateModel>, RequestError>;

    /// Changes the entity `id` in `scope` to `entity`, and returns the changed entity.
    async fn update(
        &self,
        scope: &CrudScope,
        id: &SerializableId,
        entity: DynUpdateModel,
    ) -> Result<Saved<DynUpdateModel>, RequestError>;

    /// Deletes the entity `id` in `scope`.
    async fn delete(&self, scope: &CrudScope, id: &SerializableId)
    -> Result<Deleted, RequestError>;

    /// Deletes the entities `ids` in `scope`, and reports the outcome of each.
    ///
    /// By default, it deletes one entity after the other with [`Self::delete`]. A refusal with
    /// [`RequestError::Forbidden`] or [`RequestError::UnprocessableEntity`] counts as aborted, one
    /// with [`RequestError::CriticalValidationErrors`] as failed validation, and every other
    /// failure as an error. Override it when the source can delete many entities at once.
    async fn delete_many(
        &self,
        scope: &CrudScope,
        ids: &[SerializableId],
    ) -> Result<DeletedMany, RequestError> {
        let mut deleted = DeletedMany {
            deleted_count: 0,
            deleted_ids: Vec::new(),
            aborted: Vec::new(),
            validation_failed: Vec::new(),
            errors: Vec::new(),
        };
        for id in ids {
            match self.delete(scope, id).await {
                Ok(result) => {
                    deleted.deleted_count += result.entities_affected;
                    deleted.deleted_ids.push(id.clone());
                }
                Err(
                    RequestError::Forbidden(reason) | RequestError::UnprocessableEntity(reason),
                ) => {
                    deleted.aborted.push((id.clone(), reason));
                }
                Err(RequestError::CriticalValidationErrors { .. }) => {
                    deleted.validation_failed.push(id.clone());
                }
                Err(error) => deleted.errors.push((id.clone(), error.to_string())),
            }
        }
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DynReadField;
    use assertr::prelude::*;
    use crudkit_core::id::{IdValue, SerializableIdEntry};
    use crudkit_core::validation::PartialSerializableAggregateViolations;

    /// Deletes every entity except those it refuses by their ID.
    #[derive(Debug)]
    struct Refusing;

    #[async_trait(?Send)]
    impl CrudDataSource for Refusing {
        fn resource_name(&self) -> &'static str {
            "refusing"
        }

        fn can_order_by(&self, _field: &DynReadField) -> bool {
            false
        }

        async fn list(
            &self,
            _scope: &CrudScope,
            _request: CrudListRequest,
        ) -> Result<CrudPage, RequestError> {
            unreachable!("the test only deletes")
        }

        async fn read(
            &self,
            _scope: &CrudScope,
            _id: &SerializableId,
        ) -> Result<Option<DynReadModel>, RequestError> {
            unreachable!("the test only deletes")
        }

        async fn create(
            &self,
            _scope: &CrudScope,
            _entity: DynCreateModel,
        ) -> Result<Saved<DynUpdateModel>, RequestError> {
            unreachable!("the test only deletes")
        }

        async fn update(
            &self,
            _scope: &CrudScope,
            _id: &SerializableId,
            _entity: DynUpdateModel,
        ) -> Result<Saved<DynUpdateModel>, RequestError> {
            unreachable!("the test only deletes")
        }

        async fn delete(
            &self,
            _scope: &CrudScope,
            id: &SerializableId,
        ) -> Result<Deleted, RequestError> {
            match id.entries().next().map(|entry| &entry.value) {
                Some(IdValue::I64(2)) => Err(RequestError::Forbidden("captain".to_owned())),
                Some(IdValue::I64(3)) => Err(RequestError::CriticalValidationErrors {
                    message: "invalid".to_owned(),
                    violations: PartialSerializableAggregateViolations::default(),
                }),
                Some(IdValue::I64(4)) => Err(RequestError::Request("offline".to_owned())),
                _ => Ok(Deleted {
                    entities_affected: 1,
                }),
            }
        }
    }

    fn id(value: i64) -> SerializableId {
        SerializableId(vec![SerializableIdEntry {
            field_name: "id".to_owned(),
            value: IdValue::I64(value),
        }])
    }

    #[test]
    fn deleting_many_deletes_one_after_the_other_and_reports_each_outcome() {
        let ids = [id(1), id(2), id(3), id(4), id(5)];

        let deleted =
            futures::executor::block_on(Refusing.delete_many(&CrudScope::default(), &ids))
                .expect("the default never fails as a whole");

        assert_that!(deleted.deleted_count).is_equal_to(2);
        assert_that!(deleted.deleted_ids).is_equal_to(vec![id(1), id(5)]);
        assert_that!(deleted.aborted).is_equal_to(vec![(id(2), "captain".to_owned())]);
        assert_that!(deleted.validation_failed).is_equal_to(vec![id(3)]);
        assert_that!(deleted.errors)
            .is_equal_to(vec![(id(4), "Network error: offline".to_owned())]);
    }
}
