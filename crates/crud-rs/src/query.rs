use crate::prelude::*;
use crud_shared_types::{Condition, ConditionElement, CrudError, Operator, Order, Value};
use indexmap::IndexMap;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DeleteMany, EntityTrait, FromQueryResult, Insert, QueryFilter,
    QueryOrder, QuerySelect, Select,
};
use serde::de::DeserializeOwned;
use std::hash::Hash;

pub fn prune_active_model<R: CrudResource>(active_entity: &mut R::ActiveModel) {
    let excluded = R::ActiveModel::excluding_columns();
    for c in excluded {
        active_entity.not_set(*c);
    }
}

pub fn build_insert_query<R: CrudResource>(
    mut active_entity: R::ActiveModel,
) -> Result<Insert<R::ActiveModel>, CrudError> {
    // We might have accidentally set attributes on the "ActiveModel" that we must not have in order to persist it.
    prune_active_model::<R>(&mut active_entity);

    // Building the "insert" query.
    let insert = R::Entity::insert(active_entity);
    Ok(insert)
}

pub fn build_delete_many_query<T: EntityTrait + MaybeColumnTrait>(
    condition: &Option<Condition>,
) -> Result<DeleteMany<T>, CrudError> {
    let mut delete_many = T::delete_many();

    if let Some(condition) = condition {
        delete_many = delete_many.filter(build_condition_tree::<T>(condition)?);
    }

    Ok(delete_many)
}

pub fn build_select_query<
    E: EntityTrait<Model = M, Column = C> + MaybeColumnTrait,
    M: FromQueryResult + Sized + Send + Sync,
    A: ActiveModelTrait,
    C: ColumnTrait,
    CC: CrudColumns<C, A> + Eq + Hash + DeserializeOwned,
>(
    limit: Option<u64>,
    skip: Option<u64>,
    order_by: Option<IndexMap<CC, Order>>,
    condition: &Option<Condition>,
) -> Result<Select<E>, CrudError> {
    let mut select = E::find();

    if let Some(limit) = limit {
        select = select.limit(limit);
    }

    if let Some(skip) = skip {
        select = select.offset(skip);
    }

    if let Some(map) = order_by {
        select = apply_order_by::<E, A, C, CC>(select, map)?;
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
    T: EntityTrait<Column = C> + MaybeColumnTrait,
    A: ActiveModelTrait,
    C: ColumnTrait,
    CC: CrudColumns<C, A> + Eq + Hash + DeserializeOwned,
>(
    mut select: Select<T>,
    order_by: IndexMap<CC, Order>,
) -> Result<Select<T>, CrudError> {
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

fn build_condition_tree<T: MaybeColumnTrait>(
    condition: &Condition,
) -> Result<sea_orm::sea_query::Condition, CrudError> {
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
                                CrudError::UnableToParseValueAsColType(clause.column_name.clone(), err)
                            })? {
                                Value::String(val) => {
                                    tree = add_condition(tree, col, clause.operator, val)
                                }
                                Value::I32(val) => {
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
                        None => return Err(CrudError::UnknownColumnSpecified(clause.column_name.clone())),
                    },
                    ConditionElement::Condition(nested_condition) => {
                        let nested_condition: &Condition = &*nested_condition;
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
    }
}