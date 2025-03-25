use std::sync::Arc;

use indexmap::IndexMap;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, ModelTrait, PaginatorTrait};
use snafu::{Backtrace, ResultExt, Snafu};

use crudkit_rs::{
    repository::{DeleteResult, Repository, RepositoryError},
    resource::CrudResource,
};

use crudkit_condition::Condition;
use crudkit_shared::Order;

use crate::query;

pub struct SeaOrmRepo {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmRepo {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[derive(Debug, Snafu)]
pub enum SeaOrmRepoError {
    #[snafu(display("SeaOrmRepoError: Database error."))]
    Db { source: DbErr, backtrace: Backtrace },

    #[snafu(display(
        "SeaOrmRepoError: Unable to parse value for column'{column_name}' to column type: '{reason}'"
    ))]
    UnableToParseValueAsColType {
        column_name: String,
        reason: String,
        backtrace: Backtrace,
    },

    #[snafu(display("SeaOrmRepoError: Column '{column_name}' not found."))]
    UnknownColumnSpecified {
        column_name: String,
        backtrace: Backtrace,
    },
}

impl RepositoryError for SeaOrmRepoError {}

impl<R: CrudResource> Repository<R> for SeaOrmRepo {
    type Error = SeaOrmRepoError;

    async fn insert(&self, model: R::ActiveModel) -> Result<R::Model, Self::Error> {
        query::build_insert_query::<R>(model)?
            .exec_with_returning(self.db.as_ref())
            .await
            .context(DbSnafu {})
    }

    async fn count(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::CrudColumn, Order>>,
        condition: Option<&crudkit_condition::Condition>,
    ) -> Result<u64, Self::Error> {
        query::build_select_query::<R::Entity, R::Model, R::ActiveModel, R::Column, R::CrudColumn>(
            limit, skip, order_by, condition,
        )?
        .count(self.db.as_ref())
        .await
        .context(DbSnafu {})
    }

    async fn fetch_one(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::CrudColumn, Order>>,
        condition: Option<&Condition>,
    ) -> Result<Option<R::Model>, Self::Error> {
        query::build_select_query::<R::Entity, R::Model, R::ActiveModel, R::Column, R::CrudColumn>(
            limit, skip, order_by, condition,
        )?
        .one(self.db.as_ref())
        .await
        .context(DbSnafu {})
    }

    async fn fetch_many(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::CrudColumn, Order>>,
        condition: Option<&Condition>,
    ) -> Result<Vec<R::Model>, Self::Error> {
        query::build_select_query::<R::Entity, R::Model, R::ActiveModel, R::Column, R::CrudColumn>(
            limit, skip, order_by, condition,
        )?
        .all(self.db.as_ref())
        .await
        .context(DbSnafu {})
    }

    async fn read_one(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ReadViewCrudColumn, Order>>,
        condition: Option<&Condition>,
    ) -> Result<Option<R::ReadViewModel>, Self::Error> {
        query::build_select_query::<
            R::ReadViewEntity,
            R::ReadViewModel,
            R::ReadViewActiveModel,
            R::ReadViewColumn,
            R::ReadViewCrudColumn,
        >(limit, skip, order_by, condition)?
        .one(self.db.as_ref())
        .await
        .context(DbSnafu {})
    }

    async fn read_many(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ReadViewCrudColumn, Order>>,
        condition: Option<&Condition>,
    ) -> Result<Vec<R::ReadViewModel>, Self::Error> {
        query::build_select_query::<
            R::ReadViewEntity,
            R::ReadViewModel,
            R::ReadViewActiveModel,
            R::ReadViewColumn,
            R::ReadViewCrudColumn,
        >(limit, skip, order_by, condition)?
        .all(self.db.as_ref())
        .await
        .context(DbSnafu {})
    }

    async fn update(&self, model: R::ActiveModel) -> Result<R::Model, Self::Error> {
        model.update(self.db.as_ref()).await.context(DbSnafu {})
    }

    async fn delete(&self, model: R::Model) -> Result<DeleteResult, Self::Error> {
        R::Model::delete(model, self.db.as_ref())
            .await
            .context(DbSnafu {})
            .map(|delete_result| DeleteResult {
                entities_affected: delete_result.rows_affected,
            })
    }
}
