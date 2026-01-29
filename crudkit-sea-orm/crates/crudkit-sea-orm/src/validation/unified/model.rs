//! SeaORM entity model for validation results.
//!
//! This model stores validation results with the entity ID serialized as JSON,
//! which allows for both simple and composite primary keys.

use crate::validation::PersistedViolationSeverity;
use crudkit_core::validation::validator::ValidatorInfo;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// The validation result entity.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "CrudkitValidation")]
pub struct Model {
    /// Auto-increment primary key.
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,

    /// The resource type (e.g., "article", "comment").
    pub resource_name: String,

    /// The entity ID serialized as JSON.
    /// For simple IDs: `[["id",{"I64":123}]]`
    /// For composite keys: `[["user_id",{"I64":1}],["event_id",{"I64":2}]]`
    #[sea_orm(column_type = "JsonBinary")]
    pub entity_id: serde_json::Value,

    /// The name of the validator that produced this result.
    pub validator_name: String,

    /// The version of the validator.
    pub validator_version: i64,

    /// The severity of the violation (e.g., "MAJOR", "CRITICAL").
    pub violation_severity: PersistedViolationSeverity,

    /// The violation message.
    #[sea_orm(column_type = "Text")]
    pub violation_message: String,

    /// Timestamp when this record was created.
    pub created_at: time::OffsetDateTime,
}

impl Model {
    pub fn to_validator_info(&self) -> ValidatorInfo<'_> {
        ValidatorInfo::new(&self.validator_name, self.validator_version as u32)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
