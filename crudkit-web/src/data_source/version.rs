//! The version of an entity, which updates and deletions name as the version they are based on.

use serde::{Deserialize, Serialize};

/// An opaque version of an entity, e.g. the entity tag that an API names in `ETag` and expects
/// back in `If-Match`.
///
/// A model carries the version of its entity in a field marked `#[ck_field(version)]`, which is
/// no field of the model's field enum: forms neither show nor change it. CrudKit hands the version
/// back with every update and deletion that is based on the entity, and never reads it. A data
/// source that refuses a request based on an older version answers with
/// [`RequestError::Outdated`](crate::http::RequestError::Outdated).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
