#![forbid(unsafe_code)]

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

pub mod condition;
pub mod error;
pub mod id;
pub mod validation;
pub mod ws_messages;

pub mod prelude {
    pub use crate::condition::merge_conditions;
    pub use crate::condition::Condition;
    pub use crate::condition::ConditionClause;
    pub use crate::condition::ConditionClauseValue;
    pub use crate::condition::ConditionElement;
    pub use crate::condition::IntoAllEqualCondition;
    pub use crate::condition::Operator;
    pub use crate::error::CrudError;
    pub use crate::id::DynIdField;
    pub use crate::id::Id;
    pub use crate::id::IdField;
    pub use crate::id::IdFieldValue;
    pub use crate::id::IntoSerializableValue;
    pub use crate::Order;

    //pub use crate::Value; // TODO: Do not let this conflict with crud_yew::Value

    pub use crate::DeleteResult;
    pub use crate::SaveResult;
    pub use crate::Saved;
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

#[derive(Debug)]
pub enum Value {
    String(String),
    I32(i32),
    I64(i64),
    U32(u32),
    F32(f32),
    Bool(bool),
    DateTime(chrono::NaiveDateTime),
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
