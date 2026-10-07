use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Comparison operator of a condition clause.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum OperatorV1 {
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

/// Compares the field `column_name` with `value`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ConditionClauseV1 {
    /// Name of the compared field.
    pub column_name: String,
    pub operator: OperatorV1,
    pub value: ConditionClauseValueV1,
}

/// Value a condition clause compares with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum ConditionClauseValueV1 {
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
}

/// An element of a condition: a clause or a nested condition. Serialized without a tag.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum ConditionElementV1 {
    Clause(ConditionClauseV1),

    #[schema(no_recursion)]
    Condition(Box<ConditionV1>),
}

/// A condition tree. `All` combines its elements with AND, `Any` with OR.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum ConditionV1 {
    All(Vec<ConditionElementV1>),

    Any(Vec<ConditionElementV1>),
}
