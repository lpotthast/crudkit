use crate::resource::CrudResource;
use async_trait::async_trait;
use crudkit_condition::Condition;
use crudkit_core::Order;
use crudkit_id::Id;
use crudkit_validation::{ViolationsByEntity, ViolationsByResource};
use indexmap::IndexMap;
use snafu::Snafu;
use std::future::Future;

/// Marker error trait for error types used in repository implementations.
pub trait RepositoryError: std::fmt::Debug + Send + Sync {} // TODO: Use Display trait instead?

/// Support error-stack `Report`s being used as repository error types.
impl<T: RepositoryError> RepositoryError for error_stack::Report<T> {}

pub trait Repository<R: CrudResource> {
    type Error: RepositoryError + Send + Sync + 'static;

    fn insert(
        &self,
        model: R::ActiveModel,
    ) -> impl Future<Output = Result<R::Model, Self::Error>> + Send;

    fn count(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::CrudColumn, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<u64, Self::Error>> + Send;

    fn fetch_one(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::CrudColumn, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<Option<R::Model>, Self::Error>> + Send;

    fn fetch_many(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::CrudColumn, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<Vec<R::Model>, Self::Error>> + Send;

    fn read_one(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ReadViewCrudColumn, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<Option<R::ReadViewModel>, Self::Error>> + Send;

    fn read_many(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ReadViewCrudColumn, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<Vec<R::ReadViewModel>, Self::Error>> + Send;

    /// Update the entity declared through `model`.
    ///
    /// # Returns
    ///
    /// Returns the updated entity after it was fully handled. The value returned should equal what
    /// would otherwise be obtainable by loading this entity immediately after this function
    /// returned.
    fn update(
        &self,
        model: R::ActiveModel,
    ) -> impl Future<Output = Result<R::Model, Self::Error>> + Send;

    /// Delete the entity declared through `model`.
    ///
    /// # Returns
    ///
    /// The number of entities affected through `DeleteModel`. This should always be `1` if the
    /// entity still existed and could be deleted, or `0` if that entity does not exist anymore.
    // TODO: Would we expect a no longer existing entity to be handled through an error instead?
    fn delete(
        &self,
        model: R::Model,
    ) -> impl Future<Output = Result<DeleteResult, Self::Error>> + Send;
}

#[derive(Debug)]
pub struct DeleteResult {
    pub entities_affected: u64,
}

/// Trait for persisting validation results to a storage backend.
// TODO: Should these functions even take `I`?
#[async_trait]
pub trait ValidationResultRepository {
    type Error: RepositoryError + 'static;

    /// Delete all violations for a specific entity of the given resource type.
    async fn delete_all_of_entity<I: Id>(
        &self,
        resource_name: &str,
        entity_id: &I,
    ) -> Result<(), Self::Error>;

    /// Delete all violations for the given resource.
    async fn delete_all_of_resource(&self, resource_name: &str) -> Result<(), Self::Error>;

    /// Save all violations for the given resource type.
    async fn save_all<I: Id>(
        &self,
        resource_name: &str,
        validation_results: ViolationsByEntity<I>,
    ) -> Result<(), Self::Error>;

    /// List all violations of the given resource type.
    async fn list_all_of_resource<I: Id>(
        &self,
        resource_name: &str,
    ) -> Result<ViolationsByEntity<I>, Self::Error>;

    /// List all violations of all resource types.
    async fn list_all(&self) -> Result<ViolationsByResource, Self::Error>;
}

pub struct NoopValidationResultRepository;

#[derive(Debug, Snafu)]
pub enum NoopError {}

impl RepositoryError for NoopError {}

#[async_trait]
impl ValidationResultRepository for NoopValidationResultRepository {
    type Error = NoopError;

    async fn delete_all_of_entity<I: Id>(
        &self,
        _resource_name: &str,
        _entity_id: &I,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn delete_all_of_resource(&self, _resource_name: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn save_all<I: Id>(
        &self,
        _resource_name: &str,
        _validation_results: ViolationsByEntity<I>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn list_all_of_resource<I: Id>(
        &self,
        _resource_name: &str,
    ) -> Result<ViolationsByEntity<I>, Self::Error> {
        Ok(ViolationsByEntity::new())
    }

    async fn list_all(&self) -> Result<ViolationsByResource, Self::Error> {
        Ok(ViolationsByResource::new())
    }
}
