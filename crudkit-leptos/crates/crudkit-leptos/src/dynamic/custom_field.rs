use crate::{FieldMode, FieldOptions, ReactiveValue};
use crudkit_web::{AnyField, Value};
use leptonic::prelude::ViewCallback;
use leptos::prelude::*;
use std::fmt::Debug;
use std::sync::Arc;
use std::{collections::HashMap, error::Error};

#[derive(Clone)]
pub struct CustomField {
    pub renderer: ViewCallback<(
        StoredValue<HashMap<AnyField, ReactiveValue>>, // signals
        FieldMode,                                     // field_mode
        FieldOptions,                                  // field_options
        ReactiveValue,                                 // value
        Callback<(Result<Value, Arc<dyn Error>>,)>,    // value_changed
    )>,
}

impl Debug for CustomField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomField").finish()
    }
}

// TODO: ??
impl PartialEq for CustomField {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

pub type CustomFields = HashMap<AnyField, CustomField>;

pub type CustomCreateFields = CustomFields;

pub type CustomUpdateFields = CustomFields;

pub type CustomReadFields = CustomFields;
