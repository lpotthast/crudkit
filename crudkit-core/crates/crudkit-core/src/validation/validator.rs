//! Validator information types.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Information about a validator, including its name and version.
/// Uses `Cow` to support both borrowed static strings and owned strings.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo<'a> {
    pub validator_name: Cow<'a, str>,
    pub validator_version: u32,
}

/// Convenience alias for `ValidatorInfo<'static>` with owned data.
pub type OwnedValidatorInfo = ValidatorInfo<'static>;

impl<'a> ValidatorInfo<'a> {
    /// Create a new `ValidatorInfo` with a static string.
    pub fn new(name: impl Into<Cow<'a, str>>, version: u32) -> ValidatorInfo<'a> {
        ValidatorInfo {
            validator_name: name.into(),
            validator_version: version,
        }
    }

    /// Create a new `ValidatorInfo` with an owned string.
    pub fn new_owned(name: String, version: u32) -> ValidatorInfo<'static> {
        ValidatorInfo {
            validator_name: Cow::Owned(name),
            validator_version: version,
        }
    }

    /// Convert to an owned version.
    pub fn into_owned(self) -> OwnedValidatorInfo {
        ValidatorInfo {
            validator_name: Cow::Owned(self.validator_name.into_owned()),
            validator_version: self.validator_version,
        }
    }
}
