#![forbid(unsafe_code)]

use std::fmt::{Debug, Display};

use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};

pub mod validation;
pub mod ws_messages;

pub mod prelude {
    pub use crate::Condition;
    pub use crate::ConditionClause;
    pub use crate::ConditionClauseValue;
    pub use crate::ConditionElement;
    pub use crate::CrudError;
    pub use crate::DynIdField;
    pub use crate::IdField;
    pub use crate::IdFieldValue;
    pub use crate::Id;
    pub use crate::IntoAllEqualCondition;
    pub use crate::IntoSerializableValue;
    pub use crate::Operator;
    pub use crate::Order;

    //pub use crate::Value; // TODO: Do not let this conflict with crud_yew::Value

    pub use crate::DeleteResult;
    pub use crate::SaveResult;
    pub use crate::Saved;
}

#[derive(Debug)]
pub enum CrudError {
    UnknownColumnSpecified(String),
    UnableToParseValueAsColType(String, String),
    UnableToParseAsEntity(String, String),
    DbError(String),
    ReadOneFoundNone,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Order {
    #[serde(rename(
        serialize = "asc",
        deserialize = "asc",
        deserialize = "ascending",
        deserialize = "Asc"
    ))]
    Asc,
    #[serde(rename(
        serialize = "desc",
        deserialize = "desc",
        deserialize = "descending",
        deserialize = "Desc"
    ))]
    Desc,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionClause {
    pub column_name: String,
    pub operator: Operator,
    pub value: ConditionClauseValue,
}

// TODO: Drop in favor of "Value" type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionClauseValue {
    String(String),
    Bool(bool),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    //DateTime(chrono::NaiveDateTime), // TODO: Use UtcDateTime instead for consistency?
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionElement {
    Clause(ConditionClause),
    Condition(Box<Condition>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

//#[typetag::serde(tag = "type")]
pub trait IdFieldValue: Debug {
    fn into_condition_clause_value(&self) -> ConditionClauseValue;
}

pub trait IntoSerializableValue {
    type SerializableValue;

    fn into_serializable_value(&self) -> Self::SerializableValue;
}

pub trait IdField: Debug + Display {
    type Value: IdFieldValue + IntoSerializableValue;

    fn name(&self) -> &'static str;
    fn into_value(&self) -> Self::Value;
}

//#[typetag::serde(tag = "type")]
pub trait DynIdField: Debug + DynClone {
    fn dyn_name(&self) -> &'static str;
    fn into_dyn_value(&self) -> Box<dyn IdFieldValue>;
}
dyn_clone::clone_trait_object!(DynIdField);

/// Structs marked with this trait might be used as IDs in the crud system.
pub trait Id: Debug + Display + DynClone {
    /// This might be an enum, providing all possible fields.
    type Field: IdField;
    type FieldIter: Iterator<Item = Self::Field>;

    fn fields_iter(&self) -> Self::FieldIter;
    fn fields(&self) -> Vec<Box<dyn DynIdField>>;
}

pub trait IntoAllEqualCondition {
    fn into_all_equal_condition(self) -> Condition;
}

impl<I: Id> IntoAllEqualCondition for I {
    fn into_all_equal_condition(self) -> Condition {
        Condition::All(
            self.fields_iter()
                .map(|field| {
                    ConditionElement::Clause(ConditionClause {
                        column_name: String::from(field.name()),
                        operator: Operator::Equal,
                        value: field.into_value().into_condition_clause_value(),
                    })
                })
                .collect::<Vec<_>>(),
        )
    }
}

pub enum Value {
    String(String),
    I32(i32),
    I64(i64),
    U32(u32),
    F32(f32),
    Bool(bool),
    DateTime(chrono::NaiveDateTime),
}

impl Into<ConditionClauseValue> for Value {
    fn into(self) -> ConditionClauseValue {
        match self {
            Value::String(value) => ConditionClauseValue::String(value),
            Value::I32(value) => ConditionClauseValue::I32(value),
            Value::I64(value) => ConditionClauseValue::I64(value),
            Value::U32(value) => ConditionClauseValue::U32(value),
            Value::F32(value) => ConditionClauseValue::F32(value),
            Value::Bool(value) => ConditionClauseValue::Bool(value),
            //Value::DateTime(value) => ConditionClauseValue::DateTime(value), // TODO: implement
            Value::DateTime(_value) => panic!("Not implemented...."),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Saved<T> {
    pub entity: T,
    pub with_validation_errors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SaveResult<T> {
    Saved(Saved<T>),
    Aborted { reason: String },
    CriticalValidationErrors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeleteResult {
    Deleted(u64),
    Aborted { reason: String },
    CriticalValidationErrors,
}
