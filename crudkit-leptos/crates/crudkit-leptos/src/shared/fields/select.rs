use crate::shared::crud_instance_config::SelectConfigTrait;
use crate::shared::fields::render_label;
use crudkit_web::{CrudSelectableTrait, FieldMode, FieldOptions, Value};
use leptonic::components::alert::{Alert, AlertContent, AlertTitle, AlertVariant};
use leptos::prelude::*;
use std::sync::Arc;

#[component]
pub fn CrudSelectField(
    id: String,
    field_config: Option<Box<dyn SelectConfigTrait>>,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Box<dyn CrudSelectableTrait>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => { move || format!("{:?}", value.get()) }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                {match field_config {
                    None => {
                        view! {
                            <Alert variant=AlertVariant::Danger>
                                <AlertTitle slot>"Config error"</AlertTitle>
                                <AlertContent slot>"Missing a field_config entry for this field."</AlertContent>
                            </Alert>
                        }.into_any()
                    }
                    Some(field_config) => {
                        field_config.render_select(
                            value,
                            Callback::new(move |o| { value_changed.run(Ok(Value::Select(o))) }),
                        )
                    }
                }}

            </div>
        }.into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                {match field_config {
                    None => view! {
                        <Alert variant=AlertVariant::Danger>
                            <AlertTitle slot>"Config error"</AlertTitle>
                            <AlertContent slot>"Missing a field_config entry for this field."</AlertContent>
                        </Alert>
                    }.into_any(),
                    Some(field_config) =>
                        field_config.render_select(
                            value,
                            Callback::new(move |o| { value_changed.run(Ok(Value::Select(o))) }),
                        )
                }}
            </div>
        }.into_any(),
    }
}

#[component]
pub fn CrudOptionalSelectField(
    id: String,
    field_config: Option<Box<dyn SelectConfigTrait>>,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<Box<dyn CrudSelectableTrait>>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => { move || format!("{:?}", value.get()) }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                {match field_config {
                    None => {
                        view! {
                            <Alert variant=AlertVariant::Danger>
                                <AlertTitle slot>"Config error"</AlertTitle>
                                <AlertContent slot>"Missing a field_config entry for this field."</AlertContent>
                            </Alert>
                        }.into_any()
                    }
                    Some(field_config) => {
                        field_config.render_optional_select(
                            value,
                            Callback::new(move |o| { value_changed.run(Ok(Value::OptionalSelect(o))) }),
                        )
                    }
                }}

            </div>
        }
            .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                {match field_config {
                    None => {
                        view! {
                            <Alert variant=AlertVariant::Danger>
                                <AlertTitle slot>"Config error"</AlertTitle>
                                <AlertContent slot>"Missing a field_config entry for this field."</AlertContent>
                            </Alert>
                        }.into_any()
                    }
                    Some(field_config) => {
                        field_config.render_optional_select(
                            value,
                            Callback::new(move |o| { value_changed.run(Ok(Value::OptionalSelect(o))) }),
                        )
                    }
                }}

            </div>
        }
            .into_any(),
    }
}
