use serde::{Serialize, Deserialize};

pub enum CrudError {
    UnknownColumnSpecified(String),
    UnableToParseValueAsColType(String, String),
    UnableToParseAsEntity(String, String),
    DbError(String),
    ReadOneFoundNone,
    ValidationErrors(Vec<String>),
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
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

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub enum ConditionOperator {
    #[serde(rename(
        serialize = "and",
        deserialize = "and",
        deserialize = "AND",
        deserialize = "And"
    ))]
    And,
    #[serde(rename(
        serialize = "or",
        deserialize = "or",
        deserialize = "OR",
        deserialize = "Or"
    ))]
    Or,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ConditionClause {
    pub column_name: String,
    pub operator: Operator,
    pub value: ConditionClauseValue,
}

// TODO: Drop in favor of "Value" type
#[derive(Debug, Serialize, Deserialize)]
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
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionElement {
    Clause(ConditionClause),
    Operator(ConditionOperator),
}


pub enum Value {
    String(String),
    I32(i32),
    Bool(bool),
    DateTime(chrono::NaiveDateTime),
}
