use std::fmt::Debug;

use serde::{Deserialize, Serialize};
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
    String(String),
    Json(String),
    UuidV4(uuid::Uuid),
    UuidV7(uuid::Uuid),
    Bool(bool),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    U8Vec(Vec<u8>),
    I32Vec(Vec<i32>),
    I64Vec(Vec<i64>),
    //DateTime(time::OffsetDateTime), // TODO: Enable time
}

// TODO: Use result type instead of panicking!
impl Into<ConditionClauseValue> for crudkit_shared::Value {
    fn into(self) -> ConditionClauseValue {
        match self {
            crudkit_shared::Value::String(value) => ConditionClauseValue::String(value),
            crudkit_shared::Value::Json(value) => ConditionClauseValue::Json(value.to_string()),
            crudkit_shared::Value::UuidV4(value) => ConditionClauseValue::UuidV4(value),
            crudkit_shared::Value::UuidV7(value) => ConditionClauseValue::UuidV7(value),
            crudkit_shared::Value::I32(value) => ConditionClauseValue::I32(value),
            crudkit_shared::Value::I64(value) => ConditionClauseValue::I64(value),
            crudkit_shared::Value::U8Vec(values) => ConditionClauseValue::U8Vec(values),
            crudkit_shared::Value::I32Vec(values) => ConditionClauseValue::I32Vec(values),
            crudkit_shared::Value::I64Vec(value) => ConditionClauseValue::I64Vec(value),
            crudkit_shared::Value::U32(value) => ConditionClauseValue::U32(value),
            crudkit_shared::Value::U64(value) => ConditionClauseValue::U64(value),
            crudkit_shared::Value::F32(value) => ConditionClauseValue::F32(value),
            crudkit_shared::Value::F64(value) => ConditionClauseValue::F64(value),
            crudkit_shared::Value::Bool(value) => ConditionClauseValue::Bool(value),
            //crudkit_shared::Value::DateTime(value) => ConditionClauseValue::DateTime(value), // TODO: implement
            crudkit_shared::Value::PrimitiveDateTime(_value) => panic!("Not implemented...."),
            crudkit_shared::Value::OffsetDateTime(_value) => panic!("Not implemented...."),
        }
    }
}

// TODO: Use result type instead of panicking!
impl Into<ConditionClauseValue> for crudkit_id::IdValue {
    fn into(self) -> ConditionClauseValue {
        match self {
            crudkit_id::IdValue::String(value) => ConditionClauseValue::String(value),
            crudkit_id::IdValue::UuidV4(value) => ConditionClauseValue::UuidV4(value),
            crudkit_id::IdValue::UuidV7(value) => ConditionClauseValue::UuidV7(value),
            crudkit_id::IdValue::I32(value) => ConditionClauseValue::I32(value),
            crudkit_id::IdValue::I64(value) => ConditionClauseValue::I64(value),
            crudkit_id::IdValue::U32(value) => ConditionClauseValue::U32(value),
            crudkit_id::IdValue::U64(value) => ConditionClauseValue::U64(value),
            crudkit_id::IdValue::Bool(value) => ConditionClauseValue::Bool(value),
            //crudkit_id::IdValue::DateTime(value) => ConditionClauseValue::DateTime(value), // TODO: implement
            crudkit_id::IdValue::PrimitiveDateTime(_value) => panic!("Not implemented...."),
            crudkit_id::IdValue::OffsetDateTime(_value) => panic!("Not implemented...."),
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
