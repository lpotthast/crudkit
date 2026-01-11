use crate::ReactiveValue;
use crudkit_web::dynamic::prelude::*;
use crudkit_web::dynamic::{AnyCreateField, AnyReadField, AnyUpdateField};
use leptonic::prelude::ViewCallback;
use leptos::prelude::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Clone)]
pub struct CustomFieldRenderer<F: 'static> {
    pub(crate) view_cb: ViewCallback<(
        StoredValue<HashMap<F, ReactiveValue>>,              // signals
        FieldMode,                                           // field_mode
        FieldOptions,                                        // field_options
        ReactiveValue,                                       // value
        Callback<Result<Value, Arc<dyn std::error::Error>>>, // value_changed
    )>,
}

impl<F> CustomFieldRenderer<F> {
    pub fn new(
        view_fn: impl Fn(
            StoredValue<HashMap<F, ReactiveValue>>,
            FieldMode,
            FieldOptions,
            ReactiveValue,
            Callback<Result<Value, Arc<dyn std::error::Error>>>,
        ) -> AnyView
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            view_cb: ViewCallback::new(Callback::from(view_fn)),
        }
    }
}

#[derive(Clone)]
pub struct CustomField<F: 'static> {
    pub renderer: CustomFieldRenderer<F>,
}

impl<F> Debug for CustomField<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("$custom_field").finish_non_exhaustive()
    }
}

// TODO: ??
impl<F> PartialEq for CustomField<F> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

pub type CustomCreateFields = HashMap<AnyCreateField, CustomField<AnyCreateField>>;

pub type CustomUpdateFields = HashMap<AnyUpdateField, CustomField<AnyUpdateField>>;

pub type CustomReadFields = HashMap<AnyReadField, CustomField<AnyReadField>>;
