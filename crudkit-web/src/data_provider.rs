//! Typed client of the generated CRUD routes.

use crate::http::{CrudEndpoint, CrudOperation, RequestError, ReqwestExecutor};
use crate::model::Model;
use crate::resource::Resource;
use crudkit_core::condition::Condition;
use crudkit_core::request::{CreateOne, DeleteById, ReadCount, ReadMany, ReadOne, UpdateOne};
use crudkit_core::{Deleted, Saved};
use crudkit_wire_format::v1::{
    CreateOneV1, DeleteByIdV1, DeletedV1, ReadCountResponseV1, ReadCountV1, ReadManyResponseV1,
    ReadManyV1, ReadOneResponseV1, ReadOneV1, SavedV1, UpdateOneV1,
};
use std::marker::PhantomData;
use std::sync::Arc;

/// Client of the generated CRUD routes of `T`, preserving its concrete model types.
#[derive(Debug, Clone)]
pub struct CrudRestDataProvider<T: Resource> {
    endpoint: CrudEndpoint,
    phantom_data: PhantomData<T>,
}

impl<T: Resource> CrudRestDataProvider<T> {
    pub fn new(api_base_url: String, executor: Arc<dyn ReqwestExecutor>) -> Self {
        Self {
            endpoint: CrudEndpoint::new(api_base_url, executor, T::resource_name().to_owned()),
            phantom_data: PhantomData,
        }
    }

    /// Sets the condition that narrows every operation except creation.
    pub fn set_base_condition(&mut self, condition: Option<Condition>) {
        self.endpoint.set_base_condition(condition);
    }

    /// Counts the entities matching `read_count` and the base condition.
    ///
    /// # Errors
    ///
    /// Returns the [`RequestError`] describing why the request failed.
    pub async fn read_count(&self, mut read_count: ReadCount) -> Result<u64, RequestError> {
        read_count.condition = self.endpoint.scoped(read_count.condition);
        let response: ReadCountResponseV1 = self
            .endpoint
            .post(CrudOperation::ReadCount, ReadCountV1::from(read_count))
            .await?;
        Ok(response.count)
    }

    /// Reads the entities matching `read_many` and the base condition.
    ///
    /// # Errors
    ///
    /// Returns the [`RequestError`] describing why the request failed.
    pub async fn read_many(
        &self,
        mut read_many: ReadMany<<T::ReadModel as Model>::Field>,
    ) -> Result<Vec<T::ReadModel>, RequestError> {
        read_many.condition = self.endpoint.scoped(read_many.condition);
        let response: ReadManyResponseV1<T::ReadModel> = self
            .endpoint
            .post(CrudOperation::ReadMany, ReadManyV1::from(read_many))
            .await?;
        Ok(response.entities)
    }

    /// Reads one entity matching `read_one` and the base condition.
    ///
    /// # Errors
    ///
    /// Returns the [`RequestError`] describing why the request failed.
    pub async fn read_one(
        &self,
        mut read_one: ReadOne<<T::ReadModel as Model>::Field>,
    ) -> Result<Option<T::ReadModel>, RequestError> {
        read_one.condition = self.endpoint.scoped(read_one.condition);
        let response: ReadOneResponseV1<T::ReadModel> = self
            .endpoint
            .post(CrudOperation::ReadOne, ReadOneV1::from(read_one))
            .await?;
        Ok(response.entity)
    }

    /// Creates a new entity.
    ///
    /// # Errors
    ///
    /// Returns the [`RequestError`] describing why the request failed.
    pub async fn create_one(
        &self,
        create_one: CreateOne<T::CreateModel>,
    ) -> Result<Saved<T::UpdateModel>, RequestError> {
        let response: SavedV1<T::UpdateModel> = self
            .endpoint
            .post(CrudOperation::CreateOne, CreateOneV1::from(create_one))
            .await?;
        Ok(response.into())
    }

    /// Updates the entity matching `update_one` and the base condition.
    ///
    /// # Errors
    ///
    /// Returns the [`RequestError`] describing why the request failed.
    pub async fn update_one(
        &self,
        mut update_one: UpdateOne<T::UpdateModel>,
    ) -> Result<Saved<T::UpdateModel>, RequestError> {
        update_one.condition = self.endpoint.scoped(update_one.condition);
        let response: SavedV1<T::UpdateModel> = self
            .endpoint
            .post(CrudOperation::UpdateOne, UpdateOneV1::from(update_one))
            .await?;
        Ok(response.into())
    }

    /// Deletes the entity with the given ID, if it also matches the request condition and the base
    /// condition.
    ///
    /// # Errors
    ///
    /// Returns the [`RequestError`] describing why the request failed.
    pub async fn delete_by_id(
        &self,
        mut delete_by_id: DeleteById,
    ) -> Result<Deleted, RequestError> {
        delete_by_id.condition = self.endpoint.scoped(delete_by_id.condition);
        let response: DeletedV1 = self
            .endpoint
            .post(CrudOperation::DeleteById, DeleteByIdV1::from(delete_by_id))
            .await?;
        Ok(response.into())
    }
}
