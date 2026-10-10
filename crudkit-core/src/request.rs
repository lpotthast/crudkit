//! Internal representation of the requests of the generated CRUD routes.
//!
//! These types are not serialized. Their wire form lives in `crudkit_wire_format`; the conversions
//! are implemented in [`crate::wire_v1`]. Types that order entities are generic over the field enum
//! `F` of the model being ordered. Model-carrying types are generic over the model `T`.

use crate::Order;
use crate::condition::Condition;
use crate::id::SerializableId;
use indexmap::IndexMap;

/// Request for counting entities.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadCount {
    /// Filter condition.
    pub condition: Option<Condition>,
}

/// Request for reading one entity.
#[derive(Debug, Clone)]
pub struct ReadOne<F> {
    /// Number of entities to skip.
    pub skip: Option<u64>,
    /// Ordering criteria, highest priority first.
    pub order_by: Option<IndexMap<F, Order>>,
    /// Filter condition.
    pub condition: Option<Condition>,
}

/// Request for reading many entities.
#[derive(Debug, Clone)]
pub struct ReadMany<F> {
    /// Maximum number of entities to return.
    pub limit: Option<u64>,
    /// Number of entities to skip.
    pub skip: Option<u64>,
    /// Ordering criteria, highest priority first.
    pub order_by: Option<IndexMap<F, Order>>,
    /// Filter condition.
    pub condition: Option<Condition>,
}

/// Request for creating a single entity.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateOne<T> {
    /// The entity data to create.
    pub entity: T,
}

/// Request for updating a single entity.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateOne<T> {
    /// Condition to identify the entity to update.
    pub condition: Option<Condition>,
    /// The update data.
    pub entity: T,
}

/// Request for deleting by ID.
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteById {
    /// The ID of the entity to delete.
    pub id: SerializableId,
    /// Scope the entity must also match, e.g. the base and parent conditions of a client view. An
    /// entity outside of this scope is not found.
    pub condition: Option<Condition>,
}

/// Request for deleting one entity by condition.
#[derive(Debug, Clone)]
pub struct DeleteOne<F> {
    /// Number of entities to skip.
    pub skip: Option<u64>,
    /// Ordering criteria, highest priority first.
    pub order_by: Option<IndexMap<F, Order>>,
    /// Filter condition.
    pub condition: Option<Condition>,
}

/// Request for deleting many entities.
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteMany {
    /// Filter condition.
    pub condition: Option<Condition>,
}
