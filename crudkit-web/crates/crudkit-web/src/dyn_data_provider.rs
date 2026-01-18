//! Type-erased data provider for runtime polymorphic CRUD operations.

use crate::model::{AnyCreateModel, AnyUpdateModel, SerializableReadField};
use crate::request::post_json;
use crate::request_error::RequestError;
use crate::reqwest_executor::ReqwestExecutor;
use crudkit_condition::{Condition, merge_conditions};
use crudkit_core::{Deleted, DeletedMany, Order};
use indexmap::IndexMap;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::Arc;

// Re-export shared types from data_provider
pub use crate::data_provider::{DeleteById, ReadCount};

#[derive(Debug, Serialize)]
pub struct DynReadMany {
    pub limit: Option<u64>,
    pub skip: Option<u64>,
    pub order_by: Option<IndexMap<SerializableReadField, Order>>,
    pub condition: Option<Condition>,
}

#[derive(Debug, Serialize)]
pub struct DynReadOne {
    pub skip: Option<u64>,
    pub order_by: Option<IndexMap<SerializableReadField, Order>>,
    pub condition: Option<Condition>,
}

/// Not `Serialize`, as we perform custom serialization of the model on use.
#[derive(Debug)]
pub struct DynCreateOne {
    pub entity: AnyCreateModel,
}

/// Not `Serialize`, as we perform custom serialization of the model on use.
#[derive(Debug)]
pub struct DynUpdateOne {
    pub entity: AnyUpdateModel,
    pub condition: Option<Condition>,
}

#[derive(Debug, Serialize)]
pub struct DynDeleteMany {
    pub condition: Option<Condition>,
}

#[derive(Debug, Clone)]
pub struct DynCrudRestDataProvider {
    api_base_url: String,
    executor: Arc<dyn ReqwestExecutor>,
    base_condition: Option<Condition>,
    resource_name: String,
}

impl DynCrudRestDataProvider {
    pub fn new(
        api_base_url: String,
        executor: Arc<dyn ReqwestExecutor>,
        resource_name: String,
    ) -> Self {
        Self {
            api_base_url,
            executor,
            base_condition: None,
            resource_name,
        }
    }

    pub fn set_base_condition(&mut self, condition: Option<Condition>) {
        self.base_condition = condition;
    }

    pub async fn read_count(&self, mut read_count: ReadCount) -> Result<u64, RequestError> {
        read_count.condition = merge_conditions(self.base_condition.clone(), read_count.condition);
        crate::request::post(
            format!(
                "{}/{}/crud/read-count",
                self.api_base_url, self.resource_name
            ),
            self.executor.as_ref(),
            read_count,
        )
        .await
    }

    pub async fn read_many(
        &self,
        mut read_many: DynReadMany,
    ) -> Result<serde_json::Value, RequestError> {
        read_many.condition = merge_conditions(self.base_condition.clone(), read_many.condition);
        post_json(
            format!(
                "{}/{}/crud/read-many",
                self.api_base_url, self.resource_name
            ),
            self.executor.as_ref(),
            read_many,
        )
        .await
    }

    pub async fn read_one(
        &self,
        mut read_one: DynReadOne,
    ) -> Result<serde_json::Value, RequestError> {
        read_one.condition = merge_conditions(self.base_condition.clone(), read_one.condition);
        post_json(
            format!("{}/{}/crud/read-one", self.api_base_url, self.resource_name),
            self.executor.as_ref(),
            read_one,
        )
        .await
    }

    pub async fn create_one(
        &self,
        create_one: DynCreateOne,
    ) -> Result<serde_json::Value, RequestError> {
        #[derive(Debug, Serialize)]
        struct CreateOneDto {
            entity: serde_json::Value,
        }
        post_json(
            format!(
                "{}/{}/crud/create-one",
                self.api_base_url, self.resource_name
            ),
            self.executor.as_ref(),
            CreateOneDto {
                entity: serialize_any_as_json_value_omitting_type_information(
                    &create_one.entity.inner,
                ),
            },
        )
        .await
    }

    pub async fn update_one(
        &self,
        mut update_one: DynUpdateOne,
    ) -> Result<serde_json::Value, RequestError> {
        #[derive(Debug, Serialize)]
        struct UpdateOneDto {
            entity: serde_json::Value,
            condition: Option<Condition>,
        }

        update_one.condition = merge_conditions(self.base_condition.clone(), update_one.condition);
        post_json(
            format!(
                "{}/{}/crud/update-one",
                self.api_base_url, self.resource_name
            ),
            self.executor.as_ref(),
            UpdateOneDto {
                entity: serialize_any_as_json_value_omitting_type_information(
                    &update_one.entity.inner,
                ),
                condition: update_one.condition,
            },
        )
        .await
    }

    pub async fn delete_by_id(&self, delete_by_id: DeleteById) -> Result<Deleted, RequestError> {
        let json = post_json(
            format!(
                "{}/{}/crud/delete-by-id",
                self.api_base_url, self.resource_name
            ),
            self.executor.as_ref(),
            delete_by_id,
        )
        .await?;
        serde_json::from_value(json).map_err(|e| RequestError::Deserialize(e.to_string()))
    }

    pub async fn delete_many(
        &self,
        mut delete_many: DynDeleteMany,
    ) -> Result<DeletedMany, RequestError> {
        delete_many.condition =
            merge_conditions(self.base_condition.clone(), delete_many.condition);
        let json = post_json(
            format!(
                "{}/{}/crud/delete-many",
                self.api_base_url, self.resource_name
            ),
            self.executor.as_ref(),
            delete_many,
        )
        .await?;
        serde_json::from_value(json).map_err(|e| RequestError::Deserialize(e.to_string()))
    }
}

// Serialization helpers for type-erased models

#[allow(dead_code)]
pub(crate) fn serialize_any_as_json(data: &(impl erased_serde::Serialize + Debug)) -> String {
    let mut buf = Vec::new();
    let json = &mut serde_json::Serializer::new(&mut buf);
    let mut json_format = Box::new(<dyn erased_serde::Serializer>::erase(json));
    data.erased_serialize(&mut json_format).unwrap();
    drop(json_format);
    String::from_utf8_lossy(buf.as_slice()).to_string()
}

pub(crate) fn serialize_any_as_json_value_omitting_type_information(
    data: &(impl erased_serde::Serialize + Debug),
) -> serde_json::Value {
    let value: serde_json::Value =
        erased_serde::serialize(data, serde_json::value::Serializer).unwrap();
    assert!(value.is_object());
    match value {
        serde_json::Value::Object(object) => {
            assert_eq!(object.len(), 1);
            object
                .into_values()
                .next()
                .expect("real data to be present")
        }
        _ => unreachable!(),
    }
}
