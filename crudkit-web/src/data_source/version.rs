//! The version of an entity, which updates and deletions name as the version they are based on.

use serde::{Deserialize, Deserializer, Serialize};

/// An opaque version of an entity, e.g. the entity tag that an API names in `ETag` and expects
/// back in `If-Match`.
///
/// A model carries the version of its entity in a field marked `#[ck_field(version)]`, which is
/// no field of the model's field enum: forms neither show nor change it. CrudKit hands the version
/// back with every update and deletion that is based on the entity, and never reads it. A data
/// source that refuses a request based on an older version answers with
/// [`RequestError::Outdated`](crate::http::RequestError::Outdated).
///
/// A version serializes as a string. It deserializes from a string or from an integer, as APIs
/// that count versions return them, e.g. `"version": 7`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct CrudVersion(String);

impl CrudVersion {
    /// Wraps the version of a data source, as the source names it.
    pub fn new(version: impl Into<String>) -> Self {
        Self(version.into())
    }

    /// Returns the version as the data source named it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for CrudVersion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        /// The forms in which APIs return versions.
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Written {
            Text(String),
            Unsigned(u64),
            Signed(i64),
        }

        Ok(Self(match Written::deserialize(deserializer)? {
            Written::Text(text) => text,
            Written::Unsigned(number) => number.to_string(),
            Written::Signed(number) => number.to_string(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;

    #[test]
    fn versions_are_read_from_text_and_from_counts_and_written_as_text() {
        let from_text: CrudVersion =
            serde_json::from_value(serde_json::json!("7")).expect("a version");
        let from_count: CrudVersion =
            serde_json::from_value(serde_json::json!(7)).expect("a version");

        assert_that!(from_text.clone()).is_equal_to(CrudVersion::new("7"));
        assert_that!(from_count).is_equal_to(CrudVersion::new("7"));
        assert_that!(serde_json::to_value(from_text).expect("a version"))
            .is_equal_to(serde_json::json!("7"));
        assert_that!(serde_json::from_value::<CrudVersion>(serde_json::json!(1.5)).is_err())
            .is_true();
    }
}
