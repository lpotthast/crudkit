use crate::request_error::RequestError;
use crate::reqwest_executor::ReqwestExecutor;
use crate::{request, Model, Resource};
use crudkit_core::condition::{merge_conditions, Condition};
use crudkit_core::id::SerializableId;
use crudkit_core::{Deleted, Order, Saved};
use indexmap::IndexMap;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use std::{fmt::Debug, marker::PhantomData};
use typed_builder::TypedBuilder;

#[derive(Debug, Serialize)]
pub struct ReadCount {
    pub condition: Option<Condition>,
}

#[derive(Debug, TypedBuilder, Serialize)]
pub struct ReadMany<T: Model> {
    pub limit: Option<u64>,
    pub skip: Option<u64>,
    pub order_by: Option<IndexMap<T::Field, Order>>,
    pub condition: Option<Condition>,
}

impl<T: Model> ReadMany<T> {
    pub fn paged(
        page: u64,
        items_per_page: u64,
    ) -> ReadManyBuilder<T, ((Option<u64>,), (Option<u64>,), (), ())> {
        ReadMany::builder()
            .limit(Some(items_per_page))
            .skip(Some(items_per_page * (page - 1)))
    }
}

#[derive(Debug, Serialize)]
pub struct ReadOne<T: Model> {
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

#[derive(Debug, Clone)]
pub struct CrudRestDataProvider<T: Resource> {
    api_base_url: String,
    executor: Arc<dyn ReqwestExecutor>,
    base_condition: Option<Condition>,
    resource_name: &'static str,
    phantom_data: PhantomData<T>,
}

impl<T: Resource> CrudRestDataProvider<T> {
    pub fn new(api_base_url: String, executor: Arc<dyn ReqwestExecutor>) -> Self {
        Self {
            api_base_url,
            executor,
            base_condition: None,
            resource_name: T::resource_name(),
            phantom_data: PhantomData {},
        }
    }

    pub fn set_base_condition(&mut self, condition: Option<Condition>) {
        self.base_condition = condition;
    }

    pub async fn read_count(&self, mut read_count: ReadCount) -> Result<u64, RequestError> {
        read_count.condition = merge_conditions(self.base_condition.clone(), read_count.condition);
        request::post(
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
        mut read_many: ReadMany<T::ReadModel>,
    ) -> Result<Vec<T::ReadModel>, RequestError>
    where
        <T as Resource>::ReadModel: 'static,
    {
        read_many.condition = merge_conditions(self.base_condition.clone(), read_many.condition);
        request::post(
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
        mut read_one: ReadOne<T::ReadModel>,
    ) -> Result<Option<T::ReadModel>, RequestError>
    where
        <T as Resource>::ReadModel: 'static,
    {
        read_one.condition = merge_conditions(self.base_condition.clone(), read_one.condition);
        request::post(
            format!("{}/{}/crud/read-one", self.api_base_url, self.resource_name),
            self.executor.as_ref(),
            read_one,
        )
        .await
    }

    pub async fn create_one(
        &self,
        create_one: CreateOne<T::CreateModel>,
    ) -> Result<Saved<T::UpdateModel>, RequestError>
    where
        <T as Resource>::CreateModel: 'static,
    {
        request::post(
            format!(
                "{}/{}/crud/create-one",
                self.api_base_url, self.resource_name
            ),
            self.executor.as_ref(),
            create_one,
        )
        .await
    }

    pub async fn update_one(
        &self,
        mut update_one: UpdateOne<T::UpdateModel>,
    ) -> Result<Saved<T::UpdateModel>, RequestError>
    where
        <T as Resource>::UpdateModel: 'static,
    {
        update_one.condition = merge_conditions(self.base_condition.clone(), update_one.condition);
        request::post(
            format!(
                "{}/{}/crud/update-one",
                self.api_base_url, self.resource_name
            ),
            self.executor.as_ref(),
            update_one,
        )
        .await
    }

    pub async fn delete_by_id(&self, delete_by_id: DeleteById) -> Result<Deleted, RequestError> {
        request::post(
            format!(
                "{}/{}/crud/delete-by-id",
                self.api_base_url, self.resource_name
            ),
            self.executor.as_ref(),
            delete_by_id,
        )
        .await
    }
}
