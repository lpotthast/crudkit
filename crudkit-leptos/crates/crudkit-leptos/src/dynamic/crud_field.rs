use crate::dynamic::crud_instance_config::FieldRendererRegistry;
use crate::shared::fields::{default_field_renderer, render_label};
use crate::ReactiveValue;
use crudkit_web::dynamic::prelude::*;
use crudkit_web::dynamic::{AnyCreateField, AnyReadField, AnyUpdateField};
use leptonic::components::prelude::*;
use leptos::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

#[component]
pub fn CrudField<F: DynField>(
    field_renderer_registry: Signal<FieldRendererRegistry<F>>,
    current_view: CrudSimpleView,
    field: F,
    field_options: FieldOptions,
    field_mode: FieldMode,
    signals: StoredValue<HashMap<F, ReactiveValue>>,
    value: ReactiveValue,
    value_changed: Callback<(F, Result<Value, String>)>, // how can we handle all possible types? serialization? TODO: Only take Value, not Result?; TODO: Use WriteSignal from ReactiveValue?
) -> impl IntoView {
    let id: String = format!("f{}", Uuid::new_v4().to_string());

    let field_clone = field.clone();
    let value_changed = Callback::new(move |result| match result {
        Ok(new) => value_changed.run((field_clone.clone(), Ok(new))),
        Err(err) => tracing::error!("Could not get input value: {}", err),
    });

    let field_renderer = ViewFn::from(move || {
        match field_renderer_registry.read().reg.get(&field) {
            Some(renderer) => {
                // TODO: Is this still reactive?
                view! {
                        { render_label(field_options.label.clone()) }
                        <div class="crud-field">
                            { renderer.view_cb.run((signals, field_mode, field_options.clone(), value, value_changed)) }
                        </div>
                    }.into_any()
            }
            None => default_field_renderer(
                value,
                id.clone(),
                field_options.clone(),
                field_mode,
                value_changed,
            )
            .into_any(),
        }
    });

    // This additional closure is required so that each custom field, which may be another
    // crud instance, or, in general, anything that might `provide_context(T)`, have their
    // own context to do so and not override sibling data.
    (move || field_renderer.run()).into_any()
}
