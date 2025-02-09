use crate::dynamic::requests::request_post;
use crate::dynamic::{AnyField, AnyModel};
use crate::prelude::{RequestError, ReqwestExecutor};
use crudkit_condition::{merge_conditions, Condition};
use crudkit_id::SerializableId;
use crudkit_shared::{DeleteResult, Order, SaveResult};
use indexmap::IndexMap;
use serde::Serialize;
use serde_json::Value;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct ReadCount {
    pub condition: Option<Condition>,
}

#[derive(Debug, Serialize)]
pub struct ReadMany {
    pub limit: Option<u64>,
    pub skip: Option<u64>,
    #[serde(serialize_with = "serialize_order_by")]
    pub order_by: Option<IndexMap<AnyField, Order>>,
    pub condition: Option<Condition>,
}

fn serialize_order_by<S>(
    order_by: &Option<IndexMap<AnyField, Order>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;

    match order_by {
        None => serializer.serialize_none(),
        Some(map) => {
            let mut map_ser = serializer.serialize_map(Some(map.len()))?;
            for (field, order) in map {
                map_ser.serialize_entry(&field.get_name(), order)?;
            }
            map_ser.end()
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ReadOne {
    pub skip: Option<u64>,
    pub order_by: Option<IndexMap<AnyField, Order>>,
    pub condition: Option<Condition>,
}

#[derive(Debug, Serialize)]
pub struct CreateOne {
    pub entity: AnyModel,
}

#[derive(Debug)]
pub struct UpdateOne {
    pub entity: AnyModel,
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

#[derive(Serialize)]
struct RequestWrapper {
    data: Box<dyn erased_serde::Serialize + Send + Sync>,
}

impl Debug for RequestWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestWrapper").finish()
    }
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
            RequestWrapper {
                data: Box::new(read_many),
            },
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

    pub async fn create_one_from_create_model(
        &self,
        create_one: CreateOne,
    ) -> Result<SaveResult<AnyModel>, RequestError> // UpdateModel
    {
        let json = request_post(
            format!(
                "{}/{}/crud/create-one",
                self.api_base_url, self.resource_name
            ),
            self.executor.as_ref(),
            create_one,
        )
        .await?;
        let result: SaveResult<AnyModel> = serde_json::from_value(json).unwrap();
        Ok(result)
    }

    #[deprecated]
    pub async fn create_one_from_update_model(
        &self,
        create_one: CreateOne,
    ) -> Result<SaveResult<AnyModel>, RequestError> // UpdateModel
    {
        let json = request_post(
            format!(
                "{}/{}/crud/create-one",
                self.api_base_url, self.resource_name
            ),
            self.executor.as_ref(),
            create_one,
        )
        .await?;
        let result: SaveResult<AnyModel> = serde_json::from_value(json).unwrap();
        Ok(result)
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
                entity: serialize_any_as_json_value_omitting_type_information(&update_one.entity),
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

    // This line prints `["a","b"]` to stdout.
    tracing::info!(?data, "serializing");

    data.erased_serialize(&mut json_format).unwrap();
    drop(json_format);

    String::from_utf8_lossy(buf.as_slice()).to_string()
}

pub(crate) fn serialize_any_as_json_value_omitting_type_information(
    data: &(impl erased_serde::Serialize + Debug),
) -> serde_json::Value {
    let mut value: serde_json::Value =
        erased_serde::serialize(data, serde_json::value::Serializer).unwrap();
    assert!(value.is_object());
    match value {
        Value::Null => unreachable!(),
        Value::Bool(_) => unreachable!(),
        Value::Number(_) => unreachable!(),
        Value::String(_) => unreachable!(),
        Value::Array(_) => unreachable!(),
        Value::Object(object) => {
            assert_eq!(object.len(), 1);
            object
                .into_values()
                .next()
                .expect("real data to be present")
        }
    }
}
