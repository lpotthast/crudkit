use indexmap::IndexMap;
use std::future::Future;

use crudkit_condition::Condition;
use crudkit_core::Order;

use crate::resource::CrudResource;

pub trait Repository<R: CrudResource> {
    type Error: RepositoryError + Send + Sync + 'static;

    fn insert(
        &self,
        model: R::ActiveModel,
    ) -> impl Future<Output = Result<R::Model, Self::Error>> + Send;

    fn count(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::CrudColumn, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<u64, Self::Error>> + Send;

    fn fetch_one(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::CrudColumn, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<Option<R::Model>, Self::Error>> + Send;

    fn fetch_many(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::CrudColumn, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<Vec<R::Model>, Self::Error>> + Send;

    fn read_one(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ReadViewCrudColumn, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<Option<R::ReadViewModel>, Self::Error>> + Send;

    fn read_many(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ReadViewCrudColumn, Order>>,
        condition: Option<&Condition>,
    ) -> impl Future<Output = Result<Vec<R::ReadViewModel>, Self::Error>> + Send;

    fn update(
        &self,
        model: R::ActiveModel,
    ) -> impl Future<Output = Result<R::Model, Self::Error>> + Send;

    fn delete(
        &self,
        model: R::Model,
    ) -> impl Future<Output = Result<DeleteResult, Self::Error>> + Send;
}

#[derive(Debug)]
pub struct DeleteResult {
    pub entities_affected: u64,
}

/// Marker error trait for error types used in repository implementations.
pub trait RepositoryError: std::fmt::Debug + Send + Sync {} // TODO: Use Display trait instead?

/// Support error-stack `Report`s being used as repository error types.
impl<T: RepositoryError> RepositoryError for error_stack::Report<T> {}
