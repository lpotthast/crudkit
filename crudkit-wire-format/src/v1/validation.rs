//! Validation results carried by response bodies.

use super::SerializableIdV1;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// How severe a violation is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SeverityV1 {
    /// The entity can still be saved.
    Major,
    /// The violation prevents saving or deleting the entity.
    Critical,
}

/// One violation found by a validator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ViolationV1 {
    pub severity: SeverityV1,
    /// Human-readable description of the violation.
    pub message: String,
}

/// The violations of one entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct EntityViolationsV1 {
    /// The entity.
    pub id: SerializableIdV1,
    pub violations: Vec<ViolationV1>,
}

/// Violations of a resource, reported for the part of it an operation validated.
///
/// `null` for `general` or `create` means that the operation did not validate that part, while an
/// empty list means that it found no violations there.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
pub struct PartialViolationsV1 {
    /// Violations that do not concern a single entity.
    pub general: Option<Vec<ViolationV1>>,
    /// Violations of a create model that does not have an ID yet.
    pub create: Option<Vec<ViolationV1>>,
    /// Violations of the validated entities.
    pub by_entity: Vec<EntityViolationsV1>,
}

/// All violations of a resource, reported by a validation of every one of its entities.
///
/// Receivers can replace everything they knew about the resource's violations with it.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
pub struct FullViolationsV1 {
    /// Violations that do not concern a single entity.
    pub general: Vec<ViolationV1>,
    /// Violations of the resource's entities. An entity without an entry has no violations.
    pub by_entity: Vec<EntityViolationsV1>,
}
