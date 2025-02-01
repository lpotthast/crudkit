use crate::{
    crud_field_label::CrudFieldLabelOpt,
    crud_instance_config::{DynSelectConfig, SelectConfigTrait},
    prelude::CustomFields,
    ReactiveValue,
};
use crudkit_web::{prelude::*, DateTimeDisplay, JsonValue};
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
pub fn CrudField<T>(
    custom_fields: Signal<CustomFields<T>>,
    field_config: Signal<HashMap<T::Field, DynSelectConfig>>,
    api_base_url: Signal<String>,
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
    #[inline(never)]
    fn render_field(
        value: ReactiveValue,
        id: String,
        field_options: FieldOptions,
        field_mode: FieldMode,
        field_config: Option<Box<dyn SelectConfigTrait>>,
        value_changed: Callback<(Result<Value, Arc<dyn Error>>,)>,
        // TODO: can this be ViewFnOnce?
        custom_field_renderer: Option<ViewFn>,
    ) -> impl IntoView {
        match custom_field_renderer {
            Some(custom_field_renderer) => custom_field_renderer.run(),
            None => match value {
                ReactiveValue::String(value) => {
                    view! {
                        <CrudStringField
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::OptionalString(value) => {
                    view! {
                        <CrudOptionalStringField
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::Text(value) => {
                    view! {
                        <CrudTextField
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::Json(value) => {
                    view! {
                        <CrudJsonField
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::OptionalJson(value) => {
                    view! {
                        <CrudOptionalJsonField
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::UuidV4(value) => {
                    view! {
                        <CrudUuidV4Field
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::UuidV7(value) => {
                    view! {
                        <CrudUuidV7Field
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::I32(value) => {
                    view! {
                        <CrudI32Field
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::I64(value) => {
                    view! {
                        <CrudI64Field
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::U32(value) => {
                    view! {
                        <CrudU32Field
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::U64(value) => {
                    view! {
                        <CrudU64Field
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::OptionalU32(value) => {
                    view! {
                        <CrudOptionalU32Field
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::OptionalU64(value) => {
                    view! {
                        <CrudOptionalU64Field
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::OptionalI32(value) => {
                    view! {
                        <CrudOptionalI32Field
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::OptionalI64(value) => {
                    view! {
                        <CrudOptionalI64Field
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::F32(value) => {
                    view! {
                        <CrudF32Field
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::F64(value) => {
                    view! {
                        <CrudF64Field
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::Bool(value) => {
                    view! {
                        <CrudBoolField
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::ValidationStatus(value) => {
                    view! { <CrudValidationStatusField id=id.clone() field_options=field_options field_mode=field_mode value=value/> }.into_any()
                },
                ReactiveValue::PrimitiveDateTime(value) => {
                    view! {
                        <CrudPrimitiveDateTimeField
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::OffsetDateTime(_) => view! { "TODO: Render ReactiveValue::OffsetDateTime" }.into_any(),
                ReactiveValue::OptionalPrimitiveDateTime(value) => {
                    view! {
                        <CrudOptionalPrimitiveDateTimeField
                            id=id.clone()
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::OptionalOffsetDateTime(_) => view! { "TODO: Render ReactiveValue::OptionalOffsetDateTime" }.into_any(),
                ReactiveValue::OneToOneRelation(_) => view! { "TODO: Render ReactiveValue::OneToOneRelation" }.into_any(),
                ReactiveValue::Reference(_) => view! { "TODO: Render ReactiveValue::NestedTable" }.into_any(),
                ReactiveValue::Custom(_) => panic!("should have had custom field renderer"), // custom_field_renderer(),
                ReactiveValue::Select(value) => {
                    view! {
                        <CrudSelectField
                            id=id.clone()
                            field_config=field_config
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::Multiselect(_) => view! { "TODO: Render ReactiveValue::Multiselect" }.into_any(),
                ReactiveValue::OptionalSelect(value) => {
                    view! {
                        <CrudOptionalSelectField
                            id=id.clone()
                            field_config=field_config
                            field_options=field_options
                            field_mode=field_mode
                            value=value
                            value_changed=value_changed
                        />
                    }.into_any()
                },
                ReactiveValue::OptionalMultiselect(_) => view! { "TODO: Render ReactiveValue::OptionalMultiselect" }.into_any(),
            }
        }
    }

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

#[component]
pub fn CrudStringField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<String>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TextInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=value
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TextInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=value
                    set=move |new| value_changed.run((Ok(Value::String(new)),))
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalStringField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<String>>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TextInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get().unwrap_or_default())
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TextInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get().unwrap_or_default())
                    set=move |new| value_changed.run((Ok(Value::String(new)),))
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudTextField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<String>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TiptapEditor
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    value=value
                    disabled=true
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TiptapEditor
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    value=value
                    set_value=move |new| {
                        value_changed.run((
                            Ok(Value::Text(
                                match new {
                                    TiptapContent::Html(content) => content,
                                    TiptapContent::Json(content) => content,
                                }
                            ))
                        ,))
                    }
                    disabled=field_options.disabled
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudJsonField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<JsonValue>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => {
            view! { <div>{move || value.get().get_string_representation().to_owned()}</div> }
        }
        .into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                // TODO: Implement a proper Json editor
                {render_label(field_options.label.clone())}
                <TiptapEditor
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    value=Signal::derive(move || value.get().get_string_representation().to_owned())
                    disabled=true
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                // TODO: Implement a proper Json editor
                {render_label(field_options.label.clone())}
                <TiptapEditor
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    value=Signal::derive(move || value.get().get_string_representation().to_owned())
                    set_value=move |new| {
                        value_changed.run((
                            match new {
                                TiptapContent::Html(content) => serde_json::from_str(&content),
                                TiptapContent::Json(content) => serde_json::from_str(&content),
                            }
                                .map(|json_value| Value::Json(JsonValue::new(json_value)))
                                .map_err(|err| Arc::new(err) as Arc<dyn Error>)
                        ,));
                    }

                    disabled=field_options.disabled
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalJsonField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<JsonValue>>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {
            <div>
                {move || {
                    value
                        .get()
                        .as_ref()
                        .map(|it| Cow::Owned(it.get_string_representation().to_owned()))
                        .unwrap_or(Cow::Borrowed(""))
                }}

            </div>
        }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} "TODO: Implement TipTap editor or Json editor"
            // <CrudTipTapEditor
            // api_base_url={ctx.props().api_base_url.clone()}
            // id={self.format_id()}
            // class={"crud-input-field"}
            // value={value.as_ref().map(|it| it.get_string_representation().to_owned()).unwrap_or_default()}
            // disabled={true}
            // />
            </div>
        }.into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} "TODO: Implement TipTap editor or Json editor"
            // <CrudTipTapEditor
            // api_base_url={ctx.props().api_base_url.clone()}
            // id={self.format_id()}
            // class={"crud-input-field"}
            // value={value.as_ref().map(|it| it.get_string_representation().to_owned()).unwrap_or_default()}
            // onchange={ctx.link().callback(|input| Msg::Send(Value::Text(input)))}
            // disabled={options.disabled}
            // />
            </div>
        }.into_any(),
    }
}

#[component]
pub fn CrudUuidV4Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Uuid>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get().to_string()}</div> }.into_any(),
        // Never editable, TODO: why though? we could allow editing uuids if we can guarantee their validity.
        FieldMode::Readable | FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TextInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || { value.get().to_string() })
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudUuidV7Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Uuid>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get().to_string()}</div> }.into_any(),
        // Never editable, TODO: why though? we could allow editing uuids if we can guarantee their validity.
        FieldMode::Readable | FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TextInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || { value.get().to_string() })
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudU32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<u32>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get() as f64)
                    set=move |new: f64| { value_changed.run((Ok(Value::U32(new as u32)),)) }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudU64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<u64>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get() as f64)
                    set=move |new: f64| { value_changed.run((Ok(Value::U64(new as u64)),)) }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalU32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u32>>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => {
            move || match value.get() {
                Some(value) => view! { <div>{value}</div> }.into_any(),
                None => view! { <div>"-"</div> }.into_any(),
            }
        }
        .into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                    set=move |new: f64| {
                        value_changed.run((Ok(Value::OptionalU32(Some(new as u32))),))
                    }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalU64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u64>>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => {
            move || match value.get() {
                Some(value) => view! { <div>{value}</div> }.into_any(),
                None => view! { <div>"-"</div> }.into_any(),
            }
        }
        .into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                    set=move |new: f64| {
                        value_changed.run((Ok(Value::OptionalU64(Some(new as u64))),))
                    }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudI32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<i32>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get() as f64)
                    set=move |new: f64| { value_changed.run((Ok(Value::I32(new as i32)),)) }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalI32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i32>>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => {
            move || match value.get() {
                Some(value) => view! { <div>{value}</div> }.into_any(),
                None => view! { <div>"-"</div> }.into_any(),
            }
        }
        .into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                    set=move |new: f64| {
                        value_changed.run((Ok(Value::OptionalI32(Some(new as i32))),))
                    }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudI64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<i64>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get() as f64)
                    set=move |new: f64| { value_changed.run((Ok(Value::I64(new as i64)),)) }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalI64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i64>>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => {
            move || match value.get() {
                Some(value) => view! { <div>{value}</div> }.into_any(),
                None => view! { <div>"-"</div> }.into_any(),
            }
        }
        .into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                    set=move |new: f64| {
                        value_changed.run((Ok(Value::OptionalI64(Some(new as i64))),))
                    }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudF32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<f32>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get() as f64)
                    set=move |new: f64| { value_changed.run((Ok(Value::F32(new as f32)),)) }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudF64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<f64>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get())
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get())
                    set=move |new: f64| { value_changed.run((Ok(Value::F64(new)),)) }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudBoolField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<bool>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
                    <Toggle state=value disabled=true/>
                </div>
            </div>
        }.into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
                    <Toggle
                        state=value
                        set_state=move |new| { value_changed.run((Ok(Value::Bool(new)),)) }
                        disabled=field_options.disabled
                    />
                </div>
            </div>
        }.into_any(),
    }
}

#[component]
pub fn CrudValidationStatusField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<bool>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {
            <div>
                {move || match value.get() {
                    true => view! { <Icon icon=icondata::BsExclamationTriangleFill/> },
                    false => view! { <Icon icon=icondata::BsCheck/> },
                }}

            </div>
        }.into_any(),
        FieldMode::Readable | FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
                    {move || match value.get() {
                        true => view! { <Icon icon=icondata::BsExclamationTriangleFill/> },
                        false => view! { <Icon icon=icondata::BsCheck/> },
                    }}

                </div>
            </div>
        }.into_any(),
    }
}

#[component]
pub fn CrudPrimitiveDateTimeField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<PrimitiveDateTime>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => match field_options.date_time_display {
            DateTimeDisplay::IsoUtc => {
                view! { <div>{move || value.get().format(&Rfc3339).unwrap()}</div> }.into_any()
            }
            // TODO: Use icu4x formatting using the current users locale!
            DateTimeDisplay::LocalizedLocal => view! {
                <div>
                    {move || {
                        value.get().format(format_description!("[day].[month].[year] [hour]:[minute]")).unwrap()
                    }}
                </div>
            }.into_any(),
        },
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
                    <DateTimeInput
                        id=id.clone()
                        get=Signal::derive(move || Some(value.get().assume_utc()))
                        set=move |_v| {}
                        disabled=true
                    />
                </div>
            </div>
        }.into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
                    <DateTimeInput
                        id=id.clone()
                        get=Signal::derive(move || Some(value.get().assume_utc()))
                        set={move |v: Option<time::OffsetDateTime>| {
                            match v {
                                Some(v) => {
                                    value_changed
                                        .run((Ok(Value::PrimitiveDateTime(PrimitiveDateTime::new(v.date(), v.time()))),))
                                }
                                None => {}
                            }
                        }}
                        disabled=field_options.disabled
                    />
                </div>
            </div>
        }.into_any(),
    }
}

#[component]
pub fn CrudOptionalPrimitiveDateTimeField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<PrimitiveDateTime>>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => match field_options.date_time_display {
            DateTimeDisplay::IsoUtc => {move || match value.get() {
                Some(date_time) => view! { <div>{date_time.format(&Rfc3339).unwrap()}</div> }.into_any(),
                None => view! { <div>""</div> }.into_any(),
            }}.into_any(),
            DateTimeDisplay::LocalizedLocal => {move || match value.get() {
                // TODO: Use icu4x formatting using the current users locale!
                Some(date_time) => view! { <div>{date_time.format(format_description!("[day].[month].[year] [hour]:[minute]")).unwrap()}</div> }.into_any(),
                None => view! { <div>""</div> }.into_any(),
            }}.into_any(),
        },
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} "TODO: DataTime input field"
            // id=id.clone()
            // ty=InputType::Number
            // class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
            // disabled=true
            // get=format!("{value}")
            // set=move |_| {}
            // />
            </div>
        }.into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                "TODO: DataTime input field" {render_label(field_options.label.clone())}
            // <CrudOffsetDatetime
            // id={self.format_id()}
            // value={optional_primitive_date_time.clone().map(|it| it.assume_utc())}
            // onchange={ctx.link().callback(|datetime: Option<time::OffsetDateTime>| Msg::Send(Value::OptionalOffsetDateTime(datetime)))}
            // disabled={options.disabled}
            // />
            </div>
        }.into_any(),
    }
}

#[component]
pub fn CrudSelectField(
    id: String,
    field_config: Option<Box<dyn SelectConfigTrait>>,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Box<dyn CrudSelectableTrait>>,
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
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
                            Callback::new(move |o| { value_changed.run((Ok(Value::Select(o)),)) }),
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
                            Callback::new(move |o| { value_changed.run((Ok(Value::Select(o)),)) }),
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
    value_changed: Callback<(Result<Value, Arc<dyn std::error::Error>>,)>,
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
                            Callback::new(move |o| { value_changed.run((Ok(Value::OptionalSelect(o)),)) }),
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
                            Callback::new(move |o| { value_changed.run((Ok(Value::OptionalSelect(o)),)) }),
                        )
                    }
                }}

            </div>
        }
        .into_any(),
    }
}

fn render_label(label: Option<Label>) -> impl IntoView {
    view! { <CrudFieldLabelOpt label=label/> }
}
