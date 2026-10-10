//! Type-erased client of the generated CRUD routes for runtime polymorphic CRUD operations.

use crate::http::{CrudEndpoint, CrudOperation, RequestError, ReqwestExecutor};
use crate::model::{DynCreateModel, DynReadField, DynReadModel, DynUpdateModel};
use crate::model_handler::ModelHandler;
use crudkit_core::condition::Condition;
use crudkit_core::request::{
    CreateOne, DeleteById, DeleteMany, ReadCount, ReadMany, ReadOne, UpdateOne,
};
use crudkit_core::{Deleted, DeletedMany, Saved};
use crudkit_wire_format::v1::{
    CreateOneV1, DeleteByIdV1, DeleteManyV1, DeletedManyV1, DeletedV1, ReadCountResponseV1,
    ReadCountV1, ReadManyResponseV1, ReadManyV1, ReadOneResponseV1, ReadOneV1, SavedV1,
    UpdateOneV1,
};
use std::sync::Arc;

/// Client of the generated CRUD routes of one resource whose model types are erased.
///
/// Its [`ModelHandler`] deserializes response entities into the resource's concrete models, so
/// operations return erased models.
#[derive(Debug, Clone)]
pub struct DynCrudRestDataProvider {
    endpoint: CrudEndpoint,
    models: ModelHandler,
}

impl DynCrudRestDataProvider {
    pub fn new(
        api_base_url: String,
        executor: Arc<dyn ReqwestExecutor>,
        resource_name: String,
        models: ModelHandler,
    ) -> Self {
        Self {
            endpoint: CrudEndpoint::new(api_base_url, executor, resource_name),
            models,
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
        mut read_many: ReadMany<DynReadField>,
    ) -> Result<Vec<DynReadModel>, RequestError> {
        read_many.condition = self.endpoint.scoped(read_many.condition);
        let response: ReadManyResponseV1<serde_json::Value> = self
            .endpoint
            .post(CrudOperation::ReadMany, ReadManyV1::from(read_many))
            .await?;
        response
            .entities
            .into_iter()
            .map(|json| self.models.deserialize_read_model(json))
            .collect::<Result<_, _>>()
            .map_err(|err| RequestError::Deserialize(err.to_string()))
    }

    /// Reads one entity matching `read_one` and the base condition, or `None` if no entity matched.
    ///
    /// # Errors
    ///
    /// Returns the [`RequestError`] describing why the request failed.
    pub async fn read_one(
        &self,
        mut read_one: ReadOne<DynReadField>,
    ) -> Result<Option<DynReadModel>, RequestError> {
        read_one.condition = self.endpoint.scoped(read_one.condition);
        let response: ReadOneResponseV1<serde_json::Value> = self
            .endpoint
            .post(CrudOperation::ReadOne, ReadOneV1::from(read_one))
            .await?;
        response
            .entity
            .map(|json| self.models.deserialize_read_model(json))
            .transpose()
            .map_err(|err| RequestError::Deserialize(err.to_string()))
    }

    /// Creates a new entity, returning the saved entity as an update model.
    ///
    /// # Errors
    ///
    /// Returns `RequestError::InvalidRequest` if the erased entity cannot be serialized, or the
    /// [`RequestError`] describing why the request failed.
    pub async fn create_one(
        &self,
        create_one: CreateOne<DynCreateModel>,
    ) -> Result<Saved<DynUpdateModel>, RequestError> {
        let entity = create_one
            .entity
            .to_untagged_json()
            .map_err(|e| RequestError::InvalidRequest(e.to_string()))?;
        let response: SavedV1<serde_json::Value> = self
            .endpoint
            .post(
                CrudOperation::CreateOne,
                CreateOneV1::from(CreateOne { entity }),
            )
            .await?;
        self.saved(response.into())
    }

    /// Updates the entity matching `update_one` and the base condition, returning the saved entity.
    ///
    /// # Errors
    ///
    /// Returns `RequestError::InvalidRequest` if the erased entity cannot be serialized, or the
    /// [`RequestError`] describing why the request failed.
    pub async fn update_one(
        &self,
        update_one: UpdateOne<DynUpdateModel>,
    ) -> Result<Saved<DynUpdateModel>, RequestError> {
        let entity = update_one
            .entity
            .to_untagged_json()
            .map_err(|e| RequestError::InvalidRequest(e.to_string()))?;
        let response: SavedV1<serde_json::Value> = self
            .endpoint
            .post(
                CrudOperation::UpdateOne,
                UpdateOneV1::from(UpdateOne {
                    entity,
                    condition: self.endpoint.scoped(update_one.condition),
                }),
            )
            .await?;
        self.saved(response.into())
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

    /// Deletes all entities matching `delete_many` and the base condition.
    ///
    /// # Errors
    ///
    /// Returns the [`RequestError`] describing why the request failed.
    pub async fn delete_many(
        &self,
        mut delete_many: DeleteMany,
    ) -> Result<DeletedMany, RequestError> {
        delete_many.condition = self.endpoint.scoped(delete_many.condition);
        let response: DeletedManyV1 = self
            .endpoint
            .post(CrudOperation::DeleteMany, DeleteManyV1::from(delete_many))
            .await?;
        Ok(response.into())
    }

    /// Deserializes the entity of a create or update response into an update model.
    fn saved(
        &self,
        saved: Saved<serde_json::Value>,
    ) -> Result<Saved<DynUpdateModel>, RequestError> {
        let entity = self
            .models
            .deserialize_update_model(saved.entity)
            .map_err(|err| RequestError::Deserialize(err.to_string()))?;
        Ok(Saved {
            entity,
            violations: saved.violations,
        })
    }
}
