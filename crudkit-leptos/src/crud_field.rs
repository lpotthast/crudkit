use crate::crud_instance_config::FieldRendererRegistry;
use crate::fields::default_field_renderer;
use crate::ReactiveField;
use crudkit_core::Value;
use crudkit_web::prelude::*;
use crudkit_web::{FieldMode, FieldOptions};
use leptos::prelude::*;
use std::collections::HashMap;

#[component]
pub fn CrudField<F: TypeErasedField>(
    field_renderer_registry: Signal<FieldRendererRegistry<F>>,
    field: F,
    field_options: FieldOptions,
    field_mode: FieldMode,
    signals: StoredValue<HashMap<F, ReactiveField>>,
    value: ReactiveField,
    value_changed: Callback<(F, Result<Value, String>)>,
) -> impl IntoView {
    let field_clone = field.clone();
    let value_changed = Callback::new(move |result| match result {
        Ok(new) => value_changed.run((field_clone.clone(), Ok(new))),
        Err(err) => tracing::error!("Could not get input value: {}", err),
    });

    let field_for_closure = field.clone();

    let field_renderer = ViewFn::from(move || {
        let renderer = match field_renderer_registry.read().reg.get(&field_for_closure) {
            Some(renderer) => renderer.clone(),
            None => default_field_renderer(field.value_kind()),
        };
        renderer.view_cb.run((
            signals,
            field_for_closure.clone(),
            field_mode,
            field_options.clone(),
            value,
            value_changed,
        ))
    });

    // This additional closure is required so that each custom field, which may be another
    // crud instance, or, in general, anything that might `provide_context(T)`, have their
    // own context to do so in and not override sibling data.
    (move || field_renderer.run()).into_any()
}
