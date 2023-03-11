#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
    I32(i32),
    I32Vec(Vec<i32>),
    I64(i64),
    U32(u32),
    F32(f32),
    Bool(bool),
    PrimitiveDateTime(time::PrimitiveDateTime),
    OffsetDateTime(time::OffsetDateTime),
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
