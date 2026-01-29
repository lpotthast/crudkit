//! SeaORM repository implementation.
//!
//! `SeaOrmRepo` implements crudkit-rs's `Repository` trait for resources that
//! also implement `SeaOrmResource`.

use std::sync::Arc;

use indexmap::IndexMap;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, PaginatorTrait};
use snafu::{Backtrace, ResultExt, Snafu};

use crudkit_rs::{
    prelude::CrudResource,
    repository::{DeleteResult, Repository, RepositoryError},
};

use crudkit_rs::crudkit_condition::Condition;
use crudkit_rs::crudkit_core::Order;

use crate::query;
use crate::traits::{
    ApplyToActiveModel, IntoActiveModelForUpdate, IntoSeaOrmActiveModel, SeaOrmResource,
};

/// SeaORM-backed repository for CRUD operations.
pub struct SeaOrmRepo {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmRepo {
    /// Create a new SeaOrmRepo with the given database connection.
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Get a reference to the database connection.
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

/// Error type for SeaORM repository operations.
#[derive(Debug, Snafu)]
pub enum SeaOrmRepoError {
    #[snafu(display("SeaOrmRepoError: Database error."))]
    Db { source: DbErr, backtrace: Backtrace },

    #[snafu(display(
        "SeaOrmRepoError: Unable to parse value for column '{column_name}' to column type: '{reason}'"
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

impl<R> Repository<R> for SeaOrmRepo
where
    R: CrudResource + SeaOrmResource,
    // CreateModel can be converted to SeaORM ActiveModel.
    R::CreateModel: IntoSeaOrmActiveModel<R::ActiveModel>,
    // UpdateModel can be applied to an ActiveModel.
    R::UpdateModel: ApplyToActiveModel<R::ActiveModel>,
    // SeaOrmModel can be converted to Model (via Into trait).
    R::SeaOrmModel: Into<R::Model>,
    // Model can be converted to ActiveModel for updates/deletes.
    R::Model: IntoActiveModelForUpdate<R::ActiveModel>,
    // ReadViewSeaOrmModel can be converted to ReadModel.
    R::ReadViewSeaOrmModel: Into<R::ReadModel>,
{
    type Error = SeaOrmRepoError;

    async fn insert(&self, create_model: R::CreateModel) -> Result<R::Model, Self::Error> {
        // Convert CreateModel to SeaORM ActiveModel.
        let active_model = create_model.into_active_model().await;

        // Build and execute the insert query.
        let sea_orm_model = query::build_insert_query::<R>(active_model)?
            .exec_with_returning(self.db.as_ref())
            .await
            .context(DbSnafu {})?;

        // Convert SeaORM model to abstract Model.
        Ok(sea_orm_model.into())
    }

    async fn count(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ModelField, Order>>,
        condition: Option<&Condition>,
    ) -> Result<u64, Self::Error> {
        query::build_select_query::<R>(limit, skip, order_by, condition)?
            .count(self.db.as_ref())
            .await
            .context(DbSnafu {})
    }

    async fn fetch_one(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ModelField, Order>>,
        condition: Option<&Condition>,
    ) -> Result<Option<R::Model>, Self::Error> {
        let result: Option<R::SeaOrmModel> =
            query::build_select_query::<R>(limit, skip, order_by, condition)?
                .one(self.db.as_ref())
                .await
                .context(DbSnafu {})?;

        Ok(result.map(Into::into))
    }

    async fn fetch_many(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ModelField, Order>>,
        condition: Option<&Condition>,
    ) -> Result<Vec<R::Model>, Self::Error> {
        let results: Vec<R::SeaOrmModel> =
            query::build_select_query::<R>(limit, skip, order_by, condition)?
                .all(self.db.as_ref())
                .await
                .context(DbSnafu {})?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn read_one(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ReadModelField, Order>>,
        condition: Option<&Condition>,
    ) -> Result<Option<R::ReadModel>, Self::Error> {
        let result: Option<R::ReadViewSeaOrmModel> =
            query::build_read_view_query::<R>(limit, skip, order_by, condition)?
                .one(self.db.as_ref())
                .await
                .context(DbSnafu {})?;

        Ok(result.map(Into::into))
    }

    async fn read_many(
        &self,
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<R::ReadModelField, Order>>,
        condition: Option<&Condition>,
    ) -> Result<Vec<R::ReadModel>, Self::Error> {
        let results: Vec<R::ReadViewSeaOrmModel> =
            query::build_read_view_query::<R>(limit, skip, order_by, condition)?
                .all(self.db.as_ref())
                .await
                .context(DbSnafu {})?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn update(
        &self,
        existing: R::Model,
        update_model: R::UpdateModel,
    ) -> Result<R::Model, Self::Error> {
        // Convert existing model to ActiveModel.
        let mut active_model: R::ActiveModel = existing.into_active_model_for_update();

        // Apply updates from UpdateModel.
        update_model.apply_to(&mut active_model);

        // Execute the update.
        let updated: R::SeaOrmModel = active_model
            .update(self.db.as_ref())
            .await
            .context(DbSnafu {})?;

        Ok(updated.into())
    }

    async fn delete(&self, model: R::Model) -> Result<DeleteResult, Self::Error> {
        // Convert to ActiveModel for deletion.
        let active_model: R::ActiveModel = model.into_active_model_for_update();

        // Execute deletion.
        let delete_result = active_model
            .delete(self.db.as_ref())
            .await
            .context(DbSnafu {})?;

        Ok(DeleteResult {
            entities_affected: delete_result.rows_affected,
        })
    }
}
