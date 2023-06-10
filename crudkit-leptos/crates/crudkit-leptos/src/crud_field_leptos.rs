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
     entity: Signal<T>,
    value_changed: Callback<(T::Field, Result<Value, String>)>, // how can we handle all possible types? serialization? TODO: Only take Value, not Result
) -> impl IntoView
where
    T: CrudDataTrait + 'static,
{
    // NOTE: A field is re-rendered when the "entity" signal updates, as the fields is dependant on the current value inside the entity.
    // TODO: This is not fine-grained. When one value changes, all fields are re-rendered... We should use a Map<Field, Signal<Value>> and render fields fine-grained!!
    move || {
        let id = format!("f{}", Uuid::new_v4().to_string());

        let field_clone = field.clone();

        let value_changed = SimpleCallback::new(move |result| match result {
            Ok(new) => value_changed.call((field_clone.clone(), Ok(new))),
            Err(err) => tracing::error!("Could not get input value: {}", err),
        });

        let field_config = field_config.with(|map| map.get(&field).cloned());

        entity.with(|entity| {
            let field_options = field_options.clone();
            match field.get_value(entity) {
                Value::String(value) => {
                    view! {cx, <CrudStringField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::Text(value) => {
                    view! {cx, <CrudTextField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::Json(value) => {
                    view! {cx, <CrudJsonField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::OptionalJson(value) => {
                    view! {cx, <CrudOptionalJsonField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::UuidV4(value) => {
                    view! {cx, <CrudUuidV4Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::UuidV7(value) => {
                    view! {cx, <CrudUuidV7Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::U32(value) => {
                    view! {cx, <CrudU32Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::OptionalU32(value) => {
                    view! {cx, <CrudOptionalU32Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::I32(value) => {
                    view! {cx, <CrudI32Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::OptionalI32(value) => {
                    view! {cx, <CrudOptionalI32Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::I64(value) => {
                    view! {cx, <CrudI64Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::OptionalI64(value) => {
                    view! {cx, <CrudOptionalI64Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::F32(value) => {
                    view! {cx, <CrudF32Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::Bool(value) => {
                    view! {cx, <CrudBoolField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::ValidationStatus(_) => view! {cx, "TODO: Render Value::ValidationStatus"}.into_view(cx),
                Value::PrimitiveDateTime(value) => {
                    view! {cx, <CrudPrimitiveDateTimeField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::OffsetDateTime(_) => view! {cx, "TODO: Render Value::OffsetDateTime"}.into_view(cx),
                Value::OptionalPrimitiveDateTime(value) => {
                    view! {cx, <CrudOptionalPrimitiveDateTimeField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::OptionalOffsetDateTime(_) => view! {cx, "TODO: Render Value::OptionalOffsetDateTime"}.into_view(cx),
                Value::OneToOneRelation(_) => view! {cx, "TODO: Render Value::OneToOneRelation"}.into_view(cx),
                Value::NestedTable(_) => view! {cx, "TODO: Render Value::NestedTable"}.into_view(cx),
                Value::Custom(_) => view! {cx, "TODO: Render Value::Custom"}.into_view(cx),
                Value::Select(value) => {
                    view! {cx, <CrudSelectField id=id.clone() field_config=field_config field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::Multiselect(_) => view! {cx, "TODO: Render Value::Multiselect"}.into_view(cx),
                Value::OptionalSelect(value) => {
                    view! {cx, <CrudOptionalSelectField id=id.clone() field_config=field_config field_options=field_options field_mode=field_mode value=value value_changed=value_changed/>}.into_view(cx)
                },
                Value::OptionalMultiselect(_) => view! {cx, "TODO: Render Value::OptionalMultiselect"}.into_view(cx),
            }
        })
    }
}

#[component]
pub fn CrudStringField(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: String,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{value.clone()}</div>
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Text
                    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                    disabled=true
                    get=value.clone()
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
                    get=value.clone()
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
    value: String,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{value.clone()}</div>
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
    value: JsonValue,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{value.get_string_representation().to_owned()}</div>
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
    value: Option<JsonValue>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{value.as_ref().map(|it| Cow::Owned(it.get_string_representation().to_owned())).unwrap_or(Cow::Borrowed(""))}</div>
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
    value: Uuid,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{value.to_string()}</div>
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
                    get=value.to_string()
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
    value: Uuid,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{value.to_string()}</div>
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
                    get=value.to_string()
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
    value: u32,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{value}</div>
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                    disabled=true
                    get=format!("{value}")
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
                    get=format!("{value}")
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
    value: Option<u32>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => match value {
            Some(value) => view! {cx,
                <div>{value}</div>
            },
            None => view! {cx,
                <div>"-"</div>
            },
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                    disabled=true
                    get=format!("{}", value.unwrap_or_default())
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
                    get=format!("{}", value.unwrap_or_default())
                    set=move |new| value_changed.call(match new.parse::<u32>() {
                        Ok(new) => Ok(Value::OptionalU32(Some(new))),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        },
    }
}

#[component]
pub fn CrudI32Field(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: i32,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{value}</div>
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                    disabled=true
                    get=format!("{value}")
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
                    get=format!("{value}")
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
    value: Option<i32>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => match value {
            Some(value) => view! {cx,
                <div>{value}</div>
            },
            None => view! {cx,
                <div>"-"</div>
            },
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                    disabled=true
                    get=format!("{}", value.unwrap_or_default())
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
                    get=format!("{}", value.unwrap_or_default())
                    set=move |new| value_changed.call(match new.parse::<i32>() {
                        Ok(new) => Ok(Value::OptionalI32(Some(new))),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        },
    }
}

#[component]
pub fn CrudI64Field(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: i64,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{value}</div>
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                    disabled=true
                    get=format!("{value}")
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
                    get=format!("{value}")
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
    value: Option<i64>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => match value {
            Some(value) => view! {cx,
                <div>{value}</div>
            },
            None => view! {cx,
                <div>"-"</div>
            },
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                    disabled=true
                    get=format!("{}", value.unwrap_or_default())
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
                    get=format!("{}", value.unwrap_or_default())
                    set=move |new| value_changed.call(match new.parse::<i64>() {
                        Ok(new) => Ok(Value::OptionalI64(Some(new))),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        },
    }
}

#[component]
pub fn CrudF32Field(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: f32,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{value}</div>
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <Input
                    id=id.clone()
                    ty=InputType::Number
                    class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
                    disabled=true
                    get=format!("{value}")
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
                    get=format!("{value}")
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
    value: bool,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {cx,
            <div>{value}</div>
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
    value: PrimitiveDateTime,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => match field_options.date_time_display {
            DateTimeDisplay::IsoUtc => view! {cx,
                <div>{ value.format(&Rfc3339).unwrap() }</div>
            },
            // TODO: Use icu4x formatting using the current users locale!
            DateTimeDisplay::LocalizedLocal => view! {cx,
                <div>{ value.format(format_description!("[day].[month].[year] [hour]:[minute]")).unwrap() }</div>
            },
        },
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                <div id=id.clone() class="crud-input-field">
                    <DateTimeInput
                        id=id.clone()
                        get=Some(value.clone().assume_utc())
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
                        get=Some(value.clone().assume_utc())
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
    value: Option<PrimitiveDateTime>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => match field_options.date_time_display {
            DateTimeDisplay::IsoUtc => match value {
                Some(date_time) => view! {cx,
                    <div>{date_time.format(&Rfc3339).unwrap()}</div>
                },
                None => view! {cx,
                    <div>""</div>
                },
            },
            DateTimeDisplay::LocalizedLocal => match value {
                // TODO: Use icu4x formatting using the current users locale!
                Some(date_time) => view! {cx,
                    <div>{date_time.format(format_description!("[day].[month].[year] [hour]:[minute]")).unwrap()}</div>
                },
                None => view! {cx,
                    <div>""</div>
                },
            },
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
        },
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
        },
    }
}

#[component]
pub fn CrudSelectField(
    cx: Scope,
    id: String,
    field_config: Option<Box<dyn SelectConfigTrait>>,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: Box<dyn CrudSelectableTrait>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => format!("{value:?}").into_view(cx),
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                { match field_config {
                    None => view!{cx, <Alert variant=AlertVariant::Danger title=|_cx| "Config error">"Missing a field_config entry for this field."</Alert>}.into_view(cx),
                    Some(field_config) => field_config.render_select(cx, Signal::derive(cx, move || value.clone()), SimpleCallback::new(move |o| value_changed.call(Ok(Value::Select(o))))),
                } }
            </div>
        }
        .into_view(cx),
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                { match field_config {
                    None => view!{cx, <Alert variant=AlertVariant::Danger title=|_cx| "Config error">"Missing a field_config entry for this field."</Alert>}.into_view(cx),
                    Some(field_config) => field_config.render_select(cx, Signal::derive(cx, move || value.clone()), SimpleCallback::new(move |o| value_changed.call(Ok(Value::Select(o))))),
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
    value: Option<Box<dyn CrudSelectableTrait>>,
    value_changed: SimpleCallback<Result<Value, Box<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => format!("{value:?}").into_view(cx),
        FieldMode::Readable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                { match field_config {
                    None => view!{cx, <Alert variant=AlertVariant::Danger title=|_cx| "Config error">"Missing a field_config entry for this field."</Alert>}.into_view(cx),
                    Some(field_config) => field_config.render_optional_select(cx, Signal::derive(cx, move || value.clone()), SimpleCallback::new(move |o| value_changed.call(Ok(Value::OptionalSelect(o))))),
                } }
            </div>
        }
        .into_view(cx),
        FieldMode::Editable => view! {cx,
            <div class="crud-field">
                { render_label(cx, field_options.label.clone()) }
                { match field_config {
                    None => view!{cx, <Alert variant=AlertVariant::Danger title=|_cx| "Config error">"Missing a field_config entry for this field."</Alert>}.into_view(cx),
                    Some(field_config) => field_config.render_optional_select(cx, Signal::derive(cx, move || value.clone()), SimpleCallback::new(move |o| value_changed.call(Ok(Value::OptionalSelect(o))))),
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

// <T: CrudDataTrait + 'static>
fn render_select_child(cx: Scope) -> impl IntoView {
    view! {cx, "TODO: Implement render_select_child" }
    //ctx.props().children.iter().find(|child| {
    //    match child {
    //        Item::NestedInstance(_) => false,
    //        Item::Relation(_) => false,
    //        Item::Select(select) => select.props.name == ctx.props().field_type.get_name(),
    //    }
    //}).map_or(html! {
    //    <CrudAlert variant={crate::crud_alert::Variant::Danger}>
    //        {"Could not find required 'Select' child. Help: You might be missing the <CrudSelectField> in your instance markup."}
    //    </CrudAlert>
    //}, |it| it.into() )
}
