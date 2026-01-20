pub mod validator;
pub mod violation;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::validator::OwnedValidatorInfo;
use crate::violation::{Severity, Violation, Violations};
use crudkit_id::{Id, SerializableId};
use crudkit_resource::ResourceName;

/// Violations for one entity.
#[derive(Debug, Clone, Serialize)]
pub struct ViolationsByValidator {
    pub violations_by_validator: HashMap<OwnedValidatorInfo, Violations>,
}

impl ViolationsByValidator {
    pub fn new() -> Self {
        Self {
            violations_by_validator: HashMap::new(),
        }
    }

    pub fn of(validator: OwnedValidatorInfo, violations: Violations) -> Self {
        let mut entity_violations = HashMap::new();
        entity_violations.insert(validator, violations);
        Self {
            violations_by_validator: entity_violations,
        }
    }

    /// Extend this violation collection with the `violations` created by `validator`.
    /// This is used to combine results from multiple entity validators.
    pub fn push(&mut self, validator: OwnedValidatorInfo, violation: Violation) {
        self.violations_by_validator
            .entry(validator)
            .or_default()
            .push(violation);
    }

    /// Extend this violation collection with the `violations` created by `validator`.
    /// This is used to combine results from multiple entity validators.
    pub fn extend(&mut self, validator: OwnedValidatorInfo, violations: Violations) {
        self.violations_by_validator
            .entry(validator)
            .or_default()
            .extend(violations);
    }

    pub fn drop_critical(&mut self) {
        self.violations_by_validator
            .iter_mut()
            .for_each(|(_, violations)| violations.drop_critical())
    }

    pub fn number_of_violations(&self) -> usize {
        self.violations_by_validator
            .values()
            .fold(0usize, |acc, violations| acc + violations.len())
    }

    pub fn has_violations(&self) -> bool {
        for (_validator, violations) in &self.violations_by_validator {
            if !violations.is_empty() {
                return true;
            }
        }
        false
    }

    pub fn has_any_violations_of(&self, severity: Severity) -> bool {
        for (_validator, violations) in &self.violations_by_validator {
            for violation in violations.iter() {
                if violation.is_of_severity(severity) {
                    return true;
                }
            }
        }
        false
    }

    pub fn has_critical_violations(&self) -> bool {
        self.has_any_violations_of(Severity::Critical)
    }
}

/// All entity violations for one resource (resource identifier not stored).
pub struct ViolationsByEntity<I: Id> {
    pub map: HashMap<I, ViolationsByValidator>,
}

impl<I: Id> ViolationsByEntity<I> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn of_entity_violations(entity_id: I, violations: ViolationsByValidator) -> Self {
        let mut map = HashMap::new();
        map.insert(entity_id, violations);
        Self { map }
    }

    pub fn push(&mut self, entity_id: I, validator_info: OwnedValidatorInfo, violation: Violation) {
        self.map
            .entry(entity_id)
            .or_insert_with(ViolationsByValidator::new)
            .push(validator_info, violation);
    }
}

/// Violations for one resource type.  
pub struct ResourceViolations<I: Id> {
    /// Violations targeting the resource as a whole. Not tied to a specific entity.
    pub general: Violations,

    /// Violations unrelated to a specific entity.
    pub create: Violations,

    /// Violations targeting specific entities.
    pub by_entity: ViolationsByEntity<I>,
}

impl<I: Id> ResourceViolations<I> {
    pub fn new() -> Self {
        Self {
            general: Violations::empty(),
            create: Violations::empty(),
            by_entity: ViolationsByEntity::new(),
        }
    }
}

pub struct ViolationsByResource {
    // Note: We have to use the type-erased SerializableId here, because each resource uses a
    // different type of ID.
    pub map: HashMap<ResourceName, ResourceViolations<SerializableId>>,
}

impl ViolationsByResource {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

// ------------------------------------------------------------------------------------------------
// validator-info erased alternatives.
// ------------------------------------------------------------------------------------------------

pub type FullSerializableValidations = HashMap<ResourceName, FullSerializableAggregateViolations>;

pub type PartialSerializableValidations =
    HashMap<ResourceName, PartialSerializableAggregateViolations>;

pub fn into_serializable_validations(
    data: ViolationsByResource,
) -> HashMap<ResourceName, FullSerializableAggregateViolations> {
    let mut result = HashMap::new();
    for (resource, resource_violations) in data.map {
        let aggregate_violations: &mut FullSerializableAggregateViolations =
            result.entry(resource).or_insert_with(Default::default);
        for (entity_id, violations_by_validator) in resource_violations.by_entity.map {
            let entity_violations = aggregate_violations.by_entity.entry(entity_id).or_default();

            for (_validator, violations) in violations_by_validator.violations_by_validator {
                entity_violations.reserve(violations.len());
                entity_violations.extend(violations);
            }
        }
    }
    result
}

/// A FULL validation result for ONE resource type. Only obtainable through global validation.
///
/// Full means: We know that all entities of this resource were validated. Receivers can safely
/// replace all previously known validation results.
///
/// All violations must either target the resource or a specific entity.
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct FullSerializableAggregateViolations {
    /// Violations targeting the resource as a whole. Not tied to a specific entity.
    pub general: Violations,

    /// Violations targeting specific entities.
    /// If the map does not contain an entry for an entity ID, then no information is present.
    /// If it does, the contained Vec must hold ALL violations for the entity.
    pub by_entity: HashMap<SerializableId, Violations>,
}

impl FullSerializableAggregateViolations {
    pub fn has_entity_or_general_violations(&self) -> bool {
        let has_general_violations = !self.general.is_empty();
        let has_entity_violations = !self.by_entity.is_empty()
            && self
                .by_entity
                .iter()
                .any(|(_, violations)| !violations.is_empty());
        has_general_violations || has_entity_violations
    }
}

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct PartialSerializableAggregateViolations {
    /// Violations targeting the resource as a whole. Not tied to a specific entity.
    /// If Option::Empty, no information is present. In an Option::Some, the contained Vec must hold ALL violations for the resource at hand.
    pub general: Option<Violations>,

    /// Violations unrelated to any known entity.
    /// If Option::Empty, no information is present. In an Option::Some, the contained Vec must hold ALL violations for the resource at hand.
    pub create: Option<Violations>,

    /// Violations targeting specific entities.
    /// If the map does not contain an entry for an entity ID, then no information is present.
    /// If it does, the contained Vec must hold ALL violations for the entity.
    pub by_entity: Vec<(SerializableId, Violations)>,
}

impl PartialSerializableAggregateViolations {
    pub fn from(value: ViolationsByValidator, entity_id: Option<SerializableId>) -> Self {
        let mut create = Option::<Violations>::None;
        let mut by_entity = HashMap::<SerializableId, Violations>::new();

        for (_validator, violations) in value.violations_by_validator {
            if let Some(entity_id) = &entity_id {
                by_entity
                    .entry(entity_id.clone())
                    .or_default()
                    .extend(violations);
            } else {
                create.get_or_insert_default().extend(violations);
            }
        }

        Self {
            general: None,
            create,
            by_entity: by_entity.into_iter().collect(),
        }
    }

    /// Returns true if there are no violations in any category.
    pub fn is_empty(&self) -> bool {
        let general_empty = self.general.as_ref().map_or(true, |v| v.is_empty());
        let create_empty = self.create.as_ref().map_or(true, |v| v.is_empty());
        let by_entity_empty =
            self.by_entity.is_empty() || self.by_entity.iter().all(|(_id, v)| v.is_empty());
        general_empty && create_empty && by_entity_empty
    }
}

#[cfg(test)]
mod tests {
    use crate::violation::Violations;
    use crate::{PartialSerializableAggregateViolations, PartialSerializableValidations};
    use assertr::prelude::*;
    use crudkit_id::{IdValue, SerializableId, SerializableIdEntry};
    use crudkit_resource::ResourceName;
    use std::collections::HashMap;

    #[test]
    fn serialize_and_deserialize_serializable_id() {
        let partial: PartialSerializableValidations = HashMap::from([(
            ResourceName::new("foo"),
            PartialSerializableAggregateViolations {
                general: None,
                create: None,
                by_entity: vec![(
                    SerializableId(vec![SerializableIdEntry {
                        field_name: "bar".into(),
                        value: IdValue::I32(1),
                    }]),
                    Violations::empty(),
                )],
            },
        )]);

        let json = serde_json::to_string(&partial).unwrap();

        assert_that(&json).is_equal_to(r#"{"foo":{"general":null,"create":null,"by_entity":[[[["bar",{"I32":1}]],{"violations":[]}]]}}"#);
    }
}
