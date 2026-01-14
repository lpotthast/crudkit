use crate::dynamic::requests::request_post;
use crate::dynamic::{AnyCreateModel, AnyUpdateModel, SerializableReadField};
use crate::prelude::{RequestError, ReqwestExecutor};
use crudkit_condition::{Condition, merge_conditions};
use crudkit_core::{DeleteResult, Order};
use crudkit_id::SerializableId;
use indexmap::IndexMap;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct ReadCount {
    pub condition: Option<Condition>,
}

#[derive(Debug, Serialize)]
pub struct ReadMany {
    pub limit: Option<u64>,
    pub skip: Option<u64>,
    pub order_by: Option<IndexMap<SerializableReadField, Order>>,
    pub condition: Option<Condition>,
}

#[derive(Debug, Serialize)]
pub struct ReadOne {
    pub skip: Option<u64>,
    pub order_by: Option<IndexMap<SerializableReadField, Order>>,
    pub condition: Option<Condition>,
}

/// Not `Serialize`, as we perform custom serialization of the model on use.
#[derive(Debug)]
pub struct CreateOne {
    pub entity: AnyCreateModel,
}

/// Not `Serialize`, as we perform custom serialization of the model on use.
#[derive(Debug)]
pub struct UpdateOne {
    pub entity: AnyUpdateModel,
    pub condition: Option<Condition>,
}

#[derive(Debug, Serialize)]
pub struct DeleteById {
    pub id: SerializableId,
}

#[derive(Debug, Clone)]
pub struct CrudRestDataProvider {
    api_base_url: String,
    executor: Arc<dyn ReqwestExecutor>,
    base_condition: Option<Condition>,
    resource_name: String,
}

impl CrudRestDataProvider {
    pub fn new(
        api_base_url: String,
        executor: Arc<dyn ReqwestExecutor>,
        resource_name: String,
    ) -> Self {
        Self {
            api_base_url: api_base_url,
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
        crate::generic::requests::request_post(
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
        mut read_many: ReadMany,
    ) -> Result<serde_json::Value, RequestError> {
        // ReadModel!
        read_many.condition = merge_conditions(self.base_condition.clone(), read_many.condition);
        request_post(
            format!(
                "{}/{}/crud/read-many",
                self.api_base_url, self.resource_name
            ),
            self.executor.as_ref(),
            read_many,
        )
        .await
    }

    pub async fn read_one(&self, mut read_one: ReadOne) -> Result<serde_json::Value, RequestError> // ReadModel
    {
        read_one.condition = merge_conditions(self.base_condition.clone(), read_one.condition);
        request_post(
            format!("{}/{}/crud/read-one", self.api_base_url, self.resource_name),
            self.executor.as_ref(),
            read_one,
        )
        .await
    }

    pub async fn create_one(
        &self,
        create_one: CreateOne,
    ) -> Result<serde_json::Value, RequestError> // UpdateModel
    {
        #[derive(Debug, Serialize)]
        struct CreateOneDto {
            entity: serde_json::Value,
        }
        request_post(
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
        mut update_one: UpdateOne,
    ) -> Result<serde_json::Value, RequestError> // UpdateModel
    {
        // TODO: Should we let a callback do the concrete serialization?
        #[derive(Debug, Serialize)]
        struct UpdateOneDto {
            entity: serde_json::Value,
            condition: Option<Condition>,
        }

        update_one.condition = merge_conditions(self.base_condition.clone(), update_one.condition);
        request_post(
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

    pub async fn delete_by_id(
        &self,
        delete_by_id: DeleteById,
    ) -> Result<DeleteResult, RequestError> {
        let json = request_post(
            format!(
                "{}/{}/crud/delete-by-id",
                self.api_base_url, self.resource_name
            ),
            self.executor.as_ref(),
            delete_by_id,
        )
        .await?;
        let result: DeleteResult = serde_json::from_value(json).unwrap();
        Ok(result)
    }
}

pub(crate) fn serialize_any_as_json(data: &(impl erased_serde::Serialize + Debug)) -> String {
    // Construct some serializers.
    let mut buf = Vec::new();
    let json = &mut serde_json::Serializer::new(&mut buf);

    // The values in this map are boxed trait objects. Ordinarily this would not
    // be possible with serde::Serializer because of object safety, but type
    // erasure makes it possible with erased_serde::Serializer.
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
        serde_json::Value::Null => unreachable!(),
        serde_json::Value::Bool(_) => unreachable!(),
        serde_json::Value::Number(_) => unreachable!(),
        serde_json::Value::String(_) => unreachable!(),
        serde_json::Value::Array(_) => unreachable!(),
        serde_json::Value::Object(object) => {
            assert_eq!(object.len(), 1);
            object
                .into_values()
                .next()
                .expect("real data to be present")
        }
    }
}
