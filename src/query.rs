use crate::{
    AsColType, CreateModelTrait, CrudColumns, ExcludingColumnsOnInsert, FieldValidatorTrait,
    MaybeColumnTrait,
};
use crud_shared_types::{ConditionElement, ConditionOperator, CrudError, Operator, Order, Value};
use indexmap::IndexMap;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DeleteMany, EntityTrait, FromQueryResult, Insert, ModelTrait,
    QueryFilter, QueryOrder, QuerySelect, Select,
};

use serde::de::DeserializeOwned;
use std::{fmt::Debug, hash::Hash};

pub fn build_insert_query<C, M, A>(entity_json: &str) -> Result<Insert<A>, CrudError>
where
    C: CreateModelTrait + DeserializeOwned + Into<M> + Debug,
    M: ModelTrait,
    A: ActiveModelTrait
        + ExcludingColumnsOnInsert<<<A as ActiveModelTrait>::Entity as EntityTrait>::Column>
        + From<M>
        + FieldValidatorTrait,
    A::Entity: EntityTrait + MaybeColumnTrait,
{
    // Use the "CreateModel" to deserialize the given JSON. Some not required members are allowed to be missing.
    let create_model = serde_json::from_str::<C>(entity_json)
        .map_err(|err| CrudError::UnableToParseAsEntity(entity_json.to_owned(), err.to_string()))?;

    // Convert the "CreateModel" into the actual "Model"
    let model: M = create_model.into();

    // Create the "Model" into an "ActiveModel" ready to be persisted.
    let mut active_entity: A = model.into();

    let validation_results = active_entity.validate();
    if !validation_results.is_empty() {
        return Err(CrudError::ValidationErrors(validation_results));
    }

    // We might have accidentally set attributes on the "ActiveModel" that we must not have in order to persist it.
    let excluded = A::excluding_columns();
    for c in excluded {
        active_entity.not_set(*c);
    }

    // Building the "insert" query.
    let insert = A::Entity::insert(active_entity);
    Ok(insert)
}

// TODO: Does not work at the moment... Deserialization is throwing an error.
/*
fn build_insert_query<A>(entity_json: &str) -> Result<Insert<A>, CrudError>
where
    A: ActiveModelTrait + ExcludingColumnsOnInsert<<<A as ActiveModelTrait>::Entity as EntityTrait>::Column>,
    A::Entity: EntityTrait + MaybeColumnTrait,
    <<A as ActiveModelTrait>::Entity as EntityTrait>::Model: DeserializeOwned + IntoActiveModel<A>
{
    let json_value = serde_json::from_str(entity_json)
        .map_err(|err| CrudError::UnableToParseAsEntity(entity_json.to_owned(), err.to_string()))?;
    log::info!("{}", json_value);
    let active_entity = A::from_json(json_value)
        .map_err(|err| CrudError::DbError(err.to_string()))?;
    let insert = A::Entity::insert(active_entity);
    Ok(insert)
}
*/

pub fn build_delete_many_query<T: EntityTrait + MaybeColumnTrait>(
    condition: Option<Vec<ConditionElement>>,
) -> Result<DeleteMany<T>, CrudError> {
    let mut delete_many = T::delete_many();

    if let Some(elements) = condition {
        delete_many = delete_many.filter(build_condition_tree::<T>(elements)?);
    }

    Ok(delete_many)
}

pub fn build_select_query<
    E: EntityTrait<Model = M, Column = C> + MaybeColumnTrait,
    M: FromQueryResult + Sized + Send + Sync,
    C: ColumnTrait,
    CC: CrudColumns<C> + Eq + Hash + DeserializeOwned,
>(
    limit: Option<u64>,
    skip: Option<u64>,
    order_by: Option<IndexMap<CC, Order>>,
    condition: Option<Vec<ConditionElement>>,
) -> Result<Select<E>, CrudError> {
    let mut select = E::find();

    if let Some(limit) = limit {
        select = select.limit(limit);
    }

    if let Some(skip) = skip {
        select = select.offset(skip);
    }

    if let Some(map) = order_by {
        select = apply_order_by::<E, C, CC>(select, map)?;
    }

    if let Some(elements) = condition {
        select = select.filter(build_condition_tree::<E>(elements)?);
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

pub fn apply_order_by<
    T: EntityTrait<Column = C> + MaybeColumnTrait,
    C: ColumnTrait,
    CC: CrudColumns<C> + Eq + Hash + DeserializeOwned,
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

pub fn build_condition_tree<T: MaybeColumnTrait>(
    elements: Vec<ConditionElement>,
) -> Result<sea_orm::sea_query::Condition, CrudError> {
    let mut tree = sea_orm::sea_query::Condition::any();
    for element in elements {
        match element {
            ConditionElement::Clause(clause) => match T::get_col(&clause.column_name) {
                Some(col) => {
                    match col.as_col_type(clause.value).map_err(|err| {
                        CrudError::UnableToParseValueAsColType(clause.column_name, err)
                    })? {
                        Value::String(val) => tree = add_condition(tree, col, clause.operator, val),
                        Value::I32(val) => tree = add_condition(tree, col, clause.operator, val),
                        Value::Bool(val) => tree = add_condition(tree, col, clause.operator, val),
                        Value::DateTime(val) => {
                            tree = add_condition(tree, col, clause.operator, val)
                        }
                    }
                }
                None => return Err(CrudError::UnknownColumnSpecified(clause.column_name)),
            },
            ConditionElement::Operator(operator) => match operator {
                ConditionOperator::And => {}
                ConditionOperator::Or => {}
            },
        }
    }
    Ok(tree)
}

pub fn add_condition<C, T>(
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
