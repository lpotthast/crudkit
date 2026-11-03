//! The part of a resource that an instance works in.

use crudkit_core::condition::Condition;
use crudkit_core::id::{IdValue, SerializableIdEntry};

/// What the operations of an instance are restricted to: the parents that its entities live
/// below, and a condition that narrows them further.
///
/// A data source decides how it applies a scope. One that serves nested paths, such as
/// `/projects/{project_id}/spaces/{space_id}/services`, puts the parents into the path.
/// [`CrudRestDataSource`](super::CrudRestDataSource) turns them into conditions.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CrudScope {
    /// The IDs of the parents, outermost first. Each entry names the field through which the
    /// resource's entities refer to the parent, e.g. `club_id` or `space_id`, and holds the
    /// parent's ID.
    pub parents: Vec<SerializableIdEntry>,
    /// The condition that narrows every operation except creation, e.g. an instance's base
    /// condition. `None` narrows nothing.
    pub condition: Option<Condition>,
}

impl CrudScope {
    /// Returns this scope below one more parent, whose ID `id` the resource's entities refer to
    /// through `field`.
    #[must_use]
    pub fn with_parent(mut self, field: impl Into<String>, id: IdValue) -> Self {
        self.parents.push(SerializableIdEntry {
            field_name: field.into(),
            value: id,
        });
        self
    }

    /// Returns the ID of the parent that the resource's entities refer to through `field`, if the
    /// scope has that parent.
    #[must_use]
    pub fn parent(&self, field: &str) -> Option<&IdValue> {
        self.parents
            .iter()
            .find(|parent| parent.field_name == field)
            .map(|parent| &parent.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;

    #[test]
    fn parents_are_found_by_the_field_referring_to_them() {
        let scope = CrudScope::default()
            .with_parent("project_id", IdValue::I64(1))
            .with_parent("space_id", IdValue::I64(2));

        assert_that!(scope.parent("project_id")).is_equal_to(Some(&IdValue::I64(1)));
        assert_that!(scope.parent("space_id")).is_equal_to(Some(&IdValue::I64(2)));
        assert_that!(scope.parent("service_id")).is_none();
        assert_that!(
            scope
                .parents
                .iter()
                .map(|parent| parent.field_name.as_str())
                .collect::<Vec<_>>()
        )
        .is_equal_to(vec!["project_id", "space_id"]);
    }
}
