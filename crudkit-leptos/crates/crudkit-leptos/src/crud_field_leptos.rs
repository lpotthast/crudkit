use std::borrow::Cow;

use crudkit_web::{prelude::*, DateTimeDisplay, JsonValue};
use leptonic::prelude::*;
use leptos::*;
use time::{
    format_description::well_known::Rfc3339, macros::format_description, PrimitiveDateTime,
};
use uuid::Uuid;

use crate::prelude::CrudFieldLabel;

#[component]
pub fn CrudFieldL<T, C>(
    cx: Scope,
    //children: Children,
    custom_fields: Signal<CustomFields<T, leptos::View>>,
    api_base_url: Signal<String>,
    current_view: CrudSimpleView,
    field: T::Field,
    field_options: FieldOptions,
    field_mode: FieldMode,
    entity: T,
    value_changed: C, // how can we handle all possible types? serialization? TODO: Only take Value, not Result
) -> impl IntoView
where
    T: CrudDataTrait + 'static,
    C: Fn(T::Field, Result<Value, String>) -> () + 'static,
{
    let id = format!("f{}", Uuid::new_v4().to_string());

    let field_clone = field.clone();
    let set_value = move |result: Result<Value, Box<dyn std::error::Error>>| match result {
        Ok(new) => value_changed(field_clone.clone(), Ok(new)),
        Err(err) => tracing::error!("Could not get input value: {}", err),
    };

    match field.get_value(&entity) {
        Value::String(value) => {
            view! {cx, <CrudStringField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=set_value/>}.into_view(cx)
        },
        Value::Text(value) => {
            view! {cx, <CrudTextField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=set_value/>}.into_view(cx)
        },
        Value::Json(value) => {
            view! {cx, <CrudJsonField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=set_value/>}.into_view(cx)
        },
        Value::OptionalJson(value) => {
            view! {cx, <CrudOptionalJsonField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=set_value/>}.into_view(cx)
        },
        Value::UuidV4(value) => {
            view! {cx, <CrudUuidV4Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=set_value/>}.into_view(cx)
        },
        Value::UuidV7(value) => {
            view! {cx, <CrudUuidV7Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=set_value/>}.into_view(cx)
        },
        Value::U32(value) => {
            view! {cx, <CrudU32Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=set_value/>}.into_view(cx)
        },
        Value::OptionalU32(value) => {
            view! {cx, <CrudOptionalU32Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=set_value/>}.into_view(cx)
        },
        Value::I32(value) => {
            view! {cx, <CrudI32Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=set_value/>}.into_view(cx)
        },
        Value::OptionalI32(value) => {
            view! {cx, <CrudOptionalI32Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=set_value/>}.into_view(cx)
        },
        Value::I64(value) => {
            view! {cx, <CrudI64Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=set_value/>}.into_view(cx)
        },
        Value::OptionalI64(value) => {
            view! {cx, <CrudOptionalI64Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=set_value/>}.into_view(cx)
        },
        Value::F32(value) => {
            view! {cx, <CrudF32Field id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=set_value/>}.into_view(cx)
        },
        Value::Bool(value) => {
            view! {cx, <CrudBoolField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=set_value/>}.into_view(cx)
        },
        Value::ValidationStatus(_) => todo!(),
        Value::PrimitiveDateTime(_) => todo!(),
        Value::OffsetDateTime(_) => todo!(),
        Value::OptionalPrimitiveDateTime(value) => {
            view! {cx, <CrudOptionalPrimitiveDateTimeField id=id.clone() field_options=field_options field_mode=field_mode value=value value_changed=set_value/>}.into_view(cx)
        },
        Value::OptionalOffsetDateTime(_) => todo!(),
        Value::OneToOneRelation(_) => todo!(),
        Value::NestedTable(_) => todo!(),
        Value::Custom(_) => todo!(),
        Value::Select(_) => todo!(),
        Value::Multiselect(_) => todo!(),
        Value::OptionalSelect(_) => todo!(),
        Value::OptionalMultiselect(_) => todo!(),
    }
}

#[component]
pub fn CrudStringField<C>(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: String,
    value_changed: C, // how can we handle all possible types? serialization?
) -> impl IntoView
where
    C: Fn(Result<Value, Box<dyn std::error::Error>>) -> () + 'static,
{
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
                    set=move |new| value_changed(Ok(Value::String(new)))
                />
            </div>
        },
    }
}

#[component]
pub fn CrudTextField<C>(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: String,
    value_changed: C, // how can we handle all possible types? serialization?
) -> impl IntoView
where
    C: Fn(Result<Value, Box<dyn std::error::Error>>) -> () + 'static,
{
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
pub fn CrudJsonField<C>(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: JsonValue,
    value_changed: C, // how can we handle all possible types? serialization?
) -> impl IntoView
where
    C: Fn(Result<Value, Box<dyn std::error::Error>>) -> () + 'static,
{
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
pub fn CrudOptionalJsonField<C>(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: Option<JsonValue>,
    value_changed: C, // how can we handle all possible types? serialization?
) -> impl IntoView
where
    C: Fn(Result<Value, Box<dyn std::error::Error>>) -> () + 'static,
{
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
pub fn CrudUuidV4Field<C>(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: Uuid,
    value_changed: C, // how can we handle all possible types? serialization?
) -> impl IntoView
where
    C: Fn(Result<Value, Box<dyn std::error::Error>>) -> () + 'static,
{
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
pub fn CrudUuidV7Field<C>(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: Uuid,
    value_changed: C, // how can we handle all possible types? serialization?
) -> impl IntoView
where
    C: Fn(Result<Value, Box<dyn std::error::Error>>) -> () + 'static,
{
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
pub fn CrudU32Field<C>(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: u32,
    value_changed: C, // how can we handle all possible types? serialization?
) -> impl IntoView
where
    C: Fn(Result<Value, Box<dyn std::error::Error>>) -> () + 'static,
{
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
                    set=move |new| value_changed(match new.parse::<u32>() {
                        Ok(new) => Ok(Value::U32(new)),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        },
    }
}

#[component]
pub fn CrudOptionalU32Field<C>(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: Option<u32>,
    value_changed: C, // how can we handle all possible types? serialization?
) -> impl IntoView
where
    C: Fn(Result<Value, Box<dyn std::error::Error>>) -> () + 'static,
{
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
                    set=move |new| value_changed(match new.parse::<u32>() {
                        Ok(new) => Ok(Value::OptionalU32(Some(new))),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        },
    }
}

#[component]
pub fn CrudI32Field<C>(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: i32,
    value_changed: C, // how can we handle all possible types? serialization?
) -> impl IntoView
where
    C: Fn(Result<Value, Box<dyn std::error::Error>>) -> () + 'static,
{
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
                    set=move |new| value_changed(match new.parse::<i32>() {
                        Ok(new) => Ok(Value::I32(new)),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        },
    }
}

#[component]
pub fn CrudOptionalI32Field<C>(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: Option<i32>,
    value_changed: C, // how can we handle all possible types? serialization?
) -> impl IntoView
where
    C: Fn(Result<Value, Box<dyn std::error::Error>>) -> () + 'static,
{
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
                    set=move |new| value_changed(match new.parse::<i32>() {
                        Ok(new) => Ok(Value::OptionalI32(Some(new))),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        },
    }
}

#[component]
pub fn CrudI64Field<C>(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: i64,
    value_changed: C, // how can we handle all possible types? serialization?
) -> impl IntoView
where
    C: Fn(Result<Value, Box<dyn std::error::Error>>) -> () + 'static,
{
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
                    set=move |new| value_changed(match new.parse::<i64>() {
                        Ok(new) => Ok(Value::I64(new)),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        },
    }
}

#[component]
pub fn CrudOptionalI64Field<C>(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: Option<i64>,
    value_changed: C, // how can we handle all possible types? serialization?
) -> impl IntoView
where
    C: Fn(Result<Value, Box<dyn std::error::Error>>) -> () + 'static,
{
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
                    set=move |new| value_changed(match new.parse::<i64>() {
                        Ok(new) => Ok(Value::OptionalI64(Some(new))),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        },
    }
}

#[component]
pub fn CrudF32Field<C>(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: f32,
    value_changed: C, // how can we handle all possible types? serialization?
) -> impl IntoView
where
    C: Fn(Result<Value, Box<dyn std::error::Error>>) -> () + 'static,
{
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
                    set=move |new| value_changed(match new.parse::<f32>() {
                        Ok(new) => Ok(Value::F32(new)),
                        Err(err) =>Err(err.into()),
                    })
                />
            </div>
        },
    }
}

#[component]
pub fn CrudBoolField<C>(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: bool,
    value_changed: C, // how can we handle all possible types? serialization?
) -> impl IntoView
where
    C: Fn(Result<Value, Box<dyn std::error::Error>>) -> () + 'static,
{
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
                        on_toggle=move |new| value_changed(Ok(Value::Bool(new)))
                        disabled=field_options.disabled
                    />
                </div>
            </div>
        },
    }
}

#[component]
pub fn CrudOptionalPrimitiveDateTimeField<C>(
    cx: Scope,
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    value: Option<PrimitiveDateTime>,
    value_changed: C, // how can we handle all possible types? serialization?
) -> impl IntoView
where
    C: Fn(Result<Value, Box<dyn std::error::Error>>) -> () + 'static,
{
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

fn render_label(cx: Scope, label: Option<Label>) -> impl IntoView {
    match label {
        Some(label) => view! {cx, <CrudFieldLabel label=label.clone() />}.into_view(cx),
        None => ().into_view(cx),
    }
}
