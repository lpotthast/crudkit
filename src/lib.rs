use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crud_id::IdValue;
use crud_shared::Value;

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

// TODO: Drop in favor of "Value" type
#[derive(Debug, Clone, PartialEq, ToSchema, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionClauseValue {
    String(String),
    Json(String),
    #[schema(value_type = String)]
    UuidV4(uuid::Uuid),
    #[schema(value_type = String)]
    UuidV7(uuid::Uuid),
    Bool(bool),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    I32Vec(Vec<i32>),
    //DateTime(time::OffsetDateTime), // TODO: Enable time
}

// TODO: Use result type instead of panicking!
impl Into<ConditionClauseValue> for Value {
    fn into(self) -> ConditionClauseValue {
        match self {
            Value::String(value) => ConditionClauseValue::String(value),
            Value::Json(value) => ConditionClauseValue::Json(value.to_string()),
            Value::UuidV4(value) => ConditionClauseValue::UuidV4(value),
            Value::UuidV7(value) => ConditionClauseValue::UuidV7(value),
            Value::I32(value) => ConditionClauseValue::I32(value),
            Value::I32Vec(values) => ConditionClauseValue::I32Vec(values),
            Value::I64(value) => ConditionClauseValue::I64(value),
            Value::U32(value) => ConditionClauseValue::U32(value),
            Value::F32(value) => ConditionClauseValue::F32(value),
            Value::Bool(value) => ConditionClauseValue::Bool(value),
            //Value::DateTime(value) => ConditionClauseValue::DateTime(value), // TODO: implement
            Value::PrimitiveDateTime(_value) => panic!("Not implemented...."),
            Value::OffsetDateTime(_value) => panic!("Not implemented...."),
        }
    }
}

// TODO: Use result type instead of panicking!
impl Into<ConditionClauseValue> for IdValue {
    fn into(self) -> ConditionClauseValue {
        match self {
            IdValue::String(value) => ConditionClauseValue::String(value),
            IdValue::UuidV4(value) => ConditionClauseValue::UuidV4(value),
            IdValue::UuidV7(value) => ConditionClauseValue::UuidV7(value),
            IdValue::I32(value) => ConditionClauseValue::I32(value),
            IdValue::I64(value) => ConditionClauseValue::I64(value),
            IdValue::U32(value) => ConditionClauseValue::U32(value),
            IdValue::Bool(value) => ConditionClauseValue::Bool(value),
            //IdValue::DateTime(value) => ConditionClauseValue::DateTime(value), // TODO: implement
            IdValue::PrimitiveDateTime(_value) => panic!("Not implemented...."),
            IdValue::OffsetDateTime(_value) => panic!("Not implemented...."),
        }
    }
}

#[derive(Debug, Clone, PartialEq, ToSchema, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionElement {
    Clause(ConditionClause),
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

pub fn merge_conditions(a: Option<Condition>, b: Option<Condition>) -> Option<Condition> {
    if a.is_none() && b.is_none() {
        None
    } else if a.is_some() && b.is_none() {
        a
    } else if a.is_none() && b.is_some() {
        b
    } else {
        let mut combined = Condition::all();
        combined.push_condition(a.unwrap());
        combined.push_condition(b.unwrap());
        if combined.is_empty() {
            None
        } else {
            Some(combined)
        }
    }
}

pub trait IntoConditionClauseValue {
    fn into_condition_clause_value(&self) -> ConditionClauseValue;
}

pub trait IntoAllEqualCondition {
    fn into_all_equal_condition(self) -> Condition;
}

impl<'a, V: Into<ConditionClauseValue> + Clone + 'a, I: Iterator<Item = (String, V)>>
    IntoAllEqualCondition for I
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
