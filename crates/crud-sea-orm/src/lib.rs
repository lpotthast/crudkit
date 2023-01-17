use std::sync::Arc;

use async_trait::async_trait;
use indexmap::IndexMap;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, ModelTrait, PaginatorTrait};
use snafu::{Backtrace, ResultExt, Snafu};
use utoipa::ToSchema;

use crud_rs::{
    repository::{DeleteResult, Repository, RepositoryError},
    resource::CrudResource,
};

pub struct SeaOrmRepo {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmRepo {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[derive(Debug, Snafu, ToSchema)]
pub enum SeaOrmRepoError {
    #[snafu(display(
        "SeaOrmRepoError: Database error.\n\nCaused by:\n{source}\n\nBacktrace:\n{backtrace}"
    ))]
    Db { source: DbErr, backtrace: Backtrace },

    #[snafu(display(
        "SeaOrmRepoError: Unable to parse value for column'{column_name}' to column type: '{reason}'\n\nBacktrace:\n{backtrace}"
    ))]
    UnableToParseValueAsColType {
        column_name: String,
        reason: String,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "SeaOrmRepoError: Column '{column_name}' not found.\n\nBacktrace:\n{backtrace}"
    ))]
    UnknownColumnSpecified {
        column_name: String,
        backtrace: Backtrace,
    },
}

impl RepositoryError for SeaOrmRepoError {}

#[async_trait]
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
        order_by: Option<IndexMap<R::CrudColumn, crud_shared_types::Order>>,
        condition: Option<&crud_shared_types::condition::Condition>,
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
        order_by: Option<IndexMap<R::CrudColumn, crud_shared_types::Order>>,
        condition: Option<&crud_shared_types::condition::Condition>,
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
        order_by: Option<IndexMap<R::CrudColumn, crud_shared_types::Order>>,
        condition: Option<&crud_shared_types::condition::Condition>,
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
        order_by: Option<IndexMap<R::ReadViewCrudColumn, crud_shared_types::Order>>,
        condition: Option<&crud_shared_types::condition::Condition>,
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
        order_by: Option<IndexMap<R::ReadViewCrudColumn, crud_shared_types::Order>>,
        condition: Option<&crud_shared_types::condition::Condition>,
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

pub mod query {
    use crud_rs::prelude::*;
    use crud_shared_types::{
        condition::{Condition, ConditionElement, Operator},
        Order, Value,
    };
    use indexmap::IndexMap;
    use sea_orm::{
        ActiveModelTrait, ColumnTrait, DeleteMany, EntityTrait, FromQueryResult, Insert,
        ModelTrait, QueryFilter, QueryOrder, QuerySelect, Select,
    };
    use serde::de::DeserializeOwned;
    use snafu::{Backtrace, GenerateImplicitData};
    use std::hash::Hash;

    use super::SeaOrmRepoError;

    pub fn build_insert_query<R: CrudResource>(
        active_entity: R::ActiveModel,
    ) -> Result<Insert<R::ActiveModel>, SeaOrmRepoError> {
        // Building the "insert" query.
        let insert = R::Entity::insert(active_entity);
        Ok(insert)
    }

    pub fn build_delete_many_query<T: EntityTrait + MaybeColumnTrait>(
        condition: &Option<Condition>,
    ) -> Result<DeleteMany<T>, SeaOrmRepoError> {
        let mut delete_many = T::delete_many();

        if let Some(condition) = condition {
            delete_many = delete_many.filter(build_condition_tree::<T>(condition)?);
        }

        Ok(delete_many)
    }

    pub fn build_select_query<
        E: EntityTrait<Model = M, Column = C> + MaybeColumnTrait + 'static,
        M: ModelTrait + FromQueryResult + Sized + Send + Sync + 'static,
        A: ActiveModelTrait + 'static,
        C: ColumnTrait + 'static,
        CC: CrudColumns<C, M, A> + Eq + Hash + DeserializeOwned + 'static,
    >(
        limit: Option<u64>,
        skip: Option<u64>,
        order_by: Option<IndexMap<CC, Order>>,
        condition: Option<&Condition>,
    ) -> Result<Select<E>, SeaOrmRepoError> {
        let mut select = E::find();

        if let Some(limit) = limit {
            select = select.limit(limit);
        }

        if let Some(skip) = skip {
            select = select.offset(skip);
        }

        if let Some(map) = order_by {
            select = apply_order_by::<E, M, A, C, CC>(select, map)?;
        }

        if let Some(condition) = condition {
            select = select.filter(build_condition_tree::<E>(condition)?);
        }

        Ok(select)
    }

    // TODO: finalize
    //fn build_update_many_query<T: EntityTrait + MaybeColumnTrait>(
    //    condition: Option<Vec<ConditionElement>>,
    //) -> Result<UpdateMany<T>, CrudError> {
    //    let mut update = T::update_many();
    //
    //    //update.col_expr(T::Column::CakeId, Expr::value(Value::Null));
    //
    //    if let Some(elements) = condition {
    //        update = update.filter(build_condition_tree::<T>(elements)?);
    //    }
    //
    //    Ok(update)
    //}

    fn apply_order_by<
        E: EntityTrait<Column = C> + MaybeColumnTrait + 'static,
        M: ModelTrait + 'static,
        A: ActiveModelTrait + 'static,
        C: ColumnTrait + 'static,
        CC: CrudColumns<C, M, A> + Eq + Hash + DeserializeOwned + 'static,
    >(
        mut select: Select<E>,
        order_by: IndexMap<CC, Order>,
    ) -> Result<Select<E>, SeaOrmRepoError> {
        for (crud_column, order) in order_by {
            let column = crud_column.to_sea_orm_column();
            select = select.order_by(
                column,
                match order {
                    Order::Asc => sea_orm::Order::Asc,
                    Order::Desc => sea_orm::Order::Desc,
                },
            );
        }
        Ok(select)
    }

    // TODO: Implement this in crud-shared-types with sea-orm feature flag.
    //impl From<Condition> for Result<sea_orm::sea_query::Condition, CrudError> {
    //    fn from(condition: Condition) -> Self {
    //        todo!()
    //    }
    //}

    pub fn build_condition_tree<T: MaybeColumnTrait>(
        condition: &Condition,
    ) -> Result<sea_orm::sea_query::Condition, SeaOrmRepoError> {
        let mut tree = match &condition {
            Condition::All(_) => sea_orm::sea_query::Condition::all(),
            Condition::Any(_) => sea_orm::sea_query::Condition::any(),
        };

        match condition {
            Condition::All(elements) | Condition::Any(elements) => {
                for element in elements {
                    match element {
                        ConditionElement::Clause(clause) => match T::get_col(&clause.column_name) {
                            Some(col) => {
                                match col.as_col_type(clause.value.clone()).map_err(|err| {
                                    SeaOrmRepoError::UnableToParseValueAsColType {
                                        column_name: clause.column_name.clone(),
                                        reason: err,
                                        backtrace: Backtrace::generate(),
                                    }
                                })? {
                                    Value::String(val) => {
                                        tree = add_condition(tree, col, clause.operator, val)
                                    }
                                    Value::Json(val) => {
                                        tree = add_condition(tree, col, clause.operator, val)
                                    }
                                    Value::UuidV4(val) => {
                                        tree = add_condition(tree, col, clause.operator, val)
                                    }
                                    Value::UuidV7(val) => {
                                        tree = add_condition(tree, col, clause.operator, val)
                                    }
                                    Value::Ulid(val) => {
                                        let val: u128 = val.0;
                                        let val: uuid::Uuid = uuid::Uuid::from_u128(val);
                                        tree = add_condition(tree, col, clause.operator, val)
                                    }
                                    Value::I32(val) => {
                                        tree = add_condition(tree, col, clause.operator, val)
                                    }
                                    Value::I32Vec(values) => {
                                        tree = add_condition_iterable(
                                            tree,
                                            col,
                                            clause.operator,
                                            values,
                                        )
                                    }
                                    Value::I64(val) => {
                                        tree = add_condition(tree, col, clause.operator, val)
                                    }
                                    Value::U32(val) => {
                                        tree = add_condition(tree, col, clause.operator, val)
                                    }
                                    Value::F32(val) => {
                                        tree = add_condition(tree, col, clause.operator, val)
                                    }
                                    Value::Bool(val) => {
                                        tree = add_condition(tree, col, clause.operator, val)
                                    }
                                    Value::DateTime(val) => {
                                        tree = add_condition(tree, col, clause.operator, val)
                                    }
                                }
                            }
                            None => {
                                return Err(SeaOrmRepoError::UnknownColumnSpecified {
                                    column_name: clause.column_name.clone(),
                                    backtrace: Backtrace::generate(),
                                })
                            }
                        },
                        ConditionElement::Condition(nested_condition) => {
                            tree = tree.add(build_condition_tree::<T>(nested_condition)?);
                        }
                    }
                }
            }
        }

        Ok(tree)
    }

    fn add_condition<C, T>(
        tree: sea_orm::sea_query::Condition,
        col: C,
        operator: Operator,
        val: T,
    ) -> sea_orm::sea_query::Condition
    where
        C: ColumnTrait,
        T: Into<sea_orm::Value>,
    {
        match operator {
            Operator::Equal => tree.add(col.eq(val)),
            Operator::NotEqual => tree.add(col.ne(val)),
            Operator::Less => tree.add(col.lt(val)),
            Operator::LessOrEqual => tree.add(col.lte(val)),
            Operator::Greater => tree.add(col.gt(val)),
            Operator::GreaterOrEqual => tree.add(col.gte(val)),
            Operator::IsIn => panic!("This is a bug. Should have called add_condition_iterable!"),
        }
    }

    fn add_condition_iterable<C, T>(
        tree: sea_orm::sea_query::Condition,
        col: C,
        operator: Operator,
        val: T,
    ) -> sea_orm::sea_query::Condition
    where
        C: ColumnTrait,
        T: IntoIterator,
        sea_orm::Value: From<<T as IntoIterator>::Item>,
    {
        match operator {
            Operator::Equal => panic!("This is a bug. Should have called add_condition!"),
            Operator::NotEqual => panic!("This is a bug. Should have called add_condition!"),
            Operator::Less => panic!("This is a bug. Should have called add_condition!"),
            Operator::LessOrEqual => panic!("This is a bug. Should have called add_condition!"),
            Operator::Greater => panic!("This is a bug. Should have called add_condition!"),
            Operator::GreaterOrEqual => panic!("This is a bug. Should have called add_condition!"),
            Operator::IsIn => tree.add(col.is_in(val)),
        }
    }
}
