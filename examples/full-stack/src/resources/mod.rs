//! Instance configurations shared by both UI variants of the example.

use crudkit_leptos::prelude::*;
use std::borrow::Cow;

pub mod clubs;
pub mod people;

/// How an instance is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ui {
    /// CrudKit's built-in views, styled by CrudKit's theme.
    Builtin,
    /// The application's own views, built on CrudKit's hooks and atoms (see [`crate::marauder`]).
    Custom,
}

impl Ui {
    /// Returns the renderer of read-model `bool` fields meaning "validation errors exist".
    pub fn validation_status_renderer(self) -> FieldRenderer<DynReadField> {
        match self {
            Self::Builtin => FieldRenderer::for_validation_status(),
            Self::Custom => crate::marauder::validation_status_renderer(),
        }
    }
}

pub(crate) fn header(field: impl ErasedReadField, display_name: &'static str) -> Header {
    Header::showing(
        field,
        HeaderOptions {
            display_name: Cow::Borrowed(display_name),
            ..Default::default()
        },
    )
}

pub(crate) fn id_header(field: impl ErasedReadField) -> Header {
    Header::showing(
        field,
        HeaderOptions {
            display_name: Cow::Borrowed("#"),
            min_width: true,
            ..Default::default()
        },
    )
}

pub(crate) fn field_options(label: &'static str) -> FieldOptions {
    FieldOptions {
        label: Some(Label::new(label)),
        ..Default::default()
    }
}

pub(crate) fn disabled_field_options(label: &'static str) -> FieldOptions {
    FieldOptions {
        disabled: true,
        ..field_options(label)
    }
}
