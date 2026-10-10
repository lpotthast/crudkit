//! Request bodies of the generated CRUD routes.
//!
//! Ordering criteria are keyed by field name, the same name condition clauses use.

use super::{ConditionV1, OrderV1, SerializableIdV1, WireFormatVersionV1};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for counting entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ReadCountV1 {
    /// Wire format version of the body.
    pub wire_format_version: WireFormatVersionV1,
    /// Filter condition.
    pub condition: Option<ConditionV1>,
}

/// Request body for reading one entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ReadOneV1 {
    /// Wire format version of the body.
    pub wire_format_version: WireFormatVersionV1,
    /// Number of entities to skip.
    pub skip: Option<u64>,
    /// Ordering criteria by field name, highest priority first.
    #[schema(value_type = Option<Object>, example = json!({"id": "asc"}))]
    pub order_by: Option<IndexMap<String, OrderV1>>,
    /// Filter condition.
    pub condition: Option<ConditionV1>,
}

/// Request body for reading many entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ReadManyV1 {
    /// Wire format version of the body.
    pub wire_format_version: WireFormatVersionV1,
    /// Maximum number of entities to return.
    pub limit: Option<u64>,
    /// Number of entities to skip.
    pub skip: Option<u64>,
    /// Ordering criteria by field name, highest priority first.
    #[schema(value_type = Option<Object>, example = json!({"id": "asc"}))]
    pub order_by: Option<IndexMap<String, OrderV1>>,
    /// Filter condition.
    pub condition: Option<ConditionV1>,
}

/// Request body for creating a single entity of the application's create model `T`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CreateOneV1<T> {
    /// Wire format version of the body.
    pub wire_format_version: WireFormatVersionV1,
    /// The entity data to create.
    pub entity: T,
}

/// Request body for updating a single entity with the application's update model `T`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct UpdateOneV1<T> {
    /// Wire format version of the body.
    pub wire_format_version: WireFormatVersionV1,
    /// Condition to identify the entity to update.
    pub condition: Option<ConditionV1>,
    /// The update data.
    pub entity: T,
}

/// Request body for deleting by ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct DeleteByIdV1 {
    /// Wire format version of the body.
    pub wire_format_version: WireFormatVersionV1,
    /// The ID of the entity to delete.
    pub id: SerializableIdV1,
    /// Scope the entity must also match. An entity outside of this scope is not found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<ConditionV1>,
}

/// Request body for deleting one entity by condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct DeleteOneV1 {
    /// Wire format version of the body.
    pub wire_format_version: WireFormatVersionV1,
    /// Number of entities to skip.
    pub skip: Option<u64>,
    /// Ordering criteria by field name, highest priority first.
    #[schema(value_type = Option<Object>, example = json!({"id": "asc"}))]
    pub order_by: Option<IndexMap<String, OrderV1>>,
    /// Filter condition.
    pub condition: Option<ConditionV1>,
}

/// Request body for deleting many entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct DeleteManyV1 {
    /// Wire format version of the body.
    pub wire_format_version: WireFormatVersionV1,
    /// Filter condition.
    pub condition: Option<ConditionV1>,
}
