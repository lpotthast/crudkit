//! Repository trait for storage-agnostic data access.
//!
//! The repository pattern abstracts away the underlying storage mechanism,
//! allowing the CRUD operations to work with any backend that implements
//! this trait (e.g., SeaORM, in-memory, mock implementations).

// TODO: Rename this module to `persistence` or `storage`.

use crate::resource::CrudResource;
use async_trait::async_trait;
use crudkit_core::condition::Condition;
use crudkit_core::Order;
use crudkit_core::id::Id;
use crudkit_core::validation::{ViolationsByEntity, ViolationsByResource};
use indexmap::IndexMap;
use snafu::Snafu;
use std::future::Future;

/// Marker error trait for error types used in repository implementations.
pub trait RepositoryError: std::fmt::Debug + Send + Sync {}

/// Support error-stack `Report`s being used as repository error types.
impl<T: RepositoryError> RepositoryError for error_stack::Report<T> {}

/// Storage-agnostic repository trait for CRUD operations.
///
/// The repository takes DTOs (CreateModel, UpdateModel) directly and handles
/// the conversion to storage-specific types internally. This keeps the CRUD
/// operations decoupled from any specific storage backend.
///
/// # Type Parameters
///
/// - `R`: The resource type implementing [`CrudResource`]
pub trait Repository<R: CrudResource> {
    /// The error type returned by repository operations.
    type Error: RepositoryError + Send + Sync + 'static;

    /// Insert a new entity from a create model.
    ///
    /// The repository is responsible for converting the CreateModel to the
    /// appropriate storage format and returning the persisted Model.
    fn insert(
        &self,
        create_model: R::CreateModel,
    ) -> impl Future<Output = Result<R::Model, Self::Error>> + Send;

    /// Count entities matching the given criteria.
    fn count(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ModelField, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<u64, Self::Error>> + Send;

    /// Fetch a single entity matching the given criteria.
    ///
    /// Returns the Model (actual persisted entity) for use in lifecycle hooks.
    fn fetch_one(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ModelField, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<Option<R::Model>, Self::Error>> + Send;

    /// Fetch multiple entities matching the given criteria.
    fn fetch_many(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ModelField, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<Vec<R::Model>, Self::Error>> + Send;

    /// Read a single entity through the read view.
    ///
    /// Uses the ReadModel (which may be backed by a SQL view) for reading.
    fn read_one(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ReadModelField, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<Option<R::ReadModel>, Self::Error>> + Send;

    /// Read multiple entities through the read view.
    fn read_many(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ReadModelField, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<Vec<R::ReadModel>, Self::Error>> + Send;

    /// Update an entity using the existing model and update data.
    ///
    /// The repository:
    /// 1. Applies the UpdateModel changes to the existing Model
    /// 2. Persists the changes
    /// 3. Returns the updated Model
    fn update(
        &self,
        existing: R::Model,
        update_model: R::UpdateModel,
    ) -> impl Future<Output = Result<R::Model, Self::Error>> + Send;

    /// Delete an entity.
    ///
    /// # Returns
    ///
    /// The number of entities affected. Should be 1 if the entity existed
    /// and was deleted, or 0 if it no longer exists.
    fn delete(
        &self,
        model: R::Model,
    ) -> impl Future<Output = Result<DeleteResult, Self::Error>> + Send;
}

/// Result of a delete operation.
#[derive(Debug)]
pub struct DeleteResult {
    /// Number of entities that were deleted.
    pub entities_affected: u64,
}

/// Trait for persisting validation results to a storage backend.
#[async_trait]
pub trait ValidationResultRepository {
    /// The error type for validation result operations.
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

/// A no-op validation result repository that discards all validation results.
///
/// Useful for testing or when validation persistence is not needed.
pub struct NoopValidationResultRepository;

/// Error type for [`NoopValidationResultRepository`].
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
