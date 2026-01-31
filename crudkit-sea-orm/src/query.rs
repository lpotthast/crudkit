//! Query building functions for SeaORM.
//!
//! These functions build SeaORM queries using the storage-agnostic `CrudResource`
//! trait combined with the `SeaOrmResource` trait for SeaORM-specific mappings.

use crate::newtypes::TimeDuration;
use crate::repo::SeaOrmRepoError;
use crate::traits::SeaOrmResource;
use crudkit_core::condition::{Condition, ConditionElement, Operator};
use crudkit_core::{Order, Value};
use crudkit_rs::prelude::*;
use indexmap::IndexMap;
use sea_orm::{ColumnTrait, EntityTrait, Insert, QueryFilter, QueryOrder, QuerySelect, Select};
use snafu::{Backtrace, GenerateImplicitData};

/// Build an insert query using the SeaOrmResource trait.
pub fn build_insert_query<R>(
    active_entity: R::ActiveModel,
) -> Result<Insert<R::ActiveModel>, SeaOrmRepoError>
where
    R: CrudResource + SeaOrmResource,
{
    let insert = R::Entity::insert(active_entity);
    Ok(insert)
}

/// Build a select query for the main entity using the SeaOrmResource trait.
pub fn build_select_query<R>(
    limit: Option<u64>,
    skip: Option<u64>,
    order_by: Option<IndexMap<R::ModelField, Order>>,
    condition: Option<&Condition>,
) -> Result<Select<R::Entity>, SeaOrmRepoError>
where
    R: CrudResource + SeaOrmResource,
{
    let mut select = R::Entity::find();

    if let Some(limit) = limit {
        select = select.limit(limit);
    }

    if let Some(skip) = skip {
        select = select.offset(skip);
    }

    if let Some(map) = order_by {
        for (field, order) in map {
            let column = R::model_field_to_column(&field);
            select = select.order_by(
                column,
                match order {
                    Order::Asc => sea_orm::Order::Asc,
                    Order::Desc => sea_orm::Order::Desc,
                },
            );
        }
    }

    if let Some(condition) = condition {
        select = select.filter(build_condition_tree::<R::ModelField, R::Column>(
            condition,
            R::model_field_to_column,
        )?);
    }

    Ok(select)
}

/// Build a select query for the read view using the SeaOrmResource trait.
pub fn build_read_view_query<R>(
    limit: Option<u64>,
    skip: Option<u64>,
    order_by: Option<IndexMap<R::ReadModelField, Order>>,
    condition: Option<&Condition>,
) -> Result<Select<R::ReadViewEntity>, SeaOrmRepoError>
where
    R: CrudResource + SeaOrmResource,
{
    let mut select = R::ReadViewEntity::find();

    if let Some(limit) = limit {
        select = select.limit(limit);
    }

    if let Some(skip) = skip {
        select = select.offset(skip);
    }

    if let Some(map) = order_by {
        for (field, order) in map {
            let column = R::read_model_field_to_column(&field);
            select = select.order_by(
                column,
                match order {
                    Order::Asc => sea_orm::Order::Asc,
                    Order::Desc => sea_orm::Order::Desc,
                },
            );
        }
    }

    if let Some(condition) = condition {
        select = select.filter(
            build_condition_tree::<R::ReadModelField, R::ReadViewColumn>(
                condition,
                R::read_model_field_to_column,
            )?,
        );
    }

    Ok(select)
}

/// Build a condition tree using the field-based approach.
fn build_condition_tree<F, C>(
    condition: &Condition,
    field_to_column: fn(&F) -> C,
) -> Result<sea_query::Condition, SeaOrmRepoError>
where
    F: Field + FieldLookup + ConditionValueConverter,
    C: ColumnTrait,
{
    let mut tree = match &condition {
        Condition::All(_) => sea_query::Condition::all(),
        Condition::Any(_) => sea_query::Condition::any(),
    };

    match condition {
        Condition::All(elements) | Condition::Any(elements) => {
            for element in elements {
                match element {
                    ConditionElement::Clause(clause) => {
                        // Look up the field by name.
                        let field = F::from_name(&clause.column_name).ok_or_else(|| {
                            SeaOrmRepoError::UnknownColumnSpecified {
                                column_name: clause.column_name.clone(),
                                backtrace: Backtrace::generate(),
                            }
                        })?;

                        // Convert the condition value to a typed Value.
                        let value = field
                            .convert_condition_value(clause.value.clone())
                            .map_err(|err| SeaOrmRepoError::UnableToParseValueAsColType {
                                column_name: clause.column_name.clone(),
                                reason: err,
                                backtrace: Backtrace::generate(),
                            })?;

                        // Get the SeaORM column.
                        let col = field_to_column(&field);

                        // Add the condition based on value type.
                        tree = add_condition_from_value(tree, col, clause.operator, value)?;
                    }
                    ConditionElement::Condition(nested_condition) => {
                        tree = tree.add(build_condition_tree::<F, C>(
                            nested_condition,
                            field_to_column,
                        )?);
                    }
                }
            }
        }
    }

    Ok(tree)
}

/// Add a condition to the tree based on the Value type.
fn add_condition_from_value<C: ColumnTrait>(
    tree: sea_query::Condition,
    col: C,
    operator: Operator,
    value: Value,
) -> Result<sea_query::Condition, SeaOrmRepoError> {
    let tree = match value {
        // Null represents explicit absence - use IS NULL condition.
        Value::Null => match operator {
            Operator::Equal => tree.add(col.is_null()),
            Operator::NotEqual => tree.add(col.is_not_null()),
            _ => unimplemented!("Only Equal/NotEqual operators supported for Null"),
        },

        Value::Bool(val) => add_condition(tree, col, operator, val),
        Value::U8(val) => add_condition(tree, col, operator, val),
        Value::U16(val) => add_condition(tree, col, operator, val),
        Value::U32(val) => add_condition(tree, col, operator, val),
        Value::U64(val) => add_condition(tree, col, operator, val),
        // SeaORM doesn't support i128/u128 directly - log a warning.
        Value::U128(_) => {
            tracing::warn!("U128 values in conditions are not supported by SeaORM");
            tree
        }
        Value::I8(val) => add_condition(tree, col, operator, val),
        Value::I16(val) => add_condition(tree, col, operator, val),
        Value::I32(val) => add_condition(tree, col, operator, val),
        Value::I64(val) => add_condition(tree, col, operator, val),
        Value::I128(_) => {
            tracing::warn!("I128 values in conditions are not supported by SeaORM");
            tree
        }
        Value::F32(val) => add_condition(tree, col, operator, val),
        Value::F64(val) => add_condition(tree, col, operator, val),
        Value::String(val) => add_condition(tree, col, operator, val),
        Value::Json(val) => add_condition(tree, col, operator, val),
        Value::Uuid(val) => add_condition(tree, col, operator, val),
        Value::PrimitiveDateTime(val) => add_condition(tree, col, operator, val),
        Value::OffsetDateTime(val) => add_condition(tree, col, operator, val),
        Value::Duration(val) => add_condition(tree, col, operator, TimeDuration(val.0)),

        // Array is used for IN conditions.
        Value::Array(values) => {
            if operator != Operator::IsIn {
                panic!(
                    "Array value can only be used with IsIn operator, got {:?}",
                    operator
                );
            }
            if let Err(index) = Value::verify_array_homogeneity(&values) {
                panic!(
                    "Array elements must be homogeneous. Element at index {} has different type than first element.",
                    index
                );
            }
            // Convert each element to sea_orm::Value and use is_in.
            let sea_values: Vec<sea_orm::Value> =
                values.into_iter().map(value_to_sea_orm_value).collect();
            tree.add(col.is_in(sea_values))
        }

        Value::Void(_) => unimplemented!("Void value in condition"),
        Value::Other(_) => unimplemented!("Other value in condition"),
    };
    Ok(tree)
}

fn add_condition<C, T>(
    tree: sea_query::Condition,
    col: C,
    operator: Operator,
    val: T,
) -> sea_query::Condition
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
        Operator::IsIn => panic!("IsIn operator requires an array value, not a scalar"),
    }
}

/// Convert a crudkit Value to a sea_orm::Value for use in conditions.
fn value_to_sea_orm_value(value: Value) -> sea_orm::Value {
    match value {
        Value::Null => sea_orm::Value::String(None),
        Value::Bool(v) => v.into(),
        Value::U8(v) => v.into(),
        Value::U16(v) => (v as i32).into(),
        Value::U32(v) => v.into(),
        Value::U64(v) => v.into(),
        Value::U128(_) => panic!("U128 values are not supported by SeaORM"),
        Value::I8(v) => v.into(),
        Value::I16(v) => v.into(),
        Value::I32(v) => v.into(),
        Value::I64(v) => v.into(),
        Value::I128(_) => panic!("I128 values are not supported by SeaORM"),
        Value::F32(v) => v.into(),
        Value::F64(v) => v.into(),
        Value::String(v) => v.into(),
        Value::Json(v) => v.into(),
        Value::Uuid(v) => v.into(),
        Value::PrimitiveDateTime(v) => v.into(),
        Value::OffsetDateTime(v) => v.into(),
        Value::Duration(v) => TimeDuration(v.0).into(),
        Value::Void(_) => panic!("Void value cannot be converted to sea_orm::Value"),
        Value::Array(_) => panic!("Nested arrays are not supported in conditions"),
        Value::Other(_) => panic!("Other values are not supported in conditions"),
    }
}
