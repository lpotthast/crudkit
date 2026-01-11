use crate::ReactiveValue;
use crate::dynamic::custom_field::{CustomCreateFields, CustomReadFields, CustomUpdateFields};
use crate::shared::crud_instance_config::{DynSelectConfig, SelectConfigTrait};
use crate::shared::fields::{render_field, render_label};
use crudkit_web::dynamic::prelude::*;
use crudkit_web::dynamic::{AnyCreateField, AnyReadField, AnyUpdateField};
use leptonic::components::prelude::*;
use leptos::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

#[component]
pub fn CrudCreateField(
    custom_fields: Signal<CustomCreateFields>,
    field_config: Signal<HashMap<AnyCreateField, DynSelectConfig>>,
    current_view: CrudSimpleView,
    field: AnyCreateField,
    field_options: FieldOptions,
    field_mode: FieldMode,
    signals: StoredValue<HashMap<AnyCreateField, ReactiveValue>>,
    value: ReactiveValue,
    value_changed: Callback<(AnyCreateField, Result<Value, String>)>, // how can we handle all possible types? serialization? TODO: Only take Value, not Result?; TODO: Use WriteSignal from ReactiveValue?
) -> impl IntoView {
    // TODO: This wrapping closure could be removed safely.
    move || {
        let id: String = format!("f{}", Uuid::new_v4().to_string());

        let field_clone = field.clone();
        let value_changed = Callback::new(move |result| match result {
            Ok(new) => value_changed.run((field_clone.clone(), Ok(new))),
            Err(err) => tracing::error!("Could not get input value: {}", err),
        });

        let field_config: Option<Box<dyn SelectConfigTrait>> =
            field_config.with(|map| map.get(&field).cloned());

        let field_options_clone = field_options.clone();

        let field_clone = field.clone();
        let has_custom_renderer = custom_fields.with(|fields| fields.contains_key(&field_clone));
        let custom_field_renderer: Option<ViewFn> = match has_custom_renderer {
            true => Some(ViewFn::from(move || {
                let field_clone = field_clone.clone();
                let field_options = field_options_clone.clone();

                match custom_fields.read().get(&field_clone) {
                    Some(custom_field) => {
                        // TODO: Is this still reactive?
                        view! {
                            { render_label(field_options.label.clone()) }
                            <div class="crud-field">
                                { custom_field.renderer.view_cb.run((signals, field_mode, field_options.clone(), value, value_changed)) }
                            </div>
                        }.into_any()
                    },
                    None => view! {
                        <Alert variant=AlertVariant::Danger>
                            <AlertTitle slot>"Missing custom field declaration!"</AlertTitle>
                            <AlertContent slot>
                                "The custom field '"
                                {format!("{field_clone:?}")}
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

#[component]
pub fn CrudUpdateField(
    custom_fields: Signal<CustomUpdateFields>,
    field_config: Signal<HashMap<AnyUpdateField, DynSelectConfig>>,
    current_view: CrudSimpleView,
    field: AnyUpdateField,
    field_options: FieldOptions,
    field_mode: FieldMode,
    signals: StoredValue<HashMap<AnyUpdateField, ReactiveValue>>,
    value: ReactiveValue,
    value_changed: Callback<(AnyUpdateField, Result<Value, String>)>, // how can we handle all possible types? serialization? TODO: Only take Value, not Result?; TODO: Use WriteSignal from ReactiveValue?
) -> impl IntoView {
    // TODO: This wrapping closure could be removed safely.
    move || {
        let id: String = format!("f{}", Uuid::new_v4().to_string());

        let field_clone = field.clone();
        let value_changed = Callback::new(move |result| match result {
            Ok(new) => value_changed.run((field_clone.clone(), Ok(new))),
            Err(err) => tracing::error!("Could not get input value: {}", err),
        });

        let field_config: Option<Box<dyn SelectConfigTrait>> =
            field_config.with(|map| map.get(&field).cloned());

        let field_options_clone = field_options.clone();

        let field_clone = field.clone();
        let has_custom_renderer = custom_fields.with(|fields| fields.contains_key(&field_clone));
        let custom_field_renderer: Option<ViewFn> = match has_custom_renderer {
            true => Some(ViewFn::from(move || {
                let field_clone = field_clone.clone();
                let field_options = field_options_clone.clone();

                match custom_fields.read().get(&field_clone) {
                    Some(custom_field) => {
                        // TODO: Is this still reactive?
                        view! {
                            { render_label(field_options.label.clone()) }
                            <div class="crud-field">
                                { custom_field.renderer.view_cb.run((signals, field_mode, field_options.clone(), value, value_changed)) }
                            </div>
                        }.into_any()
                    },
                    None => view! {
                        <Alert variant=AlertVariant::Danger>
                            <AlertTitle slot>"Missing custom field declaration!"</AlertTitle>
                            <AlertContent slot>
                                "The custom field '"
                                {format!("{field_clone:?}")}
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

#[component]
pub fn CrudReadField(
    custom_fields: Signal<CustomReadFields>,
    field_config: Signal<HashMap<AnyReadField, DynSelectConfig>>,
    current_view: CrudSimpleView,
    field: AnyReadField,
    field_options: FieldOptions,
    field_mode: FieldMode,
    signals: StoredValue<HashMap<AnyReadField, ReactiveValue>>,
    value: ReactiveValue,
    value_changed: Callback<(AnyReadField, Result<Value, String>)>, // how can we handle all possible types? serialization? TODO: Only take Value, not Result?; TODO: Use WriteSignal from ReactiveValue?
) -> impl IntoView {
    // TODO: This wrapping closure could be removed safely.
    move || {
        let id: String = format!("f{}", Uuid::new_v4().to_string());

        let field_clone = field.clone();
        let value_changed = Callback::new(move |result| match result {
            Ok(new) => value_changed.run((field_clone.clone(), Ok(new))),
            Err(err) => tracing::error!("Could not get input value: {}", err),
        });

        let field_config: Option<Box<dyn SelectConfigTrait>> =
            field_config.with(|map| map.get(&field).cloned());

        let field_options_clone = field_options.clone();

        let field_clone = field.clone();
        let has_custom_renderer = custom_fields.with(|fields| fields.contains_key(&field_clone));
        let custom_field_renderer: Option<ViewFn> = match has_custom_renderer {
            true => Some(ViewFn::from(move || {
                let field_clone = field_clone.clone();
                let field_options = field_options_clone.clone();

                match custom_fields.read().get(&field_clone) {
                    Some(custom_field) => {
                        // TODO: Is this still reactive?
                        view! {
                            { render_label(field_options.label.clone()) }
                            <div class="crud-field">
                                { custom_field.renderer.view_cb.run((signals, field_mode, field_options.clone(), value, value_changed)) }
                            </div>
                        }.into_any()
                    },
                    None => view! {
                        <Alert variant=AlertVariant::Danger>
                            <AlertTitle slot>"Missing custom field declaration!"</AlertTitle>
                            <AlertContent slot>
                                "The custom field '"
                                {format!("{field_clone:?}")}
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
