//! JSON merge patches (RFC 7396): what changed between two versions of an entity.

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
/// patch cannot set a member to `null`, so a member that is `null` counts as absent.
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
        patch.insert(key.clone(), change);
    }
    Value::Object(patch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use serde_json::json;

    /// Applies `patch` to `target`, as RFC 7396 defines it.
    fn apply(target: &Value, patch: &Value) -> Value {
        let Value::Object(patch) = patch else {
            return patch.clone();
        };
        let mut result = match target {
            Value::Object(target) => target.clone(),
            _ => Map::new(),
        };
        for (key, value) in patch {
            if value.is_null() {
                result.remove(key);
            } else {
                let current = result.get(key).cloned().unwrap_or(Value::Null);
                result.insert(key.clone(), apply(&current, value));
            }
        }
        Value::Object(result)
    }

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
            assert_that!(apply(&from, &merge_patch(&from, &to))).is_equal_to(to);
        }
    }
}
