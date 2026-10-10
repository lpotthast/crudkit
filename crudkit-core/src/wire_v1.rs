//! Conversions between the internal types and version 1 of the wire format.
//!
//! The mappings are exhaustive in both directions, so adding a variant on either side does not
//! compile until the other side and this module agree. Converting a request that orders entities
//! from the wire resolves each field name with a caller-supplied lookup, e.g. the backend's
//! `FieldLookup::from_name`, and fails on unknown names.

use crate::collaboration::{CollabMessage, EntityCreated, EntityDeleted, EntityUpdated};
use crate::condition::{
    Condition, ConditionClause, ConditionClauseValue, ConditionElement, Operator,
};
use crate::id::{IdValue, SerializableId, SerializableIdEntry};
use crate::request::{
    CreateOne, DeleteById, DeleteMany, DeleteOne, ReadCount, ReadMany, ReadOne, UpdateOne,
};
use crate::resource::ResourceName;
use crate::validation::violation::{Violation, Violations};
use crate::validation::{
    FullSerializableAggregateViolations, PartialSerializableAggregateViolations,
};
use crate::{Deleted, DeletedMany, Named, Order, Saved};
use crudkit_wire_format::v1::{
    CollabEventV1, CollabMessageV1, FullViolationsV1, ResourceFullViolationsV1,
    ResourcePartialViolationsV1,
};
use crudkit_wire_format::v1::{
    ConditionClauseV1, ConditionClauseValueV1, ConditionElementV1, ConditionV1, CreateOneV1,
    DeleteByIdV1, DeleteManyV1, DeleteOneV1, IdValueV1, OperatorV1, OrderV1, ReadCountV1,
    ReadManyV1, ReadOneV1, SerializableIdEntryV1, SerializableIdV1, UpdateOneV1,
    WireFormatVersionV1,
};
use crudkit_wire_format::v1::{
    DeletedManyV1, DeletedV1, EntityViolationsV1, FailedDeletionV1, PartialViolationsV1, SavedV1,
    SeverityV1, ViolationV1,
};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::hash::Hash;

/// A request named a field that the addressed model does not have.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown field '{field}'")]
pub struct UnknownFieldError {
    /// The unknown field name.
    pub field: String,
}

impl From<Order> for OrderV1 {
    fn from(order: Order) -> Self {
        match order {
            Order::Asc => Self::Asc,
            Order::Desc => Self::Desc,
        }
    }
}

impl From<OrderV1> for Order {
    fn from(order: OrderV1) -> Self {
        match order {
            OrderV1::Asc => Self::Asc,
            OrderV1::Desc => Self::Desc,
        }
    }
}

impl From<Operator> for OperatorV1 {
    fn from(operator: Operator) -> Self {
        match operator {
            Operator::Equal => Self::Equal,
            Operator::NotEqual => Self::NotEqual,
            Operator::Less => Self::Less,
            Operator::LessOrEqual => Self::LessOrEqual,
            Operator::Greater => Self::Greater,
            Operator::GreaterOrEqual => Self::GreaterOrEqual,
            Operator::IsIn => Self::IsIn,
        }
    }
}

impl From<OperatorV1> for Operator {
    fn from(operator: OperatorV1) -> Self {
        match operator {
            OperatorV1::Equal => Self::Equal,
            OperatorV1::NotEqual => Self::NotEqual,
            OperatorV1::Less => Self::Less,
            OperatorV1::LessOrEqual => Self::LessOrEqual,
            OperatorV1::Greater => Self::Greater,
            OperatorV1::GreaterOrEqual => Self::GreaterOrEqual,
            OperatorV1::IsIn => Self::IsIn,
        }
    }
}

impl From<ConditionClauseValue> for ConditionClauseValueV1 {
    fn from(value: ConditionClauseValue) -> Self {
        match value {
            ConditionClauseValue::Bool(value) => Self::Bool(value),
            ConditionClauseValue::U8(value) => Self::U8(value),
            ConditionClauseValue::U16(value) => Self::U16(value),
            ConditionClauseValue::U32(value) => Self::U32(value),
            ConditionClauseValue::U64(value) => Self::U64(value),
            ConditionClauseValue::U128(value) => Self::U128(value),
            ConditionClauseValue::I8(value) => Self::I8(value),
            ConditionClauseValue::I16(value) => Self::I16(value),
            ConditionClauseValue::I32(value) => Self::I32(value),
            ConditionClauseValue::I64(value) => Self::I64(value),
            ConditionClauseValue::I128(value) => Self::I128(value),
            ConditionClauseValue::F32(value) => Self::F32(value),
            ConditionClauseValue::F64(value) => Self::F64(value),
            ConditionClauseValue::String(value) => Self::String(value),
            ConditionClauseValue::Json(value) => Self::Json(value),
            ConditionClauseValue::Uuid(value) => Self::Uuid(value),
            ConditionClauseValue::U8Vec(value) => Self::U8Vec(value),
            ConditionClauseValue::I32Vec(value) => Self::I32Vec(value),
            ConditionClauseValue::I64Vec(value) => Self::I64Vec(value),
        }
    }
}

impl From<ConditionClauseValueV1> for ConditionClauseValue {
    fn from(value: ConditionClauseValueV1) -> Self {
        match value {
            ConditionClauseValueV1::Bool(value) => Self::Bool(value),
            ConditionClauseValueV1::U8(value) => Self::U8(value),
            ConditionClauseValueV1::U16(value) => Self::U16(value),
            ConditionClauseValueV1::U32(value) => Self::U32(value),
            ConditionClauseValueV1::U64(value) => Self::U64(value),
            ConditionClauseValueV1::U128(value) => Self::U128(value),
            ConditionClauseValueV1::I8(value) => Self::I8(value),
            ConditionClauseValueV1::I16(value) => Self::I16(value),
            ConditionClauseValueV1::I32(value) => Self::I32(value),
            ConditionClauseValueV1::I64(value) => Self::I64(value),
            ConditionClauseValueV1::I128(value) => Self::I128(value),
            ConditionClauseValueV1::F32(value) => Self::F32(value),
            ConditionClauseValueV1::F64(value) => Self::F64(value),
            ConditionClauseValueV1::String(value) => Self::String(value),
            ConditionClauseValueV1::Json(value) => Self::Json(value),
            ConditionClauseValueV1::Uuid(value) => Self::Uuid(value),
            ConditionClauseValueV1::U8Vec(value) => Self::U8Vec(value),
            ConditionClauseValueV1::I32Vec(value) => Self::I32Vec(value),
            ConditionClauseValueV1::I64Vec(value) => Self::I64Vec(value),
        }
    }
}

impl From<ConditionClause> for ConditionClauseV1 {
    fn from(clause: ConditionClause) -> Self {
        Self {
            column_name: clause.column_name,
            operator: clause.operator.into(),
            value: clause.value.into(),
        }
    }
}

impl From<ConditionClauseV1> for ConditionClause {
    fn from(clause: ConditionClauseV1) -> Self {
        Self {
            column_name: clause.column_name,
            operator: clause.operator.into(),
            value: clause.value.into(),
        }
    }
}

impl From<ConditionElement> for ConditionElementV1 {
    fn from(element: ConditionElement) -> Self {
        match element {
            ConditionElement::Clause(clause) => Self::Clause(clause.into()),
            ConditionElement::Condition(condition) => {
                Self::Condition(Box::new((*condition).into()))
            }
        }
    }
}

impl From<ConditionElementV1> for ConditionElement {
    fn from(element: ConditionElementV1) -> Self {
        match element {
            ConditionElementV1::Clause(clause) => Self::Clause(clause.into()),
            ConditionElementV1::Condition(condition) => {
                Self::Condition(Box::new((*condition).into()))
            }
        }
    }
}

impl From<Condition> for ConditionV1 {
    fn from(condition: Condition) -> Self {
        match condition {
            Condition::All(elements) => Self::All(elements.into_iter().map(Into::into).collect()),
            Condition::Any(elements) => Self::Any(elements.into_iter().map(Into::into).collect()),
        }
    }
}

impl From<ConditionV1> for Condition {
    fn from(condition: ConditionV1) -> Self {
        match condition {
            ConditionV1::All(elements) => Self::All(elements.into_iter().map(Into::into).collect()),
            ConditionV1::Any(elements) => Self::Any(elements.into_iter().map(Into::into).collect()),
        }
    }
}

impl From<IdValue> for IdValueV1 {
    fn from(value: IdValue) -> Self {
        match value {
            IdValue::I8(value) => Self::I8(value),
            IdValue::I16(value) => Self::I16(value),
            IdValue::I32(value) => Self::I32(value),
            IdValue::I64(value) => Self::I64(value),
            IdValue::I128(value) => Self::I128(value),
            IdValue::U8(value) => Self::U8(value),
            IdValue::U16(value) => Self::U16(value),
            IdValue::U32(value) => Self::U32(value),
            IdValue::U64(value) => Self::U64(value),
            IdValue::U128(value) => Self::U128(value),
            IdValue::Bool(value) => Self::Bool(value),
            IdValue::String(value) => Self::String(value),
            IdValue::Uuid(value) => Self::Uuid(value),
            IdValue::PrimitiveDateTime(value) => Self::PrimitiveDateTime(value),
            IdValue::OffsetDateTime(value) => Self::OffsetDateTime(value),
        }
    }
}

impl From<IdValueV1> for IdValue {
    fn from(value: IdValueV1) -> Self {
        match value {
            IdValueV1::I8(value) => Self::I8(value),
            IdValueV1::I16(value) => Self::I16(value),
            IdValueV1::I32(value) => Self::I32(value),
            IdValueV1::I64(value) => Self::I64(value),
            IdValueV1::I128(value) => Self::I128(value),
            IdValueV1::U8(value) => Self::U8(value),
            IdValueV1::U16(value) => Self::U16(value),
            IdValueV1::U32(value) => Self::U32(value),
            IdValueV1::U64(value) => Self::U64(value),
            IdValueV1::U128(value) => Self::U128(value),
            IdValueV1::Bool(value) => Self::Bool(value),
            IdValueV1::String(value) => Self::String(value),
            IdValueV1::Uuid(value) => Self::Uuid(value),
            IdValueV1::PrimitiveDateTime(value) => Self::PrimitiveDateTime(value),
            IdValueV1::OffsetDateTime(value) => Self::OffsetDateTime(value),
        }
    }
}

impl From<SerializableIdEntry> for SerializableIdEntryV1 {
    fn from(entry: SerializableIdEntry) -> Self {
        Self {
            field_name: entry.field_name,
            value: entry.value.into(),
        }
    }
}

impl From<SerializableIdEntryV1> for SerializableIdEntry {
    fn from(entry: SerializableIdEntryV1) -> Self {
        Self {
            field_name: entry.field_name,
            value: entry.value.into(),
        }
    }
}

impl From<SerializableId> for SerializableIdV1 {
    fn from(id: SerializableId) -> Self {
        Self(id.0.into_iter().map(Into::into).collect())
    }
}

impl From<SerializableIdV1> for SerializableId {
    fn from(id: SerializableIdV1) -> Self {
        Self(id.0.into_iter().map(Into::into).collect())
    }
}

fn order_by_to_v1<F: Named>(
    order_by: Option<IndexMap<F, Order>>,
) -> Option<IndexMap<String, OrderV1>> {
    order_by.map(|order_by| {
        order_by
            .into_iter()
            .map(|(field, order)| (field.name().into_owned(), order.into()))
            .collect()
    })
}

fn order_by_from_v1<F: Hash + Eq>(
    order_by: Option<IndexMap<String, OrderV1>>,
    lookup: impl Fn(&str) -> Option<F>,
) -> Result<Option<IndexMap<F, Order>>, UnknownFieldError> {
    order_by
        .map(|order_by| {
            order_by
                .into_iter()
                .map(|(name, order)| match lookup(&name) {
                    Some(field) => Ok((field, order.into())),
                    None => Err(UnknownFieldError { field: name }),
                })
                .collect()
        })
        .transpose()
}

impl From<ReadCount> for ReadCountV1 {
    fn from(request: ReadCount) -> Self {
        Self {
            wire_format_version: WireFormatVersionV1,
            condition: request.condition.map(Into::into),
        }
    }
}

impl From<ReadCountV1> for ReadCount {
    fn from(request: ReadCountV1) -> Self {
        Self {
            condition: request.condition.map(Into::into),
        }
    }
}

impl<F: Named> From<ReadOne<F>> for ReadOneV1 {
    fn from(request: ReadOne<F>) -> Self {
        Self {
            wire_format_version: WireFormatVersionV1,
            skip: request.skip,
            order_by: order_by_to_v1(request.order_by),
            condition: request.condition.map(Into::into),
        }
    }
}

impl<F: Hash + Eq> ReadOne<F> {
    /// Converts a wire request, resolving ordering field names with `lookup`.
    ///
    /// # Errors
    ///
    /// Returns an error if `lookup` does not know an ordering field name.
    pub fn try_from_v1(
        request: ReadOneV1,
        lookup: impl Fn(&str) -> Option<F>,
    ) -> Result<Self, UnknownFieldError> {
        Ok(Self {
            skip: request.skip,
            order_by: order_by_from_v1(request.order_by, lookup)?,
            condition: request.condition.map(Into::into),
        })
    }
}

impl<F: Named> From<ReadMany<F>> for ReadManyV1 {
    fn from(request: ReadMany<F>) -> Self {
        Self {
            wire_format_version: WireFormatVersionV1,
            limit: request.limit,
            skip: request.skip,
            order_by: order_by_to_v1(request.order_by),
            condition: request.condition.map(Into::into),
        }
    }
}

impl<F: Hash + Eq> ReadMany<F> {
    /// Converts a wire request, resolving ordering field names with `lookup`.
    ///
    /// # Errors
    ///
    /// Returns an error if `lookup` does not know an ordering field name.
    pub fn try_from_v1(
        request: ReadManyV1,
        lookup: impl Fn(&str) -> Option<F>,
    ) -> Result<Self, UnknownFieldError> {
        Ok(Self {
            limit: request.limit,
            skip: request.skip,
            order_by: order_by_from_v1(request.order_by, lookup)?,
            condition: request.condition.map(Into::into),
        })
    }
}

impl<T> From<CreateOne<T>> for CreateOneV1<T> {
    fn from(request: CreateOne<T>) -> Self {
        Self {
            wire_format_version: WireFormatVersionV1,
            entity: request.entity,
        }
    }
}

impl<T> From<CreateOneV1<T>> for CreateOne<T> {
    fn from(request: CreateOneV1<T>) -> Self {
        Self {
            entity: request.entity,
        }
    }
}

impl<T> From<UpdateOne<T>> for UpdateOneV1<T> {
    fn from(request: UpdateOne<T>) -> Self {
        Self {
            wire_format_version: WireFormatVersionV1,
            condition: request.condition.map(Into::into),
            entity: request.entity,
        }
    }
}

impl<T> From<UpdateOneV1<T>> for UpdateOne<T> {
    fn from(request: UpdateOneV1<T>) -> Self {
        Self {
            condition: request.condition.map(Into::into),
            entity: request.entity,
        }
    }
}

impl From<DeleteById> for DeleteByIdV1 {
    fn from(request: DeleteById) -> Self {
        Self {
            wire_format_version: WireFormatVersionV1,
            id: request.id.into(),
            condition: request.condition.map(Into::into),
        }
    }
}

impl From<DeleteByIdV1> for DeleteById {
    fn from(request: DeleteByIdV1) -> Self {
        Self {
            id: request.id.into(),
            condition: request.condition.map(Into::into),
        }
    }
}

impl<F: Named> From<DeleteOne<F>> for DeleteOneV1 {
    fn from(request: DeleteOne<F>) -> Self {
        Self {
            wire_format_version: WireFormatVersionV1,
            skip: request.skip,
            order_by: order_by_to_v1(request.order_by),
            condition: request.condition.map(Into::into),
        }
    }
}

impl<F: Hash + Eq> DeleteOne<F> {
    /// Converts a wire request, resolving ordering field names with `lookup`.
    ///
    /// # Errors
    ///
    /// Returns an error if `lookup` does not know an ordering field name.
    pub fn try_from_v1(
        request: DeleteOneV1,
        lookup: impl Fn(&str) -> Option<F>,
    ) -> Result<Self, UnknownFieldError> {
        Ok(Self {
            skip: request.skip,
            order_by: order_by_from_v1(request.order_by, lookup)?,
            condition: request.condition.map(Into::into),
        })
    }
}

impl From<DeleteMany> for DeleteManyV1 {
    fn from(request: DeleteMany) -> Self {
        Self {
            wire_format_version: WireFormatVersionV1,
            condition: request.condition.map(Into::into),
        }
    }
}

impl From<DeleteManyV1> for DeleteMany {
    fn from(request: DeleteManyV1) -> Self {
        Self {
            condition: request.condition.map(Into::into),
        }
    }
}

impl From<Violation> for ViolationV1 {
    fn from(violation: Violation) -> Self {
        match violation {
            Violation::Major(message) => Self {
                severity: SeverityV1::Major,
                message,
            },
            Violation::Critical(message) => Self {
                severity: SeverityV1::Critical,
                message,
            },
        }
    }
}

impl From<ViolationV1> for Violation {
    fn from(violation: ViolationV1) -> Self {
        match violation.severity {
            SeverityV1::Major => Self::Major(violation.message),
            SeverityV1::Critical => Self::Critical(violation.message),
        }
    }
}

fn violations_to_v1(violations: Violations) -> Vec<ViolationV1> {
    violations.into_iter().map(Into::into).collect()
}

fn violations_from_v1(violations: Vec<ViolationV1>) -> Violations {
    Violations {
        violations: violations.into_iter().map(Into::into).collect(),
    }
}

impl From<PartialSerializableAggregateViolations> for PartialViolationsV1 {
    fn from(violations: PartialSerializableAggregateViolations) -> Self {
        Self {
            general: violations.general.map(violations_to_v1),
            create: violations.create.map(violations_to_v1),
            by_entity: violations
                .by_entity
                .into_iter()
                .map(|(id, violations)| EntityViolationsV1 {
                    id: id.into(),
                    violations: violations_to_v1(violations),
                })
                .collect(),
        }
    }
}

impl From<PartialViolationsV1> for PartialSerializableAggregateViolations {
    fn from(violations: PartialViolationsV1) -> Self {
        Self {
            general: violations.general.map(violations_from_v1),
            create: violations.create.map(violations_from_v1),
            by_entity: violations
                .by_entity
                .into_iter()
                .map(|entity| (entity.id.into(), violations_from_v1(entity.violations)))
                .collect(),
        }
    }
}

impl<T> From<Saved<T>> for SavedV1<T> {
    fn from(saved: Saved<T>) -> Self {
        Self {
            wire_format_version: WireFormatVersionV1,
            entity: saved.entity,
            violations: saved.violations.into(),
        }
    }
}

impl<T> From<SavedV1<T>> for Saved<T> {
    fn from(saved: SavedV1<T>) -> Self {
        Self {
            entity: saved.entity,
            violations: saved.violations.into(),
        }
    }
}

impl From<Deleted> for DeletedV1 {
    fn from(deleted: Deleted) -> Self {
        Self {
            wire_format_version: WireFormatVersionV1,
            entities_affected: deleted.entities_affected,
        }
    }
}

impl From<DeletedV1> for Deleted {
    fn from(deleted: DeletedV1) -> Self {
        Self {
            entities_affected: deleted.entities_affected,
        }
    }
}

fn failed_deletions_to_v1(failures: Vec<(SerializableId, String)>) -> Vec<FailedDeletionV1> {
    failures
        .into_iter()
        .map(|(id, reason)| FailedDeletionV1 {
            id: id.into(),
            reason,
        })
        .collect()
}

fn failed_deletions_from_v1(failures: Vec<FailedDeletionV1>) -> Vec<(SerializableId, String)> {
    failures
        .into_iter()
        .map(|failure| (failure.id.into(), failure.reason))
        .collect()
}

impl From<DeletedMany> for DeletedManyV1 {
    fn from(deleted: DeletedMany) -> Self {
        Self {
            wire_format_version: WireFormatVersionV1,
            deleted_count: deleted.deleted_count,
            deleted_ids: deleted.deleted_ids.into_iter().map(Into::into).collect(),
            aborted: failed_deletions_to_v1(deleted.aborted),
            validation_failed: deleted
                .validation_failed
                .into_iter()
                .map(Into::into)
                .collect(),
            errors: failed_deletions_to_v1(deleted.errors),
        }
    }
}

impl From<DeletedManyV1> for DeletedMany {
    fn from(deleted: DeletedManyV1) -> Self {
        Self {
            deleted_count: deleted.deleted_count,
            deleted_ids: deleted.deleted_ids.into_iter().map(Into::into).collect(),
            aborted: failed_deletions_from_v1(deleted.aborted),
            validation_failed: deleted
                .validation_failed
                .into_iter()
                .map(Into::into)
                .collect(),
            errors: failed_deletions_from_v1(deleted.errors),
        }
    }
}

fn full_violations_to_v1(violations: FullSerializableAggregateViolations) -> FullViolationsV1 {
    let mut by_entity = violations.by_entity.into_iter().collect::<Vec<_>>();
    by_entity.sort_by(|(a, _), (b, _)| a.cmp(b));
    FullViolationsV1 {
        general: violations_to_v1(violations.general),
        by_entity: by_entity
            .into_iter()
            .map(|(id, violations)| EntityViolationsV1 {
                id: id.into(),
                violations: violations_to_v1(violations),
            })
            .collect(),
    }
}

fn full_violations_from_v1(violations: FullViolationsV1) -> FullSerializableAggregateViolations {
    FullSerializableAggregateViolations {
        general: violations_from_v1(violations.general),
        by_entity: violations
            .by_entity
            .into_iter()
            .map(|entity| (entity.id.into(), violations_from_v1(entity.violations)))
            .collect(),
    }
}

/// Lists `map` sorted by resource name, so that the serialized message is deterministic.
fn sorted_by_resource<V>(map: HashMap<ResourceName, V>) -> Vec<(ResourceName, V)> {
    let mut entries = map.into_iter().collect::<Vec<_>>();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));
    entries
}

impl From<CollabMessage> for CollabMessageV1 {
    fn from(message: CollabMessage) -> Self {
        let event = match message {
            CollabMessage::EntityCreated(EntityCreated {
                resource_name,
                entity_id,
                with_validation_errors,
            }) => CollabEventV1::EntityCreated {
                resource_name,
                entity_id: entity_id.into(),
                with_validation_errors,
            },
            CollabMessage::EntityUpdated(EntityUpdated {
                resource_name,
                entity_id,
                with_validation_errors,
            }) => CollabEventV1::EntityUpdated {
                resource_name,
                entity_id: entity_id.into(),
                with_validation_errors,
            },
            CollabMessage::EntityDeleted(EntityDeleted {
                resource_name,
                entity_id,
            }) => CollabEventV1::EntityDeleted {
                resource_name,
                entity_id: entity_id.into(),
            },
            CollabMessage::PartialValidationResult(validations) => {
                CollabEventV1::PartialValidationResult {
                    resources: sorted_by_resource(validations)
                        .into_iter()
                        .map(|(resource_name, violations)| ResourcePartialViolationsV1 {
                            resource_name: resource_name.as_str().to_owned(),
                            violations: violations.into(),
                        })
                        .collect(),
                }
            }
            CollabMessage::FullValidationResult(validations) => {
                CollabEventV1::FullValidationResult {
                    resources: sorted_by_resource(validations)
                        .into_iter()
                        .map(|(resource_name, violations)| ResourceFullViolationsV1 {
                            resource_name: resource_name.as_str().to_owned(),
                            violations: full_violations_to_v1(violations),
                        })
                        .collect(),
                }
            }
        };
        Self {
            wire_format_version: WireFormatVersionV1,
            event,
        }
    }
}

impl From<CollabMessageV1> for CollabMessage {
    fn from(message: CollabMessageV1) -> Self {
        match message.event {
            CollabEventV1::EntityCreated {
                resource_name,
                entity_id,
                with_validation_errors,
            } => Self::EntityCreated(EntityCreated {
                resource_name,
                entity_id: entity_id.into(),
                with_validation_errors,
            }),
            CollabEventV1::EntityUpdated {
                resource_name,
                entity_id,
                with_validation_errors,
            } => Self::EntityUpdated(EntityUpdated {
                resource_name,
                entity_id: entity_id.into(),
                with_validation_errors,
            }),
            CollabEventV1::EntityDeleted {
                resource_name,
                entity_id,
            } => Self::EntityDeleted(EntityDeleted {
                resource_name,
                entity_id: entity_id.into(),
            }),
            CollabEventV1::PartialValidationResult { resources } => Self::PartialValidationResult(
                resources
                    .into_iter()
                    .map(|resource| {
                        (
                            ResourceName::new(resource.resource_name),
                            resource.violations.into(),
                        )
                    })
                    .collect(),
            ),
            CollabEventV1::FullValidationResult { resources } => Self::FullValidationResult(
                resources
                    .into_iter()
                    .map(|resource| {
                        (
                            ResourceName::new(resource.resource_name),
                            full_violations_from_v1(resource.violations),
                        )
                    })
                    .collect(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum Field {
        FirstName,
    }

    impl Named for Field {
        fn name(&self) -> Cow<'static, str> {
            Cow::Borrowed("first_name")
        }
    }

    fn lookup(name: &str) -> Option<Field> {
        (name == "first_name").then_some(Field::FirstName)
    }

    #[test]
    fn ordering_round_trips_through_field_names() {
        let condition = Condition::Any(vec![ConditionElement::Clause(ConditionClause {
            column_name: "first_name".to_owned(),
            operator: Operator::IsIn,
            value: ConditionClauseValue::I64Vec(vec![1, 2]),
        })]);
        let request = ReadMany {
            limit: Some(5),
            skip: None,
            order_by: Some(IndexMap::from([(Field::FirstName, Order::Desc)])),
            condition: Some(condition.clone()),
        };

        let wire = ReadManyV1::from(request);
        assert_eq!(
            wire.order_by,
            Some(IndexMap::from([("first_name".to_owned(), OrderV1::Desc)]))
        );
        let restored = ReadMany::try_from_v1(wire, lookup).expect("known field");
        assert_eq!(
            restored.order_by,
            Some(IndexMap::from([(Field::FirstName, Order::Desc)]))
        );
        assert_eq!(restored.condition, Some(condition));
    }

    #[test]
    fn unknown_ordering_fields_are_rejected() {
        let wire = ReadOneV1 {
            wire_format_version: WireFormatVersionV1,
            skip: None,
            order_by: Some(IndexMap::from([("hat".to_owned(), OrderV1::Asc)])),
            condition: None,
        };

        assert_eq!(
            ReadOne::try_from_v1(wire, lookup).map(|request| request.skip),
            Err(UnknownFieldError {
                field: "hat".to_owned()
            })
        );
    }

    #[test]
    fn saved_and_deleted_results_round_trip() {
        let id = SerializableId(vec![SerializableIdEntry {
            field_name: "id".to_owned(),
            value: IdValue::I64(7),
        }]);
        let saved = Saved {
            entity: "Ginny",
            violations: PartialSerializableAggregateViolations {
                general: Some(Violations::empty()),
                create: None,
                by_entity: vec![(
                    id.clone(),
                    Violations {
                        violations: vec![Violation::major("Minor"), Violation::critical("Major")],
                    },
                )],
            },
        };
        let restored = Saved::from(SavedV1::from(saved.clone()));
        assert_eq!(restored.entity, saved.entity);
        assert_eq!(restored.violations, saved.violations);

        let deleted = DeletedMany {
            deleted_count: 1,
            deleted_ids: vec![id.clone()],
            aborted: vec![(id.clone(), "Captain".to_owned())],
            validation_failed: vec![id.clone()],
            errors: vec![(id, "Repository error.".to_owned())],
        };
        let restored = DeletedMany::from(DeletedManyV1::from(deleted.clone()));
        assert_eq!(restored.deleted_ids, deleted.deleted_ids);
        assert_eq!(restored.aborted, deleted.aborted);
        assert_eq!(restored.validation_failed, deleted.validation_failed);
        assert_eq!(restored.errors, deleted.errors);
    }

    #[test]
    fn full_validation_results_become_deterministic_lists() {
        let id = |value| {
            SerializableId(vec![SerializableIdEntry {
                field_name: "id".to_owned(),
                value: IdValue::I64(value),
            }])
        };
        let violations = || Violations {
            violations: vec![Violation::major("Unknown gender.")],
        };
        let message = CollabMessage::FullValidationResult(HashMap::from([
            (
                ResourceName::new("people"),
                FullSerializableAggregateViolations {
                    general: Violations::empty(),
                    by_entity: HashMap::from([(id(2), violations()), (id(1), violations())]),
                },
            ),
            (
                ResourceName::new("clubs"),
                FullSerializableAggregateViolations::default(),
            ),
        ]));

        let wire = CollabMessageV1::from(message.clone());
        let CollabEventV1::FullValidationResult { resources } = &wire.event else {
            panic!("expected a full validation result");
        };
        let names = resources
            .iter()
            .map(|resource| resource.resource_name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["clubs", "people"]);
        let ids = resources[1]
            .violations
            .by_entity
            .iter()
            .map(|entity| SerializableId::from(entity.id.clone()))
            .collect::<Vec<_>>();
        assert_eq!(ids, [id(1), id(2)]);
        assert_eq!(CollabMessage::from(wire), message);
    }

    #[test]
    fn ids_keep_their_entries() {
        let id = SerializableId(vec![SerializableIdEntry {
            field_name: "id".to_owned(),
            value: IdValue::I64(42),
        }]);

        assert_eq!(SerializableId::from(SerializableIdV1::from(id.clone())), id);
    }
}
