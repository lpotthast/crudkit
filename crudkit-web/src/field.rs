//! Presentation options of fields in forms and list columns.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FieldMode {
    /// The value is displayed in a simple form.
    Display,

    Readable,

    Editable,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HeaderOptions {
    pub display_name: Cow<'static, str>,
    pub min_width: bool, // TODO: Rename to fit content.
    pub ordering_allowed: bool,
    pub date_time_display: DateTimeDisplay,
}

// TODO: we might want to use the builder pattern instead of relying on ..Default.default()
impl Default for HeaderOptions {
    fn default() -> Self {
        Self {
            display_name: Cow::Borrowed(""),
            min_width: false,
            ordering_allowed: true,
            date_time_display: DateTimeDisplay::LocalizedLocal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DateTimeDisplay {
    IsoUtc,
    LocalizedLocal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Label {
    pub name: String,
}

impl Label {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldOptions {
    pub disabled: bool,
    pub label: Option<Label>,
    pub date_time_display: DateTimeDisplay,
}

impl Default for FieldOptions {
    fn default() -> Self {
        Self {
            disabled: false,
            label: None,
            date_time_display: DateTimeDisplay::LocalizedLocal,
        }
    }
}
