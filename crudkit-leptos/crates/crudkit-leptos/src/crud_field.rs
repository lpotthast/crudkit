use std::{borrow::Cow, collections::HashMap, error::Error};

use crudkit_web::{prelude::*, DateTimeDisplay, JsonValue};
use leptonic::prelude::*;
use leptos::*;
use leptos_icons::BsIcon;
use time::{
    format_description::well_known::Rfc3339, macros::format_description, PrimitiveDateTime,
};
use uuid::Uuid;

use crate::{
    crud_field_label::CrudFieldLabelOpt,
    crud_instance_config::{DynSelectConfig, SelectConfigTrait},
    ReactiveValue,
};

#[component]
pub fn CrudField<T>(
    //children: Children,
    custom_fields: Signal<CustomFields<T, leptos::View>>,
    field_config: Signal<HashMap<T::Field, DynSelectConfig>>,
    api_base_url: Signal<String>,
    current_view: CrudSimpleView,
    field: T::Field,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: ReactiveValue,
    value_changed: Callback<(T::Field, Result<Value, String>)>, // how can we handle all possible types? serialization? TODO: Only take Value, not Result?; TODO: Use WriteSignal from ReactiveValue?
    entity: Signal<T>,
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
        value_changed: SimpleCallback<Result<Value, Box<dyn Error>>>,
        custom_field_renderer: Box<dyn Fn() -> View>,
    ) -> impl IntoView {
        match value {
            ReactiveValue::String(value) => {
                view! {
                    <CrudStringField
                        id=id.clone()
                        field_options=field_options
                        field_mode=field_mode
                        value=value
                        value_changed=value_changed
                    />
                }.into_view()
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
                }.into_view()
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
                }.into_view()
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
                }.into_view()
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
                }.into_view()
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
                }.into_view()
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
                }.into_view()
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
                }.into_view()
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
                }.into_view()
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
                }.into_view()
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
                }.into_view()
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
                }.into_view()
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
                }.into_view()
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
                }.into_view()
            },
            ReactiveValue::ValidationStatus(value) => {
                view! { <CrudValidationStatusField id=id.clone() field_options=field_options field_mode=field_mode value=value/> }.into_view()
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
                }.into_view()
            },
            ReactiveValue::OffsetDateTime(_) => view! { "TODO: Render ReactiveValue::OffsetDateTime" }.into_view(),
            ReactiveValue::OptionalPrimitiveDateTime(value) => {
                view! {
                    <CrudOptionalPrimitiveDateTimeField
                        id=id.clone()
                        field_options=field_options
                        field_mode=field_mode
                        value=value
                        value_changed=value_changed
                    />
                }.into_view()
            },
            ReactiveValue::OptionalOffsetDateTime(_) => view! { "TODO: Render ReactiveValue::OptionalOffsetDateTime" }.into_view(),
            ReactiveValue::OneToOneRelation(_) => view! { "TODO: Render ReactiveValue::OneToOneRelation" }.into_view(),
            ReactiveValue::Reference(_) => view! { "TODO: Render ReactiveValue::NestedTable" }.into_view(),
            ReactiveValue::Custom(_) => custom_field_renderer(),
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
                }.into_view()
            },
            ReactiveValue::Multiselect(_) => view! { "TODO: Render ReactiveValue::Multiselect" }.into_view(),
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
                }.into_view()
            },
            ReactiveValue::OptionalMultiselect(_) => view! { "TODO: Render ReactiveValue::OptionalMultiselect" }.into_view(),
        }
    }

    move || {
        let id: String = format!("f{}", Uuid::new_v4().to_string());

        let field_clone = field.clone();
        let field_clone3 = field.clone();

        let value_changed: SimpleCallback<Result<Value, Box<dyn Error>>> =
            create_simple_callback(move |result| match result {
                Ok(new) => value_changed.call((field_clone.clone(), Ok(new))),
                Err(err) => tracing::error!("Could not get input value: {}", err),
            });

        let field_config: Option<Box<dyn SelectConfigTrait>> =
            field_config.with(|map| map.get(&field).cloned());

        let field_options = field_options.clone();

        render_field(
            value,
            id,
            field_options.clone(),
            field_mode,
            field_config,
            value_changed,
            Box::new(move || {
                let field_clone3 = field_clone3.clone();
                let field_options = field_options.clone();
                custom_fields.with(|fields| match fields.get(&field_clone3) {
                    Some(custom_field) => {
                        let custom_field = custom_field.clone();
                        // Note (lukas): Not every custom child requires the entity data. The API and possibly performance would be nicer if `entity` was passed as a signal to the `render` function,
                        // but as the function is declared in the framework-agnostic crudkit-web crate, that change is not trivial...
                        (move || custom_field.render(&entity.get(), field_mode, field_options.clone())).into_view()
                    },
                    None => view! {
                        <Alert
                            variant=AlertVariant::Danger
                            title=create_callback(move |_| "Missing custom field declaration!")
                        >
                            {format!(
                                "The custom field '{:?}' should have been displayed here, but no renderer for that field was found in the `custom_*_fields` section of the static instance config. You might have forgotten to set the required HashMap entry.",
                                & field_clone3
                            )}

                        </Alert>
                    }.into_view(),
                })
            }),
        )
    }
}

#[component]
pub fn CrudStringField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<String>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> },
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TextInput
                    id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    class="crud-input-field"
                    disabled=true
                    get=value
                />
            </div>
        },
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TextInput
                    id=id.clone()
                    class="crud-input-field"
                    disabled=field_options.disabled
                    get=value
                    set=create_callback(move |new| value_changed.call(Ok(Value::String(new))))
                />
            </div>
        },
    }
}

#[component]
pub fn CrudTextField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<String>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> },
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TiptapEditor id=id.clone() class="crud-input-field" value=value disabled=true/>
            </div>
        },
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TiptapEditor
                    id=id.clone()
                    class="crud-input-field"
                    value=value
                    set_value=create_callback(move |new| {
                        value_changed
                            .call(
                                Ok(
                                    Value::Text(
                                        match new {
                                            TiptapContent::Html(content) => content,
                                            TiptapContent::Json(content) => content,
                                        },
                                    ),
                                ),
                            )
                    })

                    disabled=field_options.disabled
                />
            </div>
        },
    }
}

#[component]
pub fn CrudJsonField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<JsonValue>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get().get_string_representation().to_owned()}</div> },
        FieldMode::Readable => view! {
            <div class="crud-field">
                // TODO: Implement a proper Json editor
                {render_label(field_options.label.clone())}
                <TiptapEditor
                    id=id.clone()
                    class="crud-input-field"
                    value=Signal::derive(move || value.get().get_string_representation().to_owned())
                    disabled=true
                />
            </div>
        },
        FieldMode::Editable => view! {
            <div class="crud-field">
                // TODO: Implement a proper Json editor
                {render_label(field_options.label.clone())}
                <TiptapEditor
                    id=id.clone()
                    class="crud-input-field"
                    value=Signal::derive(move || value.get().get_string_representation().to_owned())
                    set_value=create_callback(move |new| {
                        value_changed
                            .call(
                                match new {
                                    TiptapContent::Html(content) => serde_json::from_str(&content),
                                    TiptapContent::Json(content) => serde_json::from_str(&content),
                                }
                                    .map(|json_value| Value::Json(JsonValue::new(json_value)))
                                    .map_err(|err| Box::new(err) as Box<dyn Error>),
                            );
                    })

                    disabled=field_options.disabled
                />
            </div>
        },
    }
}

#[component]
pub fn CrudOptionalJsonField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<JsonValue>>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
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
        },
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
        },
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
        },
    }
}

#[component]
pub fn CrudUuidV4Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Uuid>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get().to_string()}</div> },
        // Never editable, TODO: why though? we could allow editing uuids if we can guarantee their validity.
        FieldMode::Readable | FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TextInput
                    id=id.clone()
                    class="crud-input-field"
                    disabled=true
                    get=MaybeSignal::derive(move || { value.get().to_string() })
                />
            </div>
        },
    }
}

#[component]
pub fn CrudUuidV7Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Uuid>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get().to_string()}</div> },
        // Never editable, TODO: why though? we could allow editing uuids if we can guarantee their validity.
        FieldMode::Readable | FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TextInput
                    id=id.clone()
                    class="crud-input-field"
                    disabled=true
                    get=MaybeSignal::derive(move || { value.get().to_string() })
                />
            </div>
        },
    }
}

#[component]
pub fn CrudU32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<u32>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> },
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    class="crud-input-field"
                    disabled=true
                    get=MaybeSignal::derive(move || value.get() as f64)
                />
            </div>
        },
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    id=id.clone()
                    class="crud-input-field"
                    disabled=field_options.disabled
                    get=MaybeSignal::derive(move || value.get() as f64)
                    set=create_callback(move |new: f64| { value_changed.call(Ok(Value::U32(new as u32))) })
                />

            </div>
        },
    }
}

#[component]
pub fn CrudOptionalU32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u32>>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => { move || match value.get() {
            Some(value) => view! { <div>{value}</div> },
            None => view! { <div>"-"</div> },
        }}.into_view(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    class="crud-input-field"
                    disabled=true
                    get=MaybeSignal::derive(move || value.get().unwrap_or_default() as f64)
                />
            </div>
        }.into_view(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    id=id.clone()
                    class="crud-input-field"
                    disabled=field_options.disabled
                    get=MaybeSignal::derive(move || value.get().unwrap_or_default() as f64)
                    set=create_callback(move |new: f64| {
                        value_changed.call(Ok(Value::OptionalU32(Some(new as u32))))
                    })
                />

            </div>
        }.into_view(),
    }
}

#[component]
pub fn CrudI32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<i32>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> },
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    class="crud-input-field"
                    disabled=true
                    get=MaybeSignal::derive(move || value.get() as f64)
                />
            </div>
        },
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    id=id.clone()
                    class="crud-input-field"
                    disabled=field_options.disabled
                    get=MaybeSignal::derive(move || value.get() as f64)
                    set=create_callback(move |new: f64| { value_changed.call(Ok(Value::I32(new as i32))) })
                />

            </div>
        },
    }
}

#[component]
pub fn CrudOptionalI32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i32>>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => { move || match value.get() {
            Some(value) => view! { <div>{value}</div> },
            None => view! { <div>"-"</div> },
        }}.into_view(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    class="crud-input-field"
                    disabled=true
                    get=MaybeSignal::derive(move || value.get().unwrap_or_default() as f64)
                />
            </div>
        }.into_view(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    id=id.clone()
                    class="crud-input-field"
                    disabled=field_options.disabled
                    get=MaybeSignal::derive(move || value.get().unwrap_or_default() as f64)
                    set=create_callback(move |new: f64| {
                        value_changed.call(Ok(Value::OptionalI32(Some(new as i32))))
                    })
                />

            </div>
        }.into_view(),
    }
}

#[component]
pub fn CrudI64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<i64>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> },
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    class="crud-input-field"
                    disabled=true
                    get=MaybeSignal::derive(move || value.get() as f64)
                />
            </div>
        },
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    id=id.clone()
                    class="crud-input-field"
                    disabled=field_options.disabled
                    get=MaybeSignal::derive(move || value.get() as f64)
                    set=create_callback(move |new: f64| { value_changed.call(Ok(Value::I64(new as i64))) })
                />

            </div>
        },
    }
}

#[component]
pub fn CrudOptionalI64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i64>>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => { move || match value.get() {
            Some(value) => view! { <div>{value}</div> },
            None => view! { <div>"-"</div> },
        }}.into_view(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    class="crud-input-field"
                    disabled=true
                    get=MaybeSignal::derive(move || value.get().unwrap_or_default() as f64)
                />
            </div>
        }.into_view(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    id=id.clone()
                    class="crud-input-field"
                    disabled=field_options.disabled
                    get=MaybeSignal::derive(move || value.get().unwrap_or_default() as f64)
                    set=create_callback(move |new: f64| {
                        value_changed.call(Ok(Value::OptionalI64(Some(new as i64))))
                    })
                />

            </div>
        }.into_view(),
    }
}

#[component]
pub fn CrudF32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<f32>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> },
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    class="crud-input-field"
                    disabled=true
                    get=MaybeSignal::derive(move || value.get() as f64)
                />
            </div>
        },
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    id=id.clone()
                    class="crud-input-field"
                    disabled=field_options.disabled
                    get=MaybeSignal::derive(move || value.get() as f64)
                    set=create_callback(move |new: f64| { value_changed.call(Ok(Value::F32(new as f32))) })
                />

            </div>
        },
    }
}

#[component]
pub fn CrudBoolField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<bool>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> },
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
                    <Toggle state=value disabled=true/>
                </div>
            </div>
        },
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
                    <Toggle
                        state=value
                        set_state=create_callback(move |new| { value_changed.call(Ok(Value::Bool(new))) })

                        disabled=field_options.disabled
                    />
                </div>
            </div>
        },
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
                    true => view! { <Icon icon=BsIcon::BsExclamationTriangleFill/> },
                    false => view! { <Icon icon=BsIcon::BsCheck/> },
                }}

            </div>
        },
        FieldMode::Readable | FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
                    {move || match value.get() {
                        true => view! { <Icon icon=BsIcon::BsExclamationTriangleFill/> },
                        false => view! { <Icon icon=BsIcon::BsCheck/> },
                    }}

                </div>
            </div>
        },
    }
}

#[component]
pub fn CrudPrimitiveDateTimeField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<PrimitiveDateTime>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => match field_options.date_time_display {
            DateTimeDisplay::IsoUtc => view! { <div>{move || value.get().format(&Rfc3339).unwrap()}</div> },
            // TODO: Use icu4x formatting using the current users locale!
            DateTimeDisplay::LocalizedLocal => view! {
                <div>
                    {move || {
                        value.get().format(format_description!("[day].[month].[year] [hour]:[minute]")).unwrap()
                    }}

                </div>
            },
        },
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
                    <DateTimeInput
                        id=id.clone()
                        get=MaybeSignal::derive(move || Some(value.get().assume_utc()))
                        set=move |_v| {}
                        disabled=true
                    />
                </div>
            </div>
        },
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
                    <DateTimeInput
                        id=id.clone()
                        get=MaybeSignal::derive(move || Some(value.get().assume_utc()))
                        set=move |v| match v {
                            Some(v) => {
                                value_changed
                                    .call(Ok(Value::PrimitiveDateTime(PrimitiveDateTime::new(v.date(), v.time()))))
                            }
                            None => {}
                        }

                        disabled=field_options.disabled
                    />
                </div>
            </div>
        },
    }
}

#[component]
pub fn CrudOptionalPrimitiveDateTimeField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<PrimitiveDateTime>>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => match field_options.date_time_display {
            DateTimeDisplay::IsoUtc => {move || match value.get() {
                Some(date_time) => view! { <div>{date_time.format(&Rfc3339).unwrap()}</div> },
                None => view! { <div>""</div> },
            }}.into_view(),
            DateTimeDisplay::LocalizedLocal => {move || match value.get() {
                // TODO: Use icu4x formatting using the current users locale!
                Some(date_time) => view! { <div>{date_time.format(format_description!("[day].[month].[year] [hour]:[minute]")).unwrap()}</div> },
                None => view! { <div>""</div> },
            }}.into_view(),
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
        }.into_view(),
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
        }.into_view(),
    }
}

#[component]
pub fn CrudSelectField(
    id: String,
    field_config: Option<Box<dyn SelectConfigTrait>>,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Box<dyn CrudSelectableTrait>>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => { move || format!("{:?}", value.get()) }.into_view(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                {match field_config {
                    None => {
                        view! {
                            <Alert variant=AlertVariant::Danger title=create_callback(|_cx| "Config error")>
                                "Missing a field_config entry for this field."
                            </Alert>
                        }
                            .into_view()
                    }
                    Some(field_config) => {
                        field_config
                            .render_select(
                                value,
                                create_simple_callback(move |o| { value_changed.call(Ok(Value::Select(o))) }),
                            )
                    }
                }}

            </div>
        }
        .into_view(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                {match field_config {
                    None => {
                        view! {
                            <Alert variant=AlertVariant::Danger title=create_callback(|_cx| "Config error")>
                                "Missing a field_config entry for this field."
                            </Alert>
                        }
                            .into_view()
                    }
                    Some(field_config) => {
                        field_config
                            .render_select(
                                value,
                                create_simple_callback(move |o| { value_changed.call(Ok(Value::Select(o))) }),
                            )
                    }
                }}

            </div>
        }
        .into_view(),
    }
}

#[component]
pub fn CrudOptionalSelectField(
    id: String,
    field_config: Option<Box<dyn SelectConfigTrait>>,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<Box<dyn CrudSelectableTrait>>>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => { move || format!("{:?}", value.get()) }.into_view(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                {match field_config {
                    None => {
                        view! {
                            <Alert variant=AlertVariant::Danger title=create_callback(|_cx| "Config error")>
                                "Missing a field_config entry for this field."
                            </Alert>
                        }
                            .into_view()
                    }
                    Some(field_config) => {
                        field_config
                            .render_optional_select(
                                value,
                                create_simple_callback(move |o| { value_changed.call(Ok(Value::OptionalSelect(o))) }),
                            )
                    }
                }}

            </div>
        }
        .into_view(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                {match field_config {
                    None => {
                        view! {
                            <Alert variant=AlertVariant::Danger title=create_callback(|_cx| "Config error")>
                                "Missing a field_config entry for this field."
                            </Alert>
                        }
                            .into_view()
                    }
                    Some(field_config) => {
                        field_config
                            .render_optional_select(
                                value,
                                create_simple_callback(move |o| { value_changed.call(Ok(Value::OptionalSelect(o))) }),
                            )
                    }
                }}

            </div>
        }
        .into_view(),
    }
}

fn render_label(label: Option<Label>) -> impl IntoView {
    view! { <CrudFieldLabelOpt label=label/> }
}
