//! Query building functions for `SeaORM`.
//!
//! These functions build `SeaORM` queries using the storage-agnostic `CrudResource`
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

/// Build an insert query using the `SeaOrmResource` trait.
///
/// # Errors
///
/// Currently never returns an error; the `Result` keeps the signature aligned with the other query
/// builders.
pub fn build_insert_query<R>(
    active_entity: R::ActiveModel,
) -> Result<Insert<R::ActiveModel>, SeaOrmRepoError>
where
    R: CrudResource + SeaOrmResource,
{
    let insert = R::Entity::insert(active_entity);
    Ok(insert)
}

/// Build a select query for the main entity using the `SeaOrmResource` trait.
///
/// # Errors
///
/// Returns `SeaOrmRepoError::UnknownColumnSpecified` if the condition names an unknown field, or
/// `SeaOrmRepoError::UnableToParseValueAsColType` if a condition value cannot be converted to the
/// field's type.
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

/// Build a select query for the read view using the `SeaOrmResource` trait.
///
/// # Errors
///
/// Returns `SeaOrmRepoError::UnknownColumnSpecified` if the condition names an unknown field, or
/// `SeaOrmRepoError::UnableToParseValueAsColType` if a condition value cannot be converted to the
/// field's type.
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
                        tree = add_condition_from_value(tree, col, clause.operator, value)
                            .map_err(|reason| SeaOrmRepoError::UnsupportedCondition {
                                column_name: clause.column_name.clone(),
                                reason,
                                backtrace: Backtrace::generate(),
                            })?;
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
///
/// Returns why the condition cannot be expressed if the operator does not fit the value or SeaORM
/// cannot represent the value.
fn add_condition_from_value<C: ColumnTrait>(
    tree: sea_query::Condition,
    col: C,
    operator: Operator,
    value: Value,
) -> Result<sea_query::Condition, String> {
    match value {
        // Null represents explicit absence - use IS NULL condition.
        Value::Null => match operator {
            Operator::Equal => Ok(tree.add(col.is_null())),
            Operator::NotEqual => Ok(tree.add(col.is_not_null())),
            _ => Err(format!(
                "a null value only supports the Equal and NotEqual operators, got {operator:?}"
            )),
        },
        // Array is used for IN conditions.
        Value::Array(values) => {
            if operator != Operator::IsIn {
                return Err(format!(
                    "a list value only supports the IsIn operator, got {operator:?}"
                ));
            }
            if let Err(index) = Value::verify_array_homogeneity(&values) {
                return Err(format!(
                    "the list element at index {index} has a different type than the first element"
                ));
            }
            let sea_values = values
                .into_iter()
                .map(value_to_sea_orm_value)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(tree.add(col.is_in(sea_values)))
        }
        value => {
            let value = value_to_sea_orm_value(value)?;
            match operator {
                Operator::Equal => Ok(tree.add(col.eq(value))),
                Operator::NotEqual => Ok(tree.add(col.ne(value))),
                Operator::Less => Ok(tree.add(col.lt(value))),
                Operator::LessOrEqual => Ok(tree.add(col.lte(value))),
                Operator::Greater => Ok(tree.add(col.gt(value))),
                Operator::GreaterOrEqual => Ok(tree.add(col.gte(value))),
                Operator::IsIn => Err("the IsIn operator requires a list value".to_owned()),
            }
        }
    }
}

/// Convert a crudkit Value to a `sea_orm::Value` for use in conditions.
///
/// Returns why the value cannot be used if SeaORM cannot represent it.
fn value_to_sea_orm_value(value: Value) -> Result<sea_orm::Value, String> {
    Ok(match value {
        Value::Null => sea_orm::Value::String(None),
        Value::Bool(v) => v.into(),
        Value::U8(v) => v.into(),
        Value::U16(v) => i32::from(v).into(),
        Value::U32(v) => v.into(),
        Value::U64(v) => v.into(),
        Value::I8(v) => v.into(),
        Value::I16(v) => v.into(),
        Value::I32(v) => v.into(),
        Value::I64(v) => v.into(),
        Value::F32(v) => v.into(),
        Value::F64(v) => v.into(),
        Value::String(v) => v.into(),
        Value::Json(v) => v.into(),
        Value::Uuid(v) => v.into(),
        Value::PrimitiveDateTime(v) => v.into(),
        Value::OffsetDateTime(v) => v.into(),
        Value::Duration(v) => TimeDuration(v.0).into(),
        Value::U128(_) | Value::I128(_) => {
            return Err("128-bit integers are not supported by SeaORM".to_owned());
        }
        Value::Void(()) => return Err("a unit value cannot be compared".to_owned()),
        Value::Array(_) => return Err("nested lists are not supported".to_owned()),
        Value::Other(_) => return Err("custom values are not supported".to_owned()),
    })
}

#[cfg(test)]
mod tests {
    use super::add_condition_from_value;
    use crate::validation::unified::model::Column;
    use crudkit_core::Value;
    use crudkit_core::condition::Operator;

    fn condition(operator: Operator, value: Value) -> Result<sea_query::Condition, String> {
        add_condition_from_value(
            sea_query::Condition::all(),
            Column::ValidatorVersion,
            operator,
            value,
        )
    }

    #[test]
    fn supported_values_add_a_condition() {
        let scalar = condition(Operator::Equal, Value::I64(1)).expect("supported condition");
        assert!(!scalar.is_empty());

        let list = condition(
            Operator::IsIn,
            Value::Array(vec![Value::I64(1), Value::I64(2)]),
        )
        .expect("supported condition");
        assert!(!list.is_empty());

        let null = condition(Operator::NotEqual, Value::Null).expect("supported condition");
        assert!(!null.is_empty());
    }

    #[test]
    fn unsupported_conditions_are_rejected_instead_of_panicking_or_being_dropped() {
        let unsupported = [
            (Operator::Less, Value::Null),
            (Operator::Equal, Value::Array(vec![Value::I64(1)])),
            (
                Operator::IsIn,
                Value::Array(vec![Value::I64(1), Value::String("2".to_owned())]),
            ),
            (Operator::IsIn, Value::Array(vec![Value::U128(1)])),
            (Operator::IsIn, Value::I64(1)),
            (Operator::Equal, Value::U128(1)),
            (Operator::Equal, Value::I128(1)),
            (Operator::Equal, Value::Void(())),
        ];
        for (operator, value) in unsupported {
            let description = format!("{operator:?} {value:?}");
            assert!(condition(operator, value).is_err(), "{description}");
        }
    }
}
