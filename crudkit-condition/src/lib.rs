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
/// You can convert any `crudkit_shared::Value` using `.into`.
/// You can convert any `crudkit_id::IdValue` using `.into`.
// TODO: Drop in favor of "crudkit_shared::Value" type??
#[derive(Debug, Clone, PartialEq, ToSchema, Serialize, Deserialize)]
pub enum ConditionClauseValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
    Bool(bool),
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

// TODO: Use result type instead of panicking!
impl Into<ConditionClauseValue> for Value {
    fn into(self) -> ConditionClauseValue {
        match self {
            Value::String(value) => ConditionClauseValue::String(value),
            Value::Json(value) => ConditionClauseValue::Json(value.to_string()),
            Value::Uuid(value) => ConditionClauseValue::Uuid(value),
            Value::I32(value) => ConditionClauseValue::I32(value),
            Value::I64(value) => ConditionClauseValue::I64(value),
            Value::U8Vec(values) => ConditionClauseValue::U8Vec(values),
            Value::I32Vec(values) => ConditionClauseValue::I32Vec(values),
            Value::I64Vec(value) => ConditionClauseValue::I64Vec(value),
            Value::U32(value) => ConditionClauseValue::U32(value),
            Value::U64(value) => ConditionClauseValue::U64(value),
            Value::F32(value) => ConditionClauseValue::F32(value),
            Value::F64(value) => ConditionClauseValue::F64(value),
            Value::Bool(value) => ConditionClauseValue::Bool(value),
            //crudkit_shared::Value::DateTime(value) => ConditionClauseValue::DateTime(value), // TODO: implement
            Value::PrimitiveDateTime(_value) => unimplemented!(),
            Value::OffsetDateTime(_value) => unimplemented!(),
            Value::Duration(_value) => unimplemented!(),
        }
    }
}

// TODO: Use result type instead of panicking!
impl Into<ConditionClauseValue> for IdValue {
    fn into(self) -> ConditionClauseValue {
        match self {
            IdValue::I32(value) => ConditionClauseValue::I32(value),
            IdValue::U32(value) => ConditionClauseValue::U32(value),
            IdValue::I64(value) => ConditionClauseValue::I64(value),
            IdValue::U64(value) => ConditionClauseValue::U64(value),
            IdValue::I128(value) => ConditionClauseValue::I128(value),
            IdValue::U128(value) => ConditionClauseValue::U128(value),
            IdValue::Bool(value) => ConditionClauseValue::Bool(value),
            IdValue::String(value) => ConditionClauseValue::String(value),
            IdValue::Uuid(value) => ConditionClauseValue::Uuid(value),
            //crudkit_id::IdValue::DateTime(value) => ConditionClauseValue::DateTime(value), // TODO: implement
            IdValue::PrimitiveDateTime(_value) => panic!("Not implemented...."),
            IdValue::OffsetDateTime(_value) => panic!("Not implemented...."),
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

pub trait IntoConditionClauseValue {
    fn into_condition_clause_value(&self) -> ConditionClauseValue;
}

pub trait IntoAllEqualCondition {
    fn into_all_equal_condition(self) -> Condition;
}

impl<'a, V, I> IntoAllEqualCondition for I
where
    V: Into<ConditionClauseValue> + Clone + 'a,
    I: Iterator<Item = (String, V)>,
{
    fn into_all_equal_condition(self) -> Condition {
        Condition::All(
            self.map(|(name, value)| {
                ConditionElement::Clause(ConditionClause {
                    column_name: name.clone(),
                    operator: Operator::Equal,
                    value: value.clone().into(),
                })
            })
            .collect::<Vec<_>>(),
        )
    }
}
