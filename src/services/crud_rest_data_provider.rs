use super::requests::*;
use crate::{types::RequestError, CrudDataTrait, CrudMainTrait};
use crud_shared_types::{Condition, Order, SaveResult};
use indexmap::IndexMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fmt::Debug, marker::PhantomData};

#[derive(Debug, Serialize)]
pub struct ReadCount {
    pub condition: Option<Condition>,
}

#[derive(Debug, Serialize)]
pub struct ReadMany<T: CrudDataTrait> {
    pub limit: Option<u64>,
    pub skip: Option<u64>,
    pub order_by: Option<IndexMap<T::Field, Order>>,
    pub condition: Option<Condition>,
}

#[derive(Debug, Serialize)]
pub struct ReadOne<T: CrudDataTrait> {
    pub skip: Option<u64>,
    pub order_by: Option<IndexMap<T::Field, Order>>,
    pub condition: Option<Condition>,
}

#[derive(Debug, Serialize)]
pub struct CreateOne<T: Serialize + DeserializeOwned> {
    pub entity: T,
}

#[derive(Debug, Serialize)]
pub struct UpdateOne<T: Serialize + DeserializeOwned> {
    pub entity: T,
    pub condition: Option<Condition>,
}

#[derive(Debug, Serialize)]
pub struct DeleteById {
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrudRestDataProvider<T: CrudMainTrait> {
    api_base_url: String,
    base_condition: Option<Condition>,
    phantom_data: PhantomData<T>,
}

impl<T: CrudMainTrait> CrudRestDataProvider<T> {
    pub fn new(api_base_url: String) -> Self {
        Self {
            api_base_url: api_base_url,
            base_condition: None,
            phantom_data: PhantomData {},
        }
    }

    pub fn set_base_condition(&mut self, condition: Option<Condition>) {
        self.base_condition = condition;
    }

    pub async fn read_count(&self, read_count: ReadCount) -> Result<usize, RequestError> {
        let resource = T::get_resource_name();
        request_post(
            format!("{}/{resource}/crud/read-count", self.api_base_url),
            read_count,
        )
        .await
    }

    pub async fn read_many(
        &self,
        mut read_many: ReadMany<T::ReadModel>,
    ) -> Result<Vec<T::ReadModel>, RequestError> {
        read_many.condition = merge_conditions(self.base_condition.clone(), read_many.condition);
        let resource = T::get_resource_name();
        request_post(
            format!("{}/{resource}/crud/read-many", self.api_base_url),
            read_many,
        )
        .await
    }

    pub async fn read_one(
        &self,
        mut read_one: ReadOne<T::ReadModel>,
    ) -> Result<Option<T::ReadModel>, RequestError> {
        read_one.condition = merge_conditions(self.base_condition.clone(), read_one.condition);
        let resource = T::get_resource_name();
        request_post(
            format!("{}/{resource}/crud/read-one", self.api_base_url),
            read_one,
        )
        .await
    }

    pub async fn create_one(
        &self,
        create_one: CreateOne<T::UpdateModel>,
    ) -> Result<Option<T::UpdateModel>, RequestError> {
        let resource = T::get_resource_name();
        request_post(
            format!("{}/{resource}/crud/create-one", self.api_base_url),
            create_one,
        )
        .await
    }

    pub async fn update_one(
        &self,
        mut update_one: UpdateOne<T::UpdateModel>,
    ) -> Result<Option<SaveResult<T::UpdateModel>>, RequestError> {
        update_one.condition = merge_conditions(self.base_condition.clone(), update_one.condition);
        let resource = T::get_resource_name();
        request_post(
            format!("{}/{resource}/crud/update-one", self.api_base_url),
            update_one,
        )
        .await
    }

    pub async fn delete_by_id(
        &self,
        delete_by_id: DeleteById,
    ) -> Result<Option<i32>, RequestError> {
        let resource = T::get_resource_name();
        request_post(
            format!("{}/{resource}/crud/delete-by-id", self.api_base_url),
            delete_by_id,
        )
        .await
    }
}

pub fn merge_conditions(a: Option<Condition>, b: Option<Condition>) -> Option<Condition> {
    if a.is_none() && b.is_none() {
        None
    } else if a.is_some() && b.is_none() {
        a
    } else if a.is_none() && b.is_some() {
        b
    } else {
        let mut combined = Condition::all();
        combined.push_condition(a.unwrap());
        combined.push_condition(b.unwrap());
        if combined.is_empty() {
            None
        } else {
            Some(combined)
        }
    }
}
