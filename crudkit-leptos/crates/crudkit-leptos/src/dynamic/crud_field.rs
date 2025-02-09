use crate::dynamic::custom_field::CustomFields;
use crate::shared::crud_instance_config::{DynSelectConfig, SelectConfigTrait};
use crate::shared::fields::{render_field, render_label};
use crate::ReactiveValue;
use crudkit_web::{prelude::*, AnyField, DateTimeDisplay, JsonValue};
use leptonic::components::prelude::*;
use leptonic::prelude::*;
use leptos::prelude::*;
use std::sync::Arc;
use std::{borrow::Cow, collections::HashMap, error::Error};
use time::{
    format_description::well_known::Rfc3339, macros::format_description, PrimitiveDateTime,
};
use uuid::Uuid;

#[component]
pub fn CrudField(
    custom_fields: Signal<CustomFields>,
    field_config: Signal<HashMap<AnyField, DynSelectConfig>>,
    api_base_url: Signal<String>,
    current_view: CrudSimpleView,
    field: AnyField,
    field_options: FieldOptions,
    field_mode: FieldMode,
    signals: StoredValue<HashMap<AnyField, ReactiveValue>>,
    value: ReactiveValue,
    value_changed: Callback<(AnyField, Result<Value, String>)>, // how can we handle all possible types? serialization? TODO: Only take Value, not Result?; TODO: Use WriteSignal from ReactiveValue?
) -> impl IntoView {
    move || {
        let id: String = format!("f{}", Uuid::new_v4().to_string());

        let field_clone = field.clone();
        let field_clone3 = field.clone();

        let value_changed: Callback<(Result<Value, Arc<dyn Error>>,)> =
            Callback::new(move |(result,)| match result {
                Ok(new) => value_changed.run((field_clone.clone(), Ok(new))),
                Err(err) => tracing::error!("Could not get input value: {}", err),
            });

        let field_config: Option<Box<dyn SelectConfigTrait>> =
            field_config.with(|map| map.get(&field).cloned());

        let field_options_clone = field_options.clone();

        let has_custom_renderer = custom_fields.with(|fields| fields.contains_key(&field_clone3));
        let custom_field_renderer: Option<ViewFn> = match has_custom_renderer {
            true => Some(ViewFn::from(move || {
                let field_clone3 = field_clone3.clone();
                let field_options = field_options_clone.clone();

                match custom_fields.read().get(&field_clone3) {
                    Some(custom_field) => {
                        // TODO: Is this still reactive?
                        view! {
                            { render_label(field_options.label.clone()) }
                            <div class="crud-field">
                                { custom_field.renderer.run((signals, field_mode, field_options.clone(), value, value_changed)) }
                            </div>
                        }.into_any()
                    },
                    None => view! {
                        <Alert variant=AlertVariant::Danger>
                            <AlertTitle slot>"Missing custom field declaration!"</AlertTitle>
                            <AlertContent slot>
                                "The custom field '"
                                {format!("{field_clone3:?}")}
                                "' should have been displayed here, but no renderer for that field was found in the `custom_*_fields` section of the static instance config. You might have forgotten to set the required HashMap entry."
                            </AlertContent>
                        </Alert>
                    }.into_any(),
                }
            })),
            false => None,
        };

        render_field(
            value,
            id,
            field_options.clone(),
            field_mode,
            field_config,
            value_changed,
            custom_field_renderer,
        )
    }
}
