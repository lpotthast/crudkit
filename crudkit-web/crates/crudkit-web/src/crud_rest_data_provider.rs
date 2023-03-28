use crudkit_condition::{merge_conditions, Condition};
use crudkit_id::SerializableId;
use crudkit_shared::{DeleteResult, Order, SaveResult};
use indexmap::IndexMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fmt::Debug, marker::PhantomData};

use super::requests::*;
use crate::{CrudDataTrait, CrudMainTrait};

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
    pub id: SerializableId,
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

    pub async fn read_count(&self, mut read_count: ReadCount) -> Result<usize, RequestError> {
        read_count.condition = merge_conditions(self.base_condition.clone(), read_count.condition);
        let resource = T::get_resource_name();
        request_post(
            format!("{}/{resource}/crud/read-count", self.api_base_url),
            Self::get_auth()?,
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
            Self::get_auth()?,
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
            Self::get_auth()?,
            read_one,
        )
        .await
    }

    pub async fn create_one_from_create_model(
        &self,
        create_one: CreateOne<T::CreateModel>,
    ) -> Result<SaveResult<T::UpdateModel>, RequestError> {
        let resource = T::get_resource_name();
        request_post(
            format!("{}/{resource}/crud/create-one", self.api_base_url),
            Self::get_auth()?,
            create_one,
        )
        .await
    }

    #[deprecated]
    pub async fn create_one_from_update_model(
        &self,
        create_one: CreateOne<T::UpdateModel>,
    ) -> Result<SaveResult<T::UpdateModel>, RequestError> {
        let resource = T::get_resource_name();
        request_post(
            format!("{}/{resource}/crud/create-one", self.api_base_url),
            Self::get_auth()?,
            create_one,
        )
        .await
    }

    pub async fn update_one(
        &self,
        mut update_one: UpdateOne<T::UpdateModel>,
    ) -> Result<SaveResult<T::UpdateModel>, RequestError> {
        update_one.condition = merge_conditions(self.base_condition.clone(), update_one.condition);
        let resource = T::get_resource_name();
        request_post(
            format!("{}/{resource}/crud/update-one", self.api_base_url),
            Self::get_auth()?,
            update_one,
        )
        .await
    }

    pub async fn delete_by_id(
        &self,
        delete_by_id: DeleteById,
    ) -> Result<DeleteResult, RequestError> {
        let resource = T::get_resource_name();
        request_post(
            format!("{}/{resource}/crud/delete-by-id", self.api_base_url),
            Self::get_auth()?,
            delete_by_id,
        )
        .await
    }

    fn get_auth() -> Result<Option<AuthMethod>, RequestError> {
        T::AuthProvider::provide().map_err(|err| RequestError::AuthProvider(err.to_string()))
    }
}
