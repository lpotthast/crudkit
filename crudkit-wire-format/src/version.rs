//! Selecting a wire format version from serialized data alone.

use serde::Deserializer as _;
use serde::de::{IgnoredAny, MapAccess, Visitor};
use std::fmt;

/// Name of the field carrying the version in every request and response body.
pub const VERSION_FIELD: &str = "wire_format_version";

/// A version of the wire format defined by this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WireFormatVersion {
    /// Version 1, defined in [`crate::v1`].
    V1,
}

impl WireFormatVersion {
    /// The number carried in the `wire_format_version` field.
    #[must_use]
    pub const fn number(self) -> u32 {
        match self {
            Self::V1 => 1,
        }
    }

    /// The path segment naming this version in route paths, e.g. `v1`.
    #[must_use]
    pub const fn path_segment(self) -> &'static str {
        match self {
            Self::V1 => "v1",
        }
    }

    /// Returns the version with `number`, if this crate defines it.
    #[must_use]
    pub const fn from_number(number: u32) -> Option<Self> {
        match number {
            1 => Some(Self::V1),
            _ => None,
        }
    }

    /// Reads the version of a JSON request or response body without deserializing the rest of
    /// it.
    ///
    /// # Errors
    ///
    /// Returns an error if `json` is not an object with a numeric `wire_format_version` field, or
    /// if this crate does not define that version.
    pub fn detect(json: &[u8]) -> Result<Self, DetectVersionError> {
        let mut deserializer = serde_json::Deserializer::from_slice(json);
        let number = deserializer
            .deserialize_map(VersionProbe)
            .map_err(|err| DetectVersionError::Malformed(err.to_string()))?
            .ok_or(DetectVersionError::Missing)?;
        Self::from_number(number).ok_or(DetectVersionError::Unknown(number))
    }
}

/// Reads the version field of a JSON object, skipping every other entry without interpreting it.
/// Unlike a derived struct, it rejects arrays.
struct VersionProbe;

impl<'de> Visitor<'de> for VersionProbe {
    type Value = Option<u32>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON object")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut version = None;
        while let Some(key) = map.next_key::<String>()? {
            if key == VERSION_FIELD {
                version = Some(map.next_value::<u32>()?);
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(version)
    }
}

/// Failure to read the wire format version of a body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectVersionError {
    /// The body is not a JSON object with a numeric version field.
    Malformed(String),
    /// The body has no version field.
    Missing,
    /// The body names a version this crate does not define.
    Unknown(u32),
}

impl fmt::Display for DetectVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(reason) => write!(f, "malformed wire format version: {reason}"),
            Self::Missing => write!(f, "missing field `{VERSION_FIELD}`"),
            Self::Unknown(number) => write!(f, "unknown wire format version {number}"),
        }
    }
}

impl std::error::Error for DetectVersionError {}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;

    #[test]
    fn detects_the_version_from_the_data_alone() {
        assert_that!(WireFormatVersion::detect(
            br#"{ "wire_format_version": 1, "unrelated": [1, 2] }"#
        ))
        .is_equal_to(Ok(WireFormatVersion::V1));
    }

    #[test]
    fn reports_missing_unknown_and_malformed_versions() {
        assert_that!(WireFormatVersion::detect(br#"{ "limit": 1 }"#))
            .is_equal_to(Err(DetectVersionError::Missing));
        assert_that!(WireFormatVersion::detect(
            br#"{ "wire_format_version": 7 }"#
        ))
        .is_equal_to(Err(DetectVersionError::Unknown(7)));
        assert_that!(WireFormatVersion::detect(br"[1]").is_err()).is_true();
        assert_that!(WireFormatVersion::detect(br#"{ "wire_format_version": "1" }"#).is_err())
            .is_true();
    }
}
