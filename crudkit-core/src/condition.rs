//! Query filtering DSL with condition clauses and operators.

use crate::id::{IdValue, SerializableIdEntry};
use crate::Value;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::str::FromStr;
use time::format_description::well_known::Rfc3339;
use utoipa::ToSchema;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, ToSchema, Serialize, Deserialize)]
pub enum Operator {
    #[serde(rename = "=")]
    Equal,
    #[serde(rename = "!=")]
    NotEqual,
    #[serde(rename = "<")]
    Less,
    #[serde(rename = "<=")]
    LessOrEqual,
    #[serde(rename = ">")]
    Greater,
    #[serde(rename = ">=")]
    GreaterOrEqual,
    #[serde(rename = "is_in")]
    IsIn,
}

#[derive(Debug, Clone, PartialEq, ToSchema, Serialize, Deserialize)]
pub struct ConditionClause {
    pub column_name: String,
    pub operator: Operator,
    pub value: ConditionClauseValue,
}

/// Values which might be part of a `ConditionClause`.
/// You can convert a `crudkit_core::Value` using `.try_into`.
/// You can convert a `crudkit_core::id::IdValue` using `.try_into`.
#[derive(Debug, Clone, PartialEq, ToSchema, Serialize, Deserialize)]
pub enum ConditionClauseValue {
    Bool(bool),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),

    F32(f32),
    F64(f64),

    String(String),
    Json(serde_json::Value),

    Uuid(uuid::Uuid),

    U8Vec(Vec<u8>),
    I32Vec(Vec<i32>),
    I64Vec(Vec<i64>),
    // TODO: Add additional vec types
    //DateTime(time::OffsetDateTime), // TODO: Enable time
}

#[derive(Debug, thiserror::Error)]
#[error("The value '{value:?}' cannot be used in a condition clause.")]
pub struct NotConditionClauseCompatibleValue {
    value: Value,
}

impl TryFrom<Value> for ConditionClauseValue {
    type Error = NotConditionClauseCompatibleValue;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            value @ Value::Null => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::Void(()) => Err(NotConditionClauseCompatibleValue { value }),

            Value::Bool(value) => Ok(Self::Bool(value)),

            Value::I8(value) => Ok(Self::I8(value)),
            Value::I16(value) => Ok(Self::I16(value)),
            Value::I32(value) => Ok(Self::I32(value)),
            Value::I64(value) => Ok(Self::I64(value)),
            Value::I128(value) => Ok(Self::I128(value)),

            Value::U8(value) => Ok(Self::U8(value)),
            Value::U16(value) => Ok(Self::U16(value)),
            Value::U32(value) => Ok(Self::U32(value)),
            Value::U64(value) => Ok(Self::U64(value)),
            Value::U128(value) => Ok(Self::U128(value)),

            Value::F32(value) => Ok(Self::F32(value)),
            Value::F64(value) => Ok(Self::F64(value)),

            Value::String(value) => Ok(Self::String(value)),

            // Ecosystem support.
            Value::Json(value) => Ok(Self::Json(value)),
            Value::Uuid(value) => Ok(Self::Uuid(value)),

            value @ Value::PrimitiveDateTime(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::OffsetDateTime(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::Duration(_) => Err(NotConditionClauseCompatibleValue { value }),

            value @ Value::Array(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::Other(_) => Err(NotConditionClauseCompatibleValue { value }),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("The ID value '{value:?}' cannot be used in a condition clause.")]
pub struct NotConditionClauseCompatibleIdValue {
    value: IdValue,
}

impl TryFrom<IdValue> for ConditionClauseValue {
    type Error = NotConditionClauseCompatibleIdValue;

    fn try_from(value: IdValue) -> Result<Self, Self::Error> {
        match value {
            IdValue::I8(value) => Ok(Self::I8(value)),
            IdValue::I16(value) => Ok(Self::I16(value)),
            IdValue::I32(value) => Ok(Self::I32(value)),
            IdValue::I64(value) => Ok(Self::I64(value)),
            IdValue::I128(value) => Ok(Self::I128(value)),
            IdValue::U8(value) => Ok(Self::U8(value)),
            IdValue::U16(value) => Ok(Self::U16(value)),
            IdValue::U32(value) => Ok(Self::U32(value)),
            IdValue::U64(value) => Ok(Self::U64(value)),
            IdValue::U128(value) => Ok(Self::U128(value)),
            IdValue::Bool(value) => Ok(Self::Bool(value)),
            IdValue::String(value) => Ok(Self::String(value)),
            IdValue::Uuid(value) => Ok(Self::Uuid(value)),
            value @ IdValue::PrimitiveDateTime(_) => {
                Err(NotConditionClauseCompatibleIdValue { value })
            }
            value @ IdValue::OffsetDateTime(_) => {
                Err(NotConditionClauseCompatibleIdValue { value })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, ToSchema, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionElement {
    Clause(ConditionClause),

    #[schema(no_recursion)]
    Condition(Box<Condition>),
}

/// A query condition combining multiple elements with AND (`All`) or OR (`Any`) logic.
///
/// # JSON Format
///
/// Uses externally tagged serialization for unambiguous deserialization:
/// - `{"All": [...]}`  - All elements must match (AND)
/// - `{"Any": [...]}`  - Any element must match (OR)
///
/// # Example
///
/// ```json
/// {
///   "Any": [
///     {"All": [{"column_name": "id", "operator": "=", "value": {"I64": 1}}]},
///     {"All": [{"column_name": "id", "operator": "=", "value": {"I64": 2}}]}
///   ]
/// }
/// ```
#[derive(Debug, Clone, PartialEq, ToSchema, Serialize, Deserialize)]
pub enum Condition {
    /// All elements must match (logical AND).
    All(Vec<ConditionElement>),

    /// Any element must match (logical OR).
    Any(Vec<ConditionElement>),
}

impl Condition {
    /// Creates an empty `All` condition.
    ///
    /// An empty `All` condition matches everything (vacuous truth).
    pub fn all() -> Self {
        Self::All(Vec::new())
    }

    /// Creates an empty `Any` condition.
    ///
    /// An empty `Any` condition matches nothing (no element can satisfy it).
    pub fn any() -> Self {
        Self::Any(Vec::new())
    }

    /// Creates a condition that matches nothing.
    ///
    /// This is a semantic alias for [`Self::any()`] - an empty OR condition.
    ///
    /// Useful when building conditions dynamically and you need a neutral starting
    /// point that won't match any entities (e.g., when no entities are selected
    /// for a bulk operation).
    pub fn none() -> Self {
        Self::any()
    }

    pub fn push_elements(&mut self, mut elements: Vec<ConditionElement>) {
        match self {
            Condition::All(vec) | Condition::Any(vec) => vec.append(&mut elements),
        }
    }

    pub fn push_condition(&mut self, condition: Condition) {
        match self {
            Condition::All(vec) | Condition::Any(vec) => {
                vec.push(ConditionElement::Condition(Box::new(condition)))
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Condition::All(vec) | Condition::Any(vec) => vec.is_empty(),
        }
    }
}

// TODO: This always "AND"s them together. Are there places where an "OR" would be equally appropriate?
pub fn merge_conditions(a: Option<Condition>, b: Option<Condition>) -> Option<Condition> {
    match (a, b) {
        (None, None) => None,
        (None, Some(b)) => Some(b),
        (Some(a), None) => Some(a),
        (Some(a), Some(b)) => {
            let mut combined = Condition::all();
            combined.push_condition(a);
            combined.push_condition(b);
            match combined.is_empty() {
                true => None,
                false => Some(combined),
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Could not build all-equal-condition.")]
pub struct IntoAllEqualConditionError;

pub trait TryIntoAllEqualCondition {
    type Error;

    fn try_into_all_equal_condition(self) -> Result<Condition, Self::Error>;
}

impl<I> TryIntoAllEqualCondition for I
where
    I: Iterator<Item = SerializableIdEntry>,
{
    type Error = IntoAllEqualConditionError;

    fn try_into_all_equal_condition(self) -> Result<Condition, Self::Error> {
        let mut clauses = Vec::new();

        for SerializableIdEntry { field_name, value } in self {
            let clause = ConditionElement::Clause(ConditionClause {
                column_name: field_name,
                operator: Operator::Equal,
                value: value
                    .clone()
                    .try_into()
                    .map_err(|_| IntoAllEqualConditionError)?,
            });

            clauses.push(clause);
        }

        Ok(Condition::All(clauses))
    }
}

impl ConditionClauseValue {
    // TODO: All these to functions support string->type parsing. Should this be removed and made explicit?
    pub fn to_i32(self) -> Result<Value, String> {
        match self {
            ConditionClauseValue::I32(num) => Ok(Value::I32(num)),
            ConditionClauseValue::I32Vec(numbers) => Ok(Value::Array(
                numbers.into_iter().map(Value::I32).collect(),
            )),
            ConditionClauseValue::String(string) => parse::<i32>(&string).map(Value::I32),
            _ => Err(format!(
                "{self:?} can not be converted to an i32 or Vec<i32>. Expected i32 or Vec<i32> or String."
            )),
        }
    }

    pub fn to_i64(self) -> Result<Value, String> {
        match self {
            ConditionClauseValue::I64(num) => Ok(Value::I64(num)),
            ConditionClauseValue::I64Vec(numbers) => Ok(Value::Array(
                numbers.into_iter().map(Value::I64).collect(),
            )),
            ConditionClauseValue::String(string) => parse::<i64>(&string).map(Value::I64),
            _ => Err(format!(
                "{self:?} can not be converted to an i64. Expected i64 or String."
            )),
        }
    }

    pub fn to_u32(self) -> Result<Value, String> {
        match self {
            ConditionClauseValue::U32(num) => Ok(Value::U32(num)),
            ConditionClauseValue::String(string) => parse::<u32>(&string).map(Value::U32),
            _ => Err(format!(
                "{self:?} can not be converted to an u32. Expected u32 or String."
            )),
        }
    }

    pub fn to_f32(self) -> Result<Value, String> {
        match self {
            ConditionClauseValue::F32(num) => Ok(Value::F32(num)),
            ConditionClauseValue::String(string) => parse::<f32>(&string).map(Value::F32),
            _ => Err(format!(
                "{self:?} can not be converted to an f32. Expected f32 or String."
            )),
        }
    }

    pub fn to_f64(self) -> Result<Value, String> {
        match self {
            ConditionClauseValue::F64(num) => Ok(Value::F64(num)),
            ConditionClauseValue::String(string) => parse::<f64>(&string).map(Value::F64),
            _ => Err(format!(
                "{self:?} can not be converted to an f32. Expected f64 or String."
            )),
        }
    }

    pub fn to_byte_vec(self) -> Result<Value, String> {
        match self {
            ConditionClauseValue::U8Vec(vec) => Ok(Value::Array(
                vec.into_iter().map(Value::U8).collect(),
            )),
            _ => Err(format!(
                "{self:?} can not be converted to an U8Vec. Expected U8Vec."
            )),
        }
    }

    pub fn to_bool(self) -> Result<Value, String> {
        match self {
            ConditionClauseValue::Bool(bool) => Ok(Value::Bool(bool)),
            ConditionClauseValue::String(string) => parse::<bool>(&string).map(Value::Bool),
            _ => Err(format!(
                "{self:?} can not be converted to a bool. Expected bool or String."
            )),
        }
    }

    pub fn to_string(self) -> Result<Value, String> {
        match self {
            ConditionClauseValue::String(string) => Ok(Value::String(string)),
            _ => Err(format!(
                "{self:?} can not be converted to a String. Expected String."
            )),
        }
    }

    pub fn to_json_value(self) -> Result<Value, String> {
        match self {
            ConditionClauseValue::String(string) => Ok(Value::String(string)),
            _ => Err(format!(
                "{self:?} can not be converted to a String. Expected String."
            )),
        }
    }

    pub fn to_uuid(self) -> Result<Value, String> {
        match self {
            ConditionClauseValue::Uuid(uuid) => Ok(Value::Uuid(uuid)),
            _ => Err(format!(
                "{self:?} can not be converted to a Uuid. Expected Uuid."
            )),
        }
    }

    pub fn to_primitive_date_time(self) -> Result<Value, String> {
        match self {
            ConditionClauseValue::String(string) => {
                time::PrimitiveDateTime::parse(&string, &Rfc3339)
                    .map_err(|err| err.to_string())
                    .map(Value::PrimitiveDateTime)
            }
            _ => Err(format!(
                "{self:?} can not be converted to a PrimitiveDateTime. Expected String."
            )),
        }
    }

    pub fn to_offset_date_time(self) -> Result<Value, String> {
        match self {
            ConditionClauseValue::String(string) => time::OffsetDateTime::parse(&string, &Rfc3339)
                .map_err(|err| err.to_string())
                .map(Value::OffsetDateTime),
            _ => Err(format!(
                "{self:?} can not be converted to an OffsetDateTime. Expected String."
            )),
        }
    }

    //pub fn to_time(self) -> Result<Value, String> {
    //    match self {
    //        ConditionClauseValue::String(string) => {
    //            let format = format_description!("[hour]:[minute]:[second]");
    //            time::Time::parse(&string, &format)
    //                .map_err(|err| err.to_string())
    //                .map(Value::Time)
    //        }
    //        _ => Err(format!(
    //            "{value:?} can not be converted to a Duration. Expected String with format '[hour]:[minute]:[second]'!"
    //        )),
    //    }
    //}

    pub fn to_time_duration(self) -> Result<Value, String> {
        unimplemented!()
    }
}

fn parse<T>(string: &str) -> Result<T, String>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    string.parse::<T>().map_err(|e| format!("{}", e))
}
