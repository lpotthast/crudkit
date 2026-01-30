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
        Value::String(val) => add_condition(tree, col, operator, val),
        Value::Json(val) => add_condition(tree, col, operator, val),
        Value::Uuid(val) => add_condition(tree, col, operator, val),
        Value::I32(val) => add_condition(tree, col, operator, val),
        Value::U8Vec(values) => add_condition_iterable(tree, col, operator, values),
        Value::I32Vec(values) => add_condition_iterable(tree, col, operator, values),
        Value::I64(val) => add_condition(tree, col, operator, val),
        Value::I64Vec(values) => add_condition_iterable(tree, col, operator, values),
        Value::U32(val) => add_condition(tree, col, operator, val),
        Value::U64(val) => add_condition(tree, col, operator, val),
        Value::F32(val) => add_condition(tree, col, operator, val),
        Value::F64(val) => add_condition(tree, col, operator, val),
        Value::Bool(val) => add_condition(tree, col, operator, val),
        Value::PrimitiveDateTime(val) => add_condition(tree, col, operator, val),
        Value::OffsetDateTime(val) => add_condition(tree, col, operator, val),
        Value::Duration(val) => add_condition(tree, col, operator, TimeDuration(val.0)),
        // TODO: Implement missing variants.
        Value::Void(_) => unimplemented!("Void value in condition"),
        Value::OptionalBool(_) => unimplemented!("OptionalBool value in condition"),
        Value::U8(_) => unimplemented!("U8 value in condition"),
        Value::U16(_) => unimplemented!("U16 value in condition"),
        Value::U128(_) => unimplemented!("U128 value in condition"),
        Value::OptionalU8(_) => unimplemented!("OptionalU8 value in condition"),
        Value::OptionalU16(_) => unimplemented!("OptionalU16 value in condition"),
        Value::OptionalU32(_) => unimplemented!("OptionalU32 value in condition"),
        Value::OptionalU64(_) => unimplemented!("OptionalU64 value in condition"),
        Value::OptionalU128(_) => unimplemented!("OptionalU128 value in condition"),
        Value::I8(_) => unimplemented!("I8 value in condition"),
        Value::I16(_) => unimplemented!("I16 value in condition"),
        Value::I128(_) => unimplemented!("I128 value in condition"),
        Value::OptionalI8(_) => unimplemented!("OptionalI8 value in condition"),
        Value::OptionalI16(_) => unimplemented!("OptionalI16 value in condition"),
        Value::OptionalI32(_) => unimplemented!("OptionalI32 value in condition"),
        Value::OptionalI64(_) => unimplemented!("OptionalI64 value in condition"),
        Value::OptionalI128(_) => unimplemented!("OptionalI128 value in condition"),
        Value::OptionalF32(_) => unimplemented!("OptionalF32 value in condition"),
        Value::OptionalF64(_) => unimplemented!("OptionalF64 value in condition"),
        Value::OptionalString(_) => unimplemented!("OptionalString value in condition"),
        Value::OptionalJson(_) => unimplemented!("OptionalJson value in condition"),
        Value::OptionalUuid(_) => unimplemented!("OptionalUuid value in condition"),
        Value::OptionalPrimitiveDateTime(_) => {
            unimplemented!("OptionalPrimitiveDateTime value in condition")
        }
        Value::OptionalOffsetDateTime(_) => {
            unimplemented!("OptionalOffsetDateTime value in condition")
        }
        Value::OptionalDuration(_) => unimplemented!("OptionalDuration value in condition"),
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
        Operator::IsIn => panic!("This is a bug. Should have called add_condition_iterable!"),
    }
}

fn add_condition_iterable<C, T>(
    tree: sea_query::Condition,
    col: C,
    operator: Operator,
    val: T,
) -> sea_query::Condition
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
