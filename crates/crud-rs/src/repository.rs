use async_trait::async_trait;
use indexmap::IndexMap;

use crate::resource::CrudResource;

#[async_trait]
pub trait Repository<R: CrudResource> {
    type Error: RepositoryError + Send + Sync + 'static;

    async fn insert(&self, model: R::ActiveModel) -> Result<R::Model, Self::Error>;

    async fn count(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::CrudColumn, crud_shared_types::Order>>,
        condition: Option<&crud_shared_types::condition::Condition>,
    ) -> Result<u64, Self::Error>;

    async fn fetch_one(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::CrudColumn, crud_shared_types::Order>>,
        condition: Option<&crud_shared_types::condition::Condition>,
    ) -> Result<Option<R::Model>, Self::Error>;

    async fn fetch_many(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::CrudColumn, crud_shared_types::Order>>,
        condition: Option<&crud_shared_types::condition::Condition>,
    ) -> Result<Vec<R::Model>, Self::Error>;

    async fn read_one(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ReadViewCrudColumn, crud_shared_types::Order>>,
        condition: Option<&crud_shared_types::condition::Condition>,
    ) -> Result<Option<R::ReadViewModel>, Self::Error>;

    async fn read_many(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ReadViewCrudColumn, crud_shared_types::Order>>,
        condition: Option<&crud_shared_types::condition::Condition>,
    ) -> Result<Vec<R::ReadViewModel>, Self::Error>;

    async fn update(&self, model: R::ActiveModel) -> Result<R::Model, Self::Error>;

    async fn delete(&self, model: R::Model) -> Result<DeleteResult, Self::Error>;
}

#[derive(Debug)]
pub struct DeleteResult {
    pub entities_affected: u64,
}

/// Marker error trait for error types used in repository implementations.
pub trait RepositoryError: std::fmt::Debug + Send + Sync {} // TODO: Use Display trait instead?
