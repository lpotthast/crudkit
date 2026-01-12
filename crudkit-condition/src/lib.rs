use crudkit_id::IdValue;
use crudkit_shared::Value;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
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
/// You can convert a `crudkit_shared::Value` using `.try_into`.
/// You can convert a `crudkit_id::IdValue` using `.try_into`.
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
    Json(String),

    Uuid(uuid::Uuid),
    UuidV7(uuid::Uuid),

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

impl TryInto<ConditionClauseValue> for Value {
    type Error = NotConditionClauseCompatibleValue;

    fn try_into(self) -> Result<ConditionClauseValue, Self::Error> {
        match self {
            value @ Value::Void(()) => Err(NotConditionClauseCompatibleValue { value }),

            Value::Bool(value) => Ok(ConditionClauseValue::Bool(value)),
            value @ Value::OptionalBool(_) => Err(NotConditionClauseCompatibleValue { value }),

            Value::I8(value) => Ok(ConditionClauseValue::I8(value)),
            Value::I16(value) => Ok(ConditionClauseValue::I16(value)),
            Value::I32(value) => Ok(ConditionClauseValue::I32(value)),
            Value::I64(value) => Ok(ConditionClauseValue::I64(value)),
            Value::I128(value) => Ok(ConditionClauseValue::I128(value)),
            value @ Value::OptionalI8(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::OptionalI16(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::OptionalI32(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::OptionalI64(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::OptionalI128(_) => Err(NotConditionClauseCompatibleValue { value }),

            Value::U8(value) => Ok(ConditionClauseValue::U8(value)),
            Value::U16(value) => Ok(ConditionClauseValue::U16(value)),
            Value::U32(value) => Ok(ConditionClauseValue::U32(value)),
            Value::U64(value) => Ok(ConditionClauseValue::U64(value)),
            Value::U128(value) => Ok(ConditionClauseValue::U128(value)),
            value @ Value::OptionalU8(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::OptionalU16(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::OptionalU32(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::OptionalU64(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::OptionalU128(_) => Err(NotConditionClauseCompatibleValue { value }),

            Value::F32(value) => Ok(ConditionClauseValue::F32(value)),
            Value::F64(value) => Ok(ConditionClauseValue::F64(value)),
            value @ Value::OptionalF32(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::OptionalF64(_) => Err(NotConditionClauseCompatibleValue { value }),

            value @ Value::U8Vec(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::I32Vec(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::I64Vec(_) => Err(NotConditionClauseCompatibleValue { value }),

            Value::String(value) => Ok(ConditionClauseValue::String(value)),
            value @ Value::OptionalString(_) => Err(NotConditionClauseCompatibleValue { value }),

            // Ecosystem support.
            Value::Json(value) => Ok(ConditionClauseValue::Json(
                serde_json::to_string(&value).unwrap(),
            )),
            value @ Value::OptionalJson(_) => Err(NotConditionClauseCompatibleValue { value }),
            Value::Uuid(value) => Ok(ConditionClauseValue::Uuid(value)),
            value @ Value::OptionalUuid(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::PrimitiveDateTime(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::OffsetDateTime(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::OptionalPrimitiveDateTime(_) => {
                Err(NotConditionClauseCompatibleValue { value })
            }
            value @ Value::OptionalOffsetDateTime(_) => {
                Err(NotConditionClauseCompatibleValue { value })
            }
            value @ Value::Duration(_) => Err(NotConditionClauseCompatibleValue { value }),
            value @ Value::OptionalDuration(_) => Err(NotConditionClauseCompatibleValue { value }),

            value @ Value::Other(_) => Err(NotConditionClauseCompatibleValue { value }),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("The ID value '{value:?}' cannot be used in a condition clause.")]
pub struct NotConditionClauseCompatibleIdValue {
    value: IdValue,
}

impl TryInto<ConditionClauseValue> for IdValue {
    type Error = NotConditionClauseCompatibleIdValue;

    fn try_into(self) -> Result<ConditionClauseValue, Self::Error> {
        match self {
            IdValue::I32(value) => Ok(ConditionClauseValue::I32(value)),
            IdValue::U32(value) => Ok(ConditionClauseValue::U32(value)),
            IdValue::I64(value) => Ok(ConditionClauseValue::I64(value)),
            IdValue::U64(value) => Ok(ConditionClauseValue::U64(value)),
            IdValue::I128(value) => Ok(ConditionClauseValue::I128(value)),
            IdValue::U128(value) => Ok(ConditionClauseValue::U128(value)),
            IdValue::Bool(value) => Ok(ConditionClauseValue::Bool(value)),
            IdValue::String(value) => Ok(ConditionClauseValue::String(value)),
            IdValue::Uuid(value) => Ok(ConditionClauseValue::Uuid(value)),
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

#[derive(Debug, Clone, PartialEq, ToSchema, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Condition {
    All(Vec<ConditionElement>),
    Any(Vec<ConditionElement>),
}

impl Condition {
    pub fn all() -> Self {
        Self::All(Vec::new())
    }

    pub fn any() -> Self {
        Self::Any(Vec::new())
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

impl<'a, V, I> TryIntoAllEqualCondition for I
where
    V: TryInto<ConditionClauseValue> + Clone + 'a,
    I: Iterator<Item = (String, V)>,
{
    type Error = IntoAllEqualConditionError;

    fn try_into_all_equal_condition(self) -> Result<Condition, Self::Error> {
        let mut clauses = Vec::new();

        for (name, value) in self {
            let clause = ConditionElement::Clause(ConditionClause {
                column_name: name.clone(),
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
