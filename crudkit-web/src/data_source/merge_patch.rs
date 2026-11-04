//! JSON merge patches (RFC 7396): what changed between two versions of an entity, how to apply a
//! change to another version, and where that change meets another one.

use crate::json_pointer::escape_token;
use serde_json::{Map, Value};

/// Returns the JSON merge patch (RFC 7396) that turns `from` into `to`.
///
/// Between two objects, the patch names each member that changed:
///
/// - a member that `to` lacks as `null`, which removes it;
/// - a member that is an object in both as the merge patch between the two objects;
/// - any other changed member with its value in `to`. Arrays are replaced as a whole.
///
/// Equal members are left out, so the patch between two equal objects is an empty object. A merge
/// patch cannot set a member to `null`, so a member that is `null` counts as absent, and an object
/// that differs only by such members is left out too.
///
/// Between anything else than two objects, the patch is `to` itself, which replaces `from`.
#[must_use]
pub fn merge_patch(from: &Value, to: &Value) -> Value {
    let (Value::Object(from), Value::Object(to)) = (from, to) else {
        return to.clone();
    };
    let mut patch = Map::new();
    for (key, before) in from {
        let removed = !before.is_null() && to.get(key).is_none_or(Value::is_null);
        if removed {
            patch.insert(key.clone(), Value::Null);
        }
    }
    for (key, after) in to {
        let before = from.get(key).unwrap_or(&Value::Null);
        if after.is_null() || before == after {
            continue;
        }
        let change = match (before, after) {
            (Value::Object(_), Value::Object(_)) => merge_patch(before, after),
            _ => after.clone(),
        };
        let changes_nothing = before.is_object() && change.as_object().is_some_and(Map::is_empty);
        if !changes_nothing {
            patch.insert(key.clone(), change);
        }
    }
    Value::Object(patch)
}

/// Applies the JSON merge patch (RFC 7396) `patch` to `target`.
///
/// A patch that is an object sets each of its members in `target`, which becomes an object if it
/// is none: a `null` member removes the member, an object member patches the member, and any
/// other member replaces it. Any other patch replaces `target`.
pub fn apply_merge_patch(target: &mut Value, patch: &Value) {
    let Value::Object(patch) = patch else {
        patch.clone_into(target);
        return;
    };
    if !target.is_object() {
        *target = Value::Object(Map::new());
    }
    if let Value::Object(target) = target {
        for (key, value) in patch {
            if value.is_null() {
                target.remove(key);
            } else {
                apply_merge_patch(target.entry(key.clone()).or_insert(Value::Null), value);
            }
        }
    }
}

/// A member that a merge patch changes, and that also changed between the patch's base and
/// another version, to another value than the patch's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergePatchConflict {
    /// The member, as a JSON pointer (RFC 6901), e.g. `/tags/team`.
    pub pointer: String,
    /// The member's value in the other version. `None` if the other version lacks it.
    pub theirs: Option<Value>,
}

/// Returns where `patch`, a change of `base`, meets a change between `base` and `theirs`: each
/// member that the patch changes, and that `theirs` changed to another value than the patch's.
/// Applying the patch to `theirs` replaces the values of these members with the patch's.
///
/// A member that both changed to the same value is no conflict. When `theirs` replaced or removed
/// an object that encloses changed members, the conflict is the enclosing object, with its value
/// in `theirs`. A member that is `null` counts as absent, as in [`merge_patch`].
#[must_use]
pub fn merge_patch_conflicts(
    patch: &Value,
    base: &Value,
    theirs: &Value,
) -> Vec<MergePatchConflict> {
    let mut conflicts = Vec::new();
    collect_conflicts(
        patch,
        Some(base),
        Some(theirs),
        &mut String::new(),
        &mut conflicts,
    );
    conflicts
}

/// Collects the conflicts of `patch` at `pointer`, where the base has the value `base`, and the
/// other version the value `theirs`.
fn collect_conflicts(
    patch: &Value,
    base: Option<&Value>,
    theirs: Option<&Value>,
    pointer: &mut String,
    conflicts: &mut Vec<MergePatchConflict>,
) {
    if let (Value::Object(members), Some(Value::Object(base))) = (patch, base) {
        let Some(Value::Object(theirs)) = theirs else {
            // The other version replaced or removed the object that the patch changes members of.
            conflicts.push(MergePatchConflict {
                pointer: pointer.clone(),
                theirs: theirs.filter(|theirs| !theirs.is_null()).cloned(),
            });
            return;
        };
        for (key, member) in members {
            let length = pointer.len();
            pointer.push('/');
            pointer.push_str(&escape_token(key));
            collect_conflicts(member, base.get(key), theirs.get(key), pointer, conflicts);
            pointer.truncate(length);
        }
        return;
    }
    let present = |value: Option<&Value>| value.filter(|value| !value.is_null()).cloned();
    let (base, theirs) = (present(base), present(theirs));
    let mut mine = base.clone().unwrap_or(Value::Null);
    apply_merge_patch(&mut mine, patch);
    let mine = present(Some(&mine));
    if base != theirs && theirs != mine {
        conflicts.push(MergePatchConflict {
            pointer: pointer.clone(),
            theirs,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use serde_json::json;

    #[test]
    fn equal_objects_need_no_change() {
        let entity = json!({ "name": "Ada", "tags": ["a"], "address": { "city": "Kiel" } });

        assert_that!(merge_patch(&entity, &entity)).is_equal_to(json!({}));
    }

    #[test]
    fn patches_name_changed_added_and_removed_members() {
        let from = json!({ "name": "Ada", "note": "old", "nickname": "A", "empty": null });
        let to =
            json!({ "name": "Ada Lovelace", "note": null, "title": "Countess", "empty": null });

        assert_that!(merge_patch(&from, &to)).is_equal_to(json!({
            "name": "Ada Lovelace",
            "note": null,
            "nickname": null,
            "title": "Countess",
        }));
    }

    #[test]
    fn nested_objects_are_patched_and_arrays_replaced() {
        let from = json!({
            "variables": { "A": "1", "B": "2" },
            "ports": [80, 443],
            "limits": 3,
        });
        let to = json!({
            "variables": { "A": "1", "C": "3" },
            "ports": [80],
            "limits": { "cpu": 2 },
        });

        assert_that!(merge_patch(&from, &to)).is_equal_to(json!({
            "variables": { "B": null, "C": "3" },
            "ports": [80],
            "limits": { "cpu": 2 },
        }));
    }

    #[test]
    fn objects_that_differ_only_by_null_members_are_left_out() {
        let from = json!({ "address": { "city": "Kiel", "zip": null } });
        let to = json!({ "address": { "city": "Kiel" } });

        assert_that!(merge_patch(&from, &to)).is_equal_to(json!({}));
    }

    #[test]
    fn anything_but_two_objects_is_replaced() {
        assert_that!(merge_patch(&json!([1]), &json!({ "a": 1 }))).is_equal_to(json!({ "a": 1 }));
        assert_that!(merge_patch(&json!({ "a": 1 }), &json!("text"))).is_equal_to(json!("text"));
    }

    #[test]
    fn applying_the_patch_turns_one_object_into_the_other() {
        let pairs = [
            (
                json!({ "a": 1, "b": { "c": 2, "d": 3 } }),
                json!({ "b": { "c": 4 }, "e": [5] }),
            ),
            (
                json!({ "a": { "b": { "c": 1 } } }),
                json!({ "a": { "b": 2 } }),
            ),
            (json!({ "a": [1, 2] }), json!({ "a": { "b": 1 } })),
            (json!({}), json!({ "a": { "b": 1 } })),
        ];
        for (from, to) in pairs {
            let mut patched = from.clone();
            apply_merge_patch(&mut patched, &merge_patch(&from, &to));
            assert_that!(patched).is_equal_to(to);
        }
    }

    #[test]
    fn a_change_applied_to_another_version_keeps_the_other_changes() {
        let base = json!({ "tags": { "a": "1" }, "name": "x" });
        let mine = json!({ "tags": { "a": "1", "mine": "m" }, "name": "x" });
        let theirs = json!({ "tags": { "a": "1", "theirs": "t" }, "name": "y" });

        let patch = merge_patch(&base, &mine);
        let mut merged = theirs.clone();
        apply_merge_patch(&mut merged, &patch);

        assert_that!(merged)
            .is_equal_to(json!({ "tags": { "a": "1", "mine": "m", "theirs": "t" }, "name": "y" }));
        assert_that!(merge_patch_conflicts(&patch, &base, &theirs)).is_empty();
    }

    #[test]
    fn replacing_an_object_conflicts_with_changes_of_its_members() {
        let base = json!({ "limits": { "cpu": 1, "memory": 1 }, "name": "x" });
        let mine = json!({ "limits": { "cpu": 2, "memory": 1 }, "name": "x" });
        let patch = merge_patch(&base, &mine);

        let replaced = json!({ "limits": "unlimited", "name": "x" });
        let removed = json!({ "name": "x" });

        assert_that!(merge_patch_conflicts(&patch, &base, &replaced)).is_equal_to(vec![
            MergePatchConflict {
                pointer: "/limits".to_owned(),
                theirs: Some(json!("unlimited")),
            },
        ]);
        assert_that!(merge_patch_conflicts(&patch, &base, &removed)).is_equal_to(vec![
            MergePatchConflict {
                pointer: "/limits".to_owned(),
                theirs: None,
            },
        ]);
    }

    #[test]
    fn members_that_both_changed_differently_conflict() {
        let base = json!({ "tags": { "a/b": "1", "c": "1" }, "name": "x", "note": "n" });
        let mine = json!({ "tags": { "a/b": "mine", "c": "same" }, "name": "mine" });
        let theirs = json!({ "tags": { "a/b": "theirs", "c": "same" }, "note": "n" });

        let patch = merge_patch(&base, &mine);

        // Both changed `c` to the same value, and only the patch removed `note`.
        assert_that!(merge_patch_conflicts(&patch, &base, &theirs)).is_equal_to(vec![
            MergePatchConflict {
                pointer: "/name".to_owned(),
                theirs: None,
            },
            MergePatchConflict {
                pointer: "/tags/a~1b".to_owned(),
                theirs: Some(json!("theirs")),
            },
        ]);
    }
}
