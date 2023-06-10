use std::{borrow::Cow, collections::HashMap};

use crudkit_web::{prelude::*, DateTimeDisplay, JsonValue};
use leptonic::prelude::*;
use leptos::*;
use time::{
    format_description::well_known::Rfc3339, macros::format_description, PrimitiveDateTime,
};
use uuid::Uuid;

use crate::{
    crud_instance_config::{DynSelectConfig, SelectConfigTrait},
    prelude::CrudFieldLabel,
    ReactiveValue,
};

#[component]
pub fn CrudField<T>(
    cx: Scope,
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
) -> impl IntoView
where
    T: CrudDataTrait + 'static,
{
    move || {
        let id = format!("f{}", Uuid::new_v4().to_string());

        let field_clone = field.clone();

        let value_changed = SimpleCallback::new(move |result| match result {
            Ok(new) => value_changed.call((field_clone.clone(), Ok(new))),
            Err(err) => tracing::error!("Could not get input value: {}", err),
        });

        let field_config = field_config.with(|map| map.get(&field).cloned());
        let field_options = field_options.clone();

        match value {
            ReactiveValue::String(value) => {
                view! {cx, <CrudStringField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::Text(value) => {
                view! {cx, <CrudTextField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::Json(value) => {
                view! {cx, <CrudJsonField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::OptionalJson(value) => {
                view! {cx, <CrudOptionalJsonField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::UuidV4(value) => {
                view! {cx, <CrudUuidV4Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::UuidV7(value) => {
                view! {cx, <CrudUuidV7Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::U32(value) => {
                view! {cx, <CrudU32Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::OptionalU32(value) => {
                view! {cx, <CrudOptionalU32Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::I32(value) => {
                view! {cx, <CrudI32Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::OptionalI32(value) => {
                view! {cx, <CrudOptionalI32Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::I64(value) => {
                view! {cx, <CrudI64Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::OptionalI64(value) => {
                view! {cx, <CrudOptionalI64Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::F32(value) => {
                view! {cx, <CrudF32Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::Bool(value) => {
                view! {cx, <CrudBoolField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::ValidationStatus(_) => view! {cx, "TODO: Render ReactiveValue::ValidationStatus"}.into_view(cx),
            ReactiveValue::PrimitiveDateTime(value) => {
                view! {cx, <CrudPrimitiveDateTimeField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::OffsetDateTime(_) => view! {cx, "TODO: Render ReactiveValue::OffsetDateTime"}.into_view(cx),
            ReactiveValue::OptionalPrimitiveDateTime(value) => {
                view! {cx, <CrudOptionalPrimitiveDateTimeField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::OptionalOffsetDateTime(_) => view! {cx, "TODO: Render ReactiveValue::OptionalOffsetDateTime"}.into_view(cx),
            ReactiveValue::OneToOneRelation(_) => view! {cx, "TODO: Render ReactiveValue::OneToOneRelation"}.into_view(cx),
            ReactiveValue::NestedTable(_) => view! {cx, "TODO: Render ReactiveValue::NestedTable"}.into_view(cx),
            ReactiveValue::Custom(_) => view! {cx, "TODO: Render ReactiveValue::Custom"}.into_view(cx),
            ReactiveValue::Select(value) => {
                view! {cx, <CrudSelectField id=id.clone() field_config=field_config field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::Multiselect(_) => view! {cx, "TODO: Render ReactiveValue::Multiselect"}.into_view(cx),
            ReactiveValue::OptionalSelect(value) => {
                view! {cx, <CrudOptionalSelectField id=id.clone() field_config=field_config field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
            },
            ReactiveValue::OptionalMultiselect(_) => view! {cx, "TODO: Render ReactiveValue::OptionalMultiselect"}.into_view(cx),
        }
    }
}

#[component]
pub fn CrudStringField(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<String>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{ move || value.get() }</div>
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Text
                    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                    disabled=true
                    get=value
                    set=move |_| {}
                />
            </div>
        },
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Text
                    class="crud-input-field"
                    disabled=field_options.disabled
                    get=value
                    set=move |new| value_changed.call(Ok(Value::String(new)))
                />
            </div>
        },
    }
}

#[component]
pub fn CrudTextField(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<String>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{ move || value.get() }</div>
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                "TODO: Implement TipTap editor"
                // <CrudTipTapEditor
                //     api_base_url={ctx.props().api_base_url.clone()}
                //     id={self.format_id()}
                //     class={"crud-input-field"}
                //     value={value}
                //     disabled={true}
                // />
            </div>
        },
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                "TODO: Implement TipTap editor"
                // <CrudTipTapEditor
                //     api_base_url={ctx.props().api_base_url.clone()}
                //     id={self.format_id()}
                //     class={"crud-input-field"}
                //     value={value}
                //     onchange={ctx.link().callback(|input| Msg::Send(Value::Text(input)))}
                //     disabled={options.disabled}
                // />
            </div>
        },
    }
}

#[component]
pub fn CrudJsonField(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<JsonValue>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{ move || value.get().get_string_representation().to_owned() }</div>
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                "TODO: Implement TipTap editor or Json editor"
                // <CrudTipTapEditor
                //     api_base_url={ctx.props().api_base_url.clone()}
                //     id={self.format_id()}
                //     class={"crud-input-field"}
                //     value={value.get_string_representation().to_owned()}
                //     disabled={true}
                // />
            </div>
        },
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                "TODO: Implement TipTap editor or Json editor"
                // <CrudTipTapEditor
                //     api_base_url={ctx.props().api_base_url.clone()}
                //     id={self.format_id()}
                //     class={"crud-input-field"}
                //     value={value.get_string_representation().to_owned()}
                //     onchange={ctx.link().callback(|input| Msg::Send(Value::Text(input)))}
                //     disabled={options.disabled}
                // />
            </div>
        },
    }
}

#[component]
pub fn CrudOptionalJsonField(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<JsonValue>>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{ move || value.get().as_ref().map(|it| Cow::Owned(it.get_string_representation().to_owned())).unwrap_or(Cow::Borrowed("")) }</div>
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                "TODO: Implement TipTap editor or Json editor"
                // <CrudTipTapEditor
                //     api_base_url={ctx.props().api_base_url.clone()}
                //     id={self.format_id()}
                //     class={"crud-input-field"}
                //     value={value.as_ref().map(|it| it.get_string_representation().to_owned()).unwrap_or_default()}
                //     disabled={true}
                // />
            </div>
        },
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                "TODO: Implement TipTap editor or Json editor"
                // <CrudTipTapEditor
                //     api_base_url={ctx.props().api_base_url.clone()}
                //     id={self.format_id()}
                //     class={"crud-input-field"}
                //     value={value.as_ref().map(|it| it.get_string_representation().to_owned()).unwrap_or_default()}
                //     onchange={ctx.link().callback(|input| Msg::Send(Value::Text(input)))}
                //     disabled={options.disabled}
                // />
            </div>
        },
    }
}

#[component]
pub fn CrudUuidV4Field(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Uuid>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{ move || value.get().to_string() }</div>
        },
        // Never editable, TODO: why though? we could allow editing uuids if we can guarantee their validity.
        FieldMode::Readable | FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Text
                    class="crud-input-field"
                    disabled=true
                    get=MaybeSignal::derive(cx, move || { value.get().to_string() })
                    set=move |_| {}
                />
            </div>
        },
    }
}

#[component]
pub fn CrudUuidV7Field(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Uuid>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{ move || value.get().to_string() }</div>
        },
        // Never editable, TODO: why though? we could allow editing uuids if we can guarantee their validity.
        FieldMode::Readable | FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Text
                    class="crud-input-field"
                    disabled=true
                    get=MaybeSignal::derive(cx, move || { value.get().to_string() })
                    set=move |_| {}
                />
            </div>
        },
    }
}

#[component]
pub fn CrudU32Field(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<u32>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{ move || value.get() }</div>
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                    disabled=true
                    get=MaybeSignal::derive(cx, move || format!("{}", value.get()))
                    set=move |_| {}
                />
            </div>
        },
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field"
                    disabled=field_options.disabled
                    get=MaybeSignal::derive(cx, move || format!("{}", value.get()))
                    set=move |new| value_changed.call(match new.parse::<u32>() {
                        Ok(new) => Ok(Value::U32(new)),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        },
    }
}

#[component]
pub fn CrudOptionalU32Field(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u32>>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => { move || match value.get() {
            Some(value) => view! {cx,
                <div>{ value }</div>
            },
            None => view! {cx,
                <div>"-"</div>
            },
        }}.into_view(cx),
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                    disabled=true
                    get=MaybeSignal::derive(cx, move || format!("{}", value.get().unwrap_or_default()))
                    set=move |_| {}
                />
            </div>
        }.into_view(cx),
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field"
                    disabled=field_options.disabled
                    get=MaybeSignal::derive(cx, move || format!("{}", value.get().unwrap_or_default()))
                    set=move |new| value_changed.call(match new.parse::<u32>() {
                        Ok(new) => Ok(Value::OptionalU32(Some(new))),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        }.into_view(cx),
    }
}

#[component]
pub fn CrudI32Field(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<i32>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{ move || value.get() }</div>
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                    disabled=true
                    get=MaybeSignal::derive(cx, move || format!("{}", value.get()))
                    set=move |_| {}
                />
            </div>
        },
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field"
                    disabled=field_options.disabled
                    get=MaybeSignal::derive(cx, move || format!("{}", value.get()))
                    set=move |new| value_changed.call(match new.parse::<i32>() {
                        Ok(new) => Ok(Value::I32(new)),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        },
    }
}

#[component]
pub fn CrudOptionalI32Field(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i32>>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => { move || match value.get() {
            Some(value) => view! {cx,
                <div>{ value }</div>
            },
            None => view! {cx,
                <div>"-"</div>
            },
        }}.into_view(cx),
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                    disabled=true
                    get=MaybeSignal::derive(cx, move || format!("{}", value.get().unwrap_or_default()))
                    set=move |_| {}
                />
            </div>
        }.into_view(cx),
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field"
                    disabled=field_options.disabled
                    get=MaybeSignal::derive(cx, move || format!("{}", value.get().unwrap_or_default()))
                    set=move |new| value_changed.call(match new.parse::<i32>() {
                        Ok(new) => Ok(Value::OptionalI32(Some(new))),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        }.into_view(cx),
    }
}

#[component]
pub fn CrudI64Field(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<i64>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{ move || value.get() }</div>
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                    disabled=true
                    get=MaybeSignal::derive(cx, move || format!("{}", value.get()))
                    set=move |_| {}
                />
            </div>
        },
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field"
                    disabled=field_options.disabled
                    get=MaybeSignal::derive(cx, move || format!("{}", value.get()))
                    set=move |new| value_changed.call(match new.parse::<i64>() {
                        Ok(new) => Ok(Value::I64(new)),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        },
    }
}

#[component]
pub fn CrudOptionalI64Field(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i64>>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => { move || match value.get() {
            Some(value) => view! {cx,
                <div>{ value }</div>
            },
            None => view! {cx,
                <div>"-"</div>
            },
        }}.into_view(cx),
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                    disabled=true
                    get=MaybeSignal::derive(cx, move || format!("{}", value.get().unwrap_or_default()))
                    set=move |_| {}
                />
            </div>
        }.into_view(cx),
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field"
                    disabled=field_options.disabled
                    get=MaybeSignal::derive(cx, move || format!("{}", value.get().unwrap_or_default()))
                    set=move |new| value_changed.call(match new.parse::<i64>() {
                        Ok(new) => Ok(Value::OptionalI64(Some(new))),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        }.into_view(cx),
    }
}

#[component]
pub fn CrudF32Field(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<f32>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{ move || value.get() }</div>
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                    disabled=true
                    get=MaybeSignal::derive(cx, move || format!("{}", value.get()))
                    set=move |_| {}
                />
            </div>
        },
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field"
                    disabled=field_options.disabled
                    get=MaybeSignal::derive(cx, move || format!("{}", value.get()))
                    set=move |new| value_changed.call(match new.parse::<f32>() {
                        Ok(new) => Ok(Value::F32(new)),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        },
    }
}

#[component]
pub fn CrudBoolField(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<bool>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{ move || value.get() }</div>
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <div id=id.clone() class="crud-input-field">
                    <Toggle
                        state=value
                        on_toggle=move |_| {}
                        disabled=true
                    />
                </div>
            </div>
        },
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <div id=id.clone() class="crud-input-field">
                    <Toggle
                        state=value
                        on_toggle=move |new| value_changed.call(Ok(Value::Bool(new)))
                        disabled=field_options.disabled
                    />
                </div>
            </div>
        },
    }
}

#[component]
pub fn CrudPrimitiveDateTimeField(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<PrimitiveDateTime>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => match field_options.date_time_display {
            DateTimeDisplay::IsoUtc => view! {cx,
                <div>{ move || value.get().format(&Rfc3339).unwrap() }</div>
            },
            // TODO: Use icu4x formatting using the current users locale!
            DateTimeDisplay::LocalizedLocal => view! {cx,
                <div>{ move || value.get().format(format_description!("[day].[month].[year] [hour]:[minute]")).unwrap() }</div>
            },
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <div id=id.clone() class="crud-input-field">
                    <DateTimeInput
                        id=id.clone()
                        get=MaybeSignal::derive(cx, move || Some(value.get().assume_utc()))
                        set=move |_v| {}
                        disabled=true
                    />
                </div>
            </div>
        },
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <div id=id.clone() class="crud-input-field">
                    <DateTimeInput
                        id=id.clone()
                        get=MaybeSignal::derive(cx, move || Some(value.get().assume_utc()))
                        set=move |v| match v {
                            // TODO: We previously did this... `Value::OffsetDateTime(datetime.expect("Expected OffsetDateTime to not be None!"))`
                            Some(v) => value_changed.call(Ok(Value::PrimitiveDateTime(PrimitiveDateTime::new(v.date(), v.time())))),
                            None => {},
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
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<PrimitiveDateTime>>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => match field_options.date_time_display {
            DateTimeDisplay::IsoUtc => {move || match value.get() {
                Some(date_time) => view! {cx,
                    <div>{date_time.format(&Rfc3339).unwrap()}</div>
                },
                None => view! {cx,
                    <div>""</div>
                },
            }}.into_view(cx),
            DateTimeDisplay::LocalizedLocal => {move || match value.get() {
                // TODO: Use icu4x formatting using the current users locale!
                Some(date_time) => view! {cx,
                    <div>{date_time.format(format_description!("[day].[month].[year] [hour]:[minute]")).unwrap()}</div>
                },
                None => view! {cx,
                    <div>""</div>
                },
            }}.into_view(cx),
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                "TODO: DataTime input field"
                //<DateTime
                //    id=id.clone()
                //    ty=InputType::Number
                //    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                //    disabled=true
                //    get=format!("{value}")
                //    set=move |_| {}
                // />
            </div>
        }.into_view(cx),
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                "TODO: DataTime input field"
                { render_label(cx, field_options.label.clone()) }
                //<CrudOffsetDatetime
                //    id={self.format_id()}
                //    value={optional_primitive_date_time.clone().map(|it| it.assume_utc())}
                //    onchange={ctx.link().callback(|datetime: Option<time::OffsetDateTime>| Msg::Send(Value::OptionalOffsetDateTime(datetime)))}
                //    disabled={options.disabled}
                // />
            </div>
        }.into_view(cx),
    }
}

#[component]
pub fn CrudSelectField(
    cx: Scope,
    id: String,
    field_config: Option<Box<dyn SelectConfigTrait>>,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Box<dyn CrudSelectableTrait>>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => { move || format!("{:?}", value.get()) }.into_view(cx),
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                { match field_config {
                    None => view!{cx, <Alert variant=AlertVariant::Danger title=|_cx| "Config error">"Missing a field_config entry for this field."</Alert>}.into_view(cx),
                    Some(field_config) => field_config.render_select(cx, value, SimpleCallback::new(move |o| value_changed.call(Ok(Value::Select(o))))),
                } }
            </div>
        }
        .into_view(cx),
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                { match field_config {
                    None => view!{cx, <Alert variant=AlertVariant::Danger title=|_cx| "Config error">"Missing a field_config entry for this field."</Alert>}.into_view(cx),
                    Some(field_config) => field_config.render_select(cx, value, SimpleCallback::new(move |o| value_changed.call(Ok(Value::Select(o))))),
                } }
            </div>
        }
        .into_view(cx),
    }
}

#[component]
pub fn CrudOptionalSelectField(
    cx: Scope,
    id: String,
    field_config: Option<Box<dyn SelectConfigTrait>>,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<Box<dyn CrudSelectableTrait>>>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => { move || format!("{:?}", value.get()) }.into_view(cx),
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                { match field_config {
                    None => view!{cx, <Alert variant=AlertVariant::Danger title=|_cx| "Config error">"Missing a field_config entry for this field."</Alert>}.into_view(cx),
                    Some(field_config) => field_config.render_optional_select(cx, value, SimpleCallback::new(move |o| value_changed.call(Ok(Value::OptionalSelect(o))))),
                } }
            </div>
        }
        .into_view(cx),
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                { match field_config {
                    None => view!{cx, <Alert variant=AlertVariant::Danger title=|_cx| "Config error">"Missing a field_config entry for this field."</Alert>}.into_view(cx),
                    Some(field_config) => field_config.render_optional_select(cx, value, SimpleCallback::new(move |o| value_changed.call(Ok(Value::OptionalSelect(o))))),
                } }
            </div>
        }
        .into_view(cx),
    }
}

fn render_label(cx: Scope, label: Option<Label>) -> impl IntoView {
    match label {
        Some(label) => view! {cx, <CrudFieldLabel label=label.clone() />}.into_view(cx),
        None => ().into_view(cx),
    }
}
