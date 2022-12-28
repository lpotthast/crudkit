#![forbid(unsafe_code)]

use std::fmt::Debug;

use chrono_utc_date_time::UtcDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
    pub use crate::id::SerializableId;
    pub use crate::Order;
    //pub use crate::Value; // TODO: Do not let this conflict with crud_yew::Value

    pub use crate::DeleteResult;
    pub use crate::SaveResult;
    pub use crate::Saved;
}

pub type UuidV4 = uuid::Uuid;
pub type UuidV7 = uuid::Uuid;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, ToSchema, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, ToSchema, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Json(serde_json::Value),
    UuidV4(uuid::Uuid),
    UuidV7(uuid::Uuid),
    Ulid(ulid::Ulid),
    I32(i32),
    I64(i64),
    U32(u32),
    F32(f32),
    Bool(bool),
    DateTime(UtcDateTime),
}

/// "ID-able" values. Values which might be part of an entities ID. All variants must implement `Eq` for proper comparability!
/// This constraint excludes options like floats as parts of primary keys.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ToSchema, Serialize, Deserialize)]
pub enum IdValue {
    String(String),
    UuidV4(uuid::Uuid),
    UuidV7(uuid::Uuid),
    Ulid(ulid::Ulid),
    I32(i32),
    I64(i64),
    U32(u32),
    Bool(bool),
    DateTime(UtcDateTime),
}

impl id::IntoSerializableValue for Value {
    type SerializableValue = Self;

    fn into_serializable_value(&self) -> Self::SerializableValue {
        self.clone()
    }
}

impl id::IntoSerializableValue for IdValue {
    type SerializableValue = Self;

    fn into_serializable_value(&self) -> Self::SerializableValue {
        self.clone()
    }
}

impl id::IdFieldValue for Value {
    fn into_condition_clause_value(&self) -> condition::ConditionClauseValue {
        // Note: This requires clone, because we take &self. We take &self, so that the trait remains dynamically usable.
        self.clone().into()
    }
}

impl id::IdFieldValue for IdValue {
    fn into_condition_clause_value(&self) -> condition::ConditionClauseValue {
        // Note: This requires clone, because we take &self. We take &self, so that the trait remains dynamically usable.
        self.clone().into()
    }
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct Saved<T> {
    pub entity: T,
    pub with_validation_errors: bool,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub enum SaveResult<T> {
    Saved(Saved<T>),
    Aborted { reason: String },
    CriticalValidationErrors,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub enum DeleteResult {
    Deleted(u64),
    Aborted { reason: String },
    CriticalValidationErrors,
}
