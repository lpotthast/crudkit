//! Instance configurations of the example's resources.

use crudkit_leptos::prelude::*;
use std::borrow::Cow;

pub mod clubs;
pub mod people;

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
