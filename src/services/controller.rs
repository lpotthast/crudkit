use super::requests::*;
use crate::{types::RequestError, CrudDataTrait};
use crud_shared_types::{ConditionElement, Order};
use indexmap::IndexMap;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize)]
pub struct ReadCount {
    pub condition: Option<Vec<ConditionElement>>,
}

pub async fn read_count<T: CrudDataTrait + DeserializeOwned + Debug>(
    read_count: ReadCount,
) -> Result<usize, RequestError> {
    let resource = T::get_resource_name();
    request_post(format!("/{resource}/crud/read-count"), read_count).await
}

#[derive(Debug, Serialize)]
pub struct ReadMany<T: CrudDataTrait> {
    pub limit: Option<u64>,
    pub skip: Option<u64>,
    pub order_by: Option<IndexMap<T::FieldType, Order>>,
    pub condition: Option<Vec<ConditionElement>>,
}

pub async fn read_many<T: CrudDataTrait + DeserializeOwned + Debug>(
    read_many: ReadMany<T>,
) -> Result<Vec<T>, RequestError> {
    let resource = T::get_resource_name();
    request_post(format!("/{resource}/crud/read-many"), read_many).await
}

#[derive(Debug, Serialize)]
pub struct ReadOne<T: CrudDataTrait> {
    pub skip: Option<u64>,
    pub order_by: Option<IndexMap<T::FieldType, Order>>,
    pub condition: Option<Vec<ConditionElement>>,
}

pub async fn read_one<T: CrudDataTrait + DeserializeOwned + Debug>(
    read_one: ReadOne<T>,
) -> Result<Option<T>, RequestError> {
    let resource = T::get_resource_name();
    request_post(format!("/{resource}/crud/read-one"), read_one).await
}

#[derive(Debug, Serialize)]
pub struct CreateOne<T: Serialize + DeserializeOwned> {
    pub entity: T,
}

pub async fn create_one<T: CrudDataTrait + Serialize + DeserializeOwned + Debug>(
    create_one: CreateOne<T>,
) -> Result<Option<T>, RequestError> {
    let resource = T::get_resource_name();
    request_post(format!("/{resource}/crud/create-one"), create_one).await
}

#[derive(Debug, Serialize)]
pub struct UpdateOne<T: Serialize + DeserializeOwned> {
    pub entity: T,
    pub condition: Option<Vec<ConditionElement>>,
}

pub async fn update_one<T: CrudDataTrait + Serialize + DeserializeOwned + Debug>(
    update_one: UpdateOne<T>,
) -> Result<Option<T>, RequestError> {
    let resource = T::get_resource_name();
    request_post(format!("/{resource}/crud/update-one"), update_one).await
}

#[derive(Debug, Serialize)]
pub struct DeleteById {
    pub id: u32,
}

pub async fn delete_by_id<T: CrudDataTrait>(
    delete_by_id: DeleteById,
) -> Result<Option<i32>, RequestError> {
    let resource = T::get_resource_name();
    request_post(format!("/{resource}/crud/delete-by-id"), delete_by_id).await
}
