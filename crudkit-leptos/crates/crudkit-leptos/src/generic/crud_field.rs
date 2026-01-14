use crate::ReactiveValue;
use crate::generic::custom_field::CustomFields;
use crate::shared::fields::{default_field_renderer, render_label};
use crudkit_core::Value;
use crudkit_web::generic::prelude::*;
use leptos::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

#[component]
pub fn CrudField<T>(
    custom_fields: Signal<CustomFields<T>>,
    current_view: CrudSimpleView,
    field: T::Field,
    field_options: FieldOptions,
    field_mode: FieldMode,
    signals: StoredValue<HashMap<T::Field, ReactiveValue>>,
    value: ReactiveValue,
    value_changed: Callback<(T::Field, Result<Value, String>)>, // how can we handle all possible types? serialization? TODO: Only take Value, not Result?; TODO: Use WriteSignal from ReactiveValue?
) -> impl IntoView
where
    T: CrudDataTrait + 'static,
{
    let id: String = format!("f{}", Uuid::new_v4().to_string());

    let field_clone = field.clone();
    let value_changed = Callback::new(move |result| match result {
        Ok(new) => value_changed.run((field_clone.clone(), Ok(new))),
        Err(err) => tracing::error!("Could not get input value: {}", err),
    });

    let field_renderer = ViewFn::from(move || {
        match custom_fields.read().get(&field) {
            Some(custom_field) => {
                // TODO: Is this still reactive?
                view! {
                    { render_label(field_options.label.clone()) }
                    <div class="crud-field">
                        { custom_field.renderer.run((signals, field_mode, field_options.clone(), value, value_changed)) }
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
