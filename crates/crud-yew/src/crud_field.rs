use std::marker::PhantomData;

use crate::{
    crud_instance::Item, event_target_as, keyboard_event_target_as, DateTimeDisplay, types::custom_field::CustomFields,
};

use super::prelude::*;
use chrono_utc_date_time::UtcDateTime;
use tracing::error;
use uuid::Uuid;
use yew::{html::ChildrenRenderer, prelude::*};
use yewbi::Bi;

// TODO: Extract fields to separate components

pub enum Msg {
    Send(Value),
    // TODO: Add SendErr variant...?
    LogInputRetrievalErr(Box<dyn std::error::Error>),
}

#[derive(Properties, PartialEq)]
pub struct Props<T: CrudDataTrait> {
    pub children: ChildrenRenderer<Item>,
    pub custom_fields: CustomFields<T>,
    pub api_base_url: String,
    pub current_view: CrudSimpleView,
    pub field_type: T::Field,
    pub field_options: FieldOptions,
    pub field_mode: FieldMode,
    // TODO: Must not OWN the complete entity!
    pub entity: Option<T>,
    pub value_changed: Callback<(T::Field, Result<Value, String>)>, // how can we handle all possible types? serialization?
}

pub struct CrudField<T> {
    phantom_data: PhantomData<T>,
    entity: T,
    id: Uuid,
}

impl<T: CrudDataTrait> CrudField<T> {
    fn format_id(&self) -> String {
        format!("f{}", self.id.to_string())
    }
}

impl<T: 'static + CrudDataTrait> Component for CrudField<T> {
    type Message = Msg;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            phantom_data: PhantomData {},
            entity: ctx
                .props()
                .entity
                .clone()
                .unwrap_or_else(|| Default::default()),
            id: Uuid::new_v4(),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.entity = ctx
            .props()
            .entity
            .clone()
            .unwrap_or_else(|| Default::default());
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Send(value) => {
                ctx.props()
                    .value_changed
                    .emit((ctx.props().field_type.clone(), Ok(value)));
                false
            }
            Msg::LogInputRetrievalErr(err) => {
                error!("Could not get input value: {}", err);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let options = &ctx.props().field_options;
        html! {
            match ctx.props().field_type.get_value(&self.entity) {
                // Value::Error => html! {
                //     "Field is in an error sate. This should not be shown!"
                // },
                Value::OneToOneRelation(_referenced_id) => match &ctx.props().field_mode {
                    FieldMode::Display => html! {
                        <div>{"FieldMode::Display wird von Feldern des Typs OneToOneRelation aktuell nicht unterstützt."}</div>
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            { ctx.props().children.iter().find(|child| {
                                match child {
                                    Item::NestedInstance(_) => false,
                                    Item::Relation(related_field) => related_field.props.name == ctx.props().field_type.get_name(),
                                    Item::Select(_) => false,
                                }
                            }).map_or(html! { "foo" }, |it| { let html: Html = it.into(); html }) }
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            { ctx.props().children.iter().find(|child| {
                                match child {
                                    Item::NestedInstance(_) => false,
                                    Item::Relation(related_field) => related_field.props.name == ctx.props().field_type.get_name(),
                                    Item::Select(_) => false,
                                }
                            }).map_or(html! { "foo" }, |it| { let html: Html = it.into(); html }) }
                        </div>
                    },
                },
                Value::NestedTable(_referenced_id) => {
                    match ctx.props().current_view {
                        CrudSimpleView::List => html! {
                            <div>{"Felder des Typs NestedTable können aktuell nicht in der Listenansicht dargestellt werden."}</div>
                        },
                        CrudSimpleView::Create => html! {
                            <>
                                if let Some(label) = &options.label {
                                    <CrudFieldLabel label={label.clone()} />
                                }
                                <div>{"Diese Informationen können erst bearbeitet werden, nachdem der Eintrag gespeichert wurde."}</div>
                            </>
                        },
                        CrudSimpleView::Read | CrudSimpleView::Edit => match &ctx.props().field_mode {
                            FieldMode::Display => html! {
                                <div>{"FieldMode::Display wird von Feldern des Typs NestedTable aktuell nicht unterstützt."}</div>
                            },
                            FieldMode::Readable => html! {
                                <div class="crud-field">
                                    if let Some(label) = &options.label {
                                        <CrudFieldLabel label={label.clone()} />
                                    }
                                    { ctx.props().children.iter().filter(|child| {
                                        match child {
                                            Item::NestedInstance(nested_instance) => nested_instance.props.name == ctx.props().field_type.get_name(),
                                            Item::Relation(_) => false,
                                            Item::Select(_) => false,
                                        }
                                    }).collect::<Html>() }
                                </div>
                            },
                            FieldMode::Editable => html! {
                                <div class="crud-field">
                                    if let Some(label) = &options.label {
                                        <CrudFieldLabel label={label.clone()} />
                                    }
                                    { ctx.props().children.iter().filter(|child| {
                                        match child {
                                            Item::NestedInstance(nested_instance) => nested_instance.props.name == ctx.props().field_type.get_name(),
                                            Item::Relation(_) => false,
                                            Item::Select(_) => false,
                                        }
                                    }).collect::<Html>() }
                                </div>
                            },
                        },
                    }
                },
                Value::Custom(_) => { match ctx.props().custom_fields
                    .get(&ctx.props().field_type) {
                        Some(custom_field) => custom_field.render(&self.entity, ctx.props().field_mode),
                        None => html! {
                            <CrudAlert variant={crate::crud_alert::Variant::Danger}>
                                { format!("The custom field '{:?}' should have been displayed here, but no renderer for that field was found in the `custom_*_fields` section of the static instance config. You might have forgotten to set the required HashMap entry.", &ctx.props().field_type)}
                            </CrudAlert>
                        }
                    }
                },
                Value::String(value) => match &ctx.props().field_mode {
                    FieldMode::Display => html! {
                        <div>{value}</div>
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"text"}
                                value={value}
                                disabled={true}
                            />
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"text"}
                                value={value}
                                onkeyup={ctx.link().callback(|event| match keyboard_event_target_as::<web_sys::HtmlInputElement>(event) {
                                    Ok(input) => Msg::Send(Value::String(input.value())),
                                    Err(err) => Msg::LogInputRetrievalErr(err.into())
                                })}
                                onchange={ctx.link().callback(|event| match event_target_as::<web_sys::HtmlInputElement>(event) {
                                    Ok(input) => Msg::Send(Value::String(input.value())),
                                    Err(err) => Msg::LogInputRetrievalErr(err.into())
                                })}
                                disabled={options.disabled}
                            />
                        </div>
                    },
                },
                Value::Text(value) => match &ctx.props().field_mode {
                    FieldMode::Display => html! {
                        <div>{value}</div>
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <CrudTipTapEditor
                                api_base_url={ctx.props().api_base_url.clone()}
                                id={self.format_id()}
                                class={"crud-input-field"}
                                value={value}
                                disabled={true}
                            />
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <CrudTipTapEditor
                                api_base_url={ctx.props().api_base_url.clone()}
                                id={self.format_id()}
                                class={"crud-input-field"}
                                value={value}
                                onchange={ctx.link().callback(|input| Msg::Send(Value::String(input)))}
                                disabled={options.disabled}
                            />
                        </div>
                    },
                },
                Value::Json(value) => match &ctx.props().field_mode {
                    FieldMode::Display => html! {
                        <div>{value.get_string_representation()}</div>
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <CrudTipTapEditor
                                api_base_url={ctx.props().api_base_url.clone()}
                                id={self.format_id()}
                                class={"crud-input-field"}
                                value={value.get_string_representation().to_owned()}
                                disabled={true}
                            />
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <CrudTipTapEditor
                                api_base_url={ctx.props().api_base_url.clone()}
                                id={self.format_id()}
                                class={"crud-input-field"}
                                value={value.get_string_representation().to_owned()}
                                onchange={ctx.link().callback(|input| Msg::Send(Value::String(input)))}
                                disabled={options.disabled}
                            />
                        </div>
                    },
                },
                // TODO: Find better way to handle `None` variant!
                Value::OptionalJson(value) => match &ctx.props().field_mode {
                    FieldMode::Display => html! {
                        <div>{value.as_ref().map(|it| it.get_string_representation()).unwrap_or("")}</div>
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <CrudTipTapEditor
                                api_base_url={ctx.props().api_base_url.clone()}
                                id={self.format_id()}
                                class={"crud-input-field"}
                                value={value.as_ref().map(|it| it.get_string_representation().to_owned()).unwrap_or_default()}
                                disabled={true}
                            />
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <CrudTipTapEditor
                                api_base_url={ctx.props().api_base_url.clone()}
                                id={self.format_id()}
                                class={"crud-input-field"}
                                value={value.as_ref().map(|it| it.get_string_representation().to_owned()).unwrap_or_default()}
                                onchange={ctx.link().callback(|input| Msg::Send(Value::String(input)))}
                                disabled={options.disabled}
                            />
                        </div>
                    },
                },
                Value::UuidV4(value) => match &ctx.props().field_mode {
                    FieldMode::Display => html! {
                        <div>{value.to_string()}</div>
                    },
                    // Never editable
                    FieldMode::Readable | FieldMode::Editable  => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"text"}
                                value={value.to_string()}
                                disabled={true}
                            />
                        </div>
                    },
                },
                Value::UuidV7(value) => match &ctx.props().field_mode {
                    FieldMode::Display => html! {
                        <div>{value.to_string()}</div>
                    },
                    // Never editable
                    FieldMode::Readable | FieldMode::Editable  => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"text"}
                                value={value.to_string()}
                                disabled={true}
                            />
                        </div>
                    },
                },
                Value::Ulid(value) => match &ctx.props().field_mode {
                    FieldMode::Display => html! {
                        <div>{value.to_string()}</div>
                    },
                    // Never editable
                    FieldMode::Readable | FieldMode::Editable  => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"text"}
                                value={value.to_string()}
                                disabled={true}
                            />
                        </div>
                    },
                },
                Value::U32(value) => match &ctx.props().field_mode {
                    FieldMode::Display => html! {
                        <div>{format!("{}", value)}</div>
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", value)}
                                disabled={true}
                            />
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", value)}
                                onkeyup={ctx.link().callback(|event| match keyboard_event_target_as::<web_sys::HtmlInputElement>(event) {
                                    Ok(input) => match input.value().parse::<u32>() {
                                        Ok(u32) => Msg::Send(Value::U32(u32)),
                                        Err(err) =>Msg::LogInputRetrievalErr(err.into()),
                                    }
                                    Err(err) => Msg::LogInputRetrievalErr(err.into())
                                })}
                                onchange={ctx.link().callback(|event| match event_target_as::<web_sys::HtmlInputElement>(event) {
                                    Ok(input) => match input.value().parse::<u32>() {
                                        Ok(u32) => Msg::Send(Value::U32(u32)),
                                        Err(err) =>Msg::LogInputRetrievalErr(err.into()),
                                    }
                                    Err(err) => Msg::LogInputRetrievalErr(err.into())
                                })}
                                disabled={options.disabled}
                            />
                        </div>
                    },
                },
                Value::OptionalU32(optional_u32) => match &ctx.props().field_mode {
                    FieldMode::Display => match optional_u32 {
                        Some(u32) => html! { <div>{format!("{}", u32)}</div> },
                        None => html! { "-" },
                    },
                    // TODO: Use nullable input field!
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", optional_u32.unwrap_or_default())}
                                disabled={true}
                            />
                        </div>
                    },
                    // TODO: Use nullable input field!
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", optional_u32.unwrap_or_default())}
                                onkeyup={ctx.link().callback(|event| match keyboard_event_target_as::<web_sys::HtmlInputElement>(event) {
                                    Ok(input) => match input.value().parse::<u32>() {
                                        Ok(u32) => Msg::Send(Value::OptionalU32(Some(u32))),
                                        Err(err) =>Msg::LogInputRetrievalErr(err.into()),
                                    }
                                    Err(err) => Msg::LogInputRetrievalErr(err.into())
                                })}
                                onchange={ctx.link().callback(|event| match event_target_as::<web_sys::HtmlInputElement>(event) {
                                    Ok(input) => match input.value().parse::<u32>() {
                                        Ok(u32) => Msg::Send(Value::OptionalU32(Some(u32))),
                                        Err(err) =>Msg::LogInputRetrievalErr(err.into()),
                                    }
                                    Err(err) => Msg::LogInputRetrievalErr(err.into())
                                })}
                                disabled={options.disabled}
                            />
                        </div>
                    },
                },
                Value::I32(value) => match &ctx.props().field_mode {
                    FieldMode::Display => html! {
                        <div>{format!("{}", value)}</div>
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", value)}
                                disabled={true}
                            />
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", value)}
                                onkeyup={ctx.link().callback(|event| match keyboard_event_target_as::<web_sys::HtmlInputElement>(event) {
                                    Ok(input) => match input.value().parse::<i32>() {
                                        Ok(i32) => Msg::Send(Value::I32(i32)),
                                        Err(err) =>Msg::LogInputRetrievalErr(err.into()),
                                    }
                                    Err(err) => Msg::LogInputRetrievalErr(err.into())
                                })}
                                onchange={ctx.link().callback(|event| match event_target_as::<web_sys::HtmlInputElement>(event) {
                                    Ok(input) => match input.value().parse::<i32>() {
                                        Ok(i32) => Msg::Send(Value::I32(i32)),
                                        Err(err) =>Msg::LogInputRetrievalErr(err.into()),
                                    }
                                    Err(err) => Msg::LogInputRetrievalErr(err.into())
                                })}
                                disabled={options.disabled}
                            />
                        </div>
                    },
                },
                Value::I64(value) => match &ctx.props().field_mode {
                    FieldMode::Display => html! {
                        <div>{format!("{}", value)}</div>
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", value)}
                                disabled={true}
                            />
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", value)}
                                onkeyup={ctx.link().callback(|event| match keyboard_event_target_as::<web_sys::HtmlInputElement>(event) {
                                    Ok(input) => match input.value().parse::<i64>() {
                                        Ok(i64) => Msg::Send(Value::I64(i64)),
                                        Err(err) =>Msg::LogInputRetrievalErr(err.into()),
                                    }
                                    Err(err) => Msg::LogInputRetrievalErr(err.into())
                                })}
                                onchange={ctx.link().callback(|event| match event_target_as::<web_sys::HtmlInputElement>(event) {
                                    Ok(input) => match input.value().parse::<i64>() {
                                        Ok(i64) => Msg::Send(Value::I64(i64)),
                                        Err(err) =>Msg::LogInputRetrievalErr(err.into()),
                                    }
                                    Err(err) => Msg::LogInputRetrievalErr(err.into())
                                })}
                                disabled={options.disabled}
                            />
                        </div>
                    },
                },
                Value::OptionalI32(value) => match &ctx.props().field_mode {
                    FieldMode::Display => match value {
                        Some(value) => html! { <div>{format!("{}", value)}</div> },
                        None => html! { "-" },
                    },
                    // TODO: Use nullable input field!
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", value.unwrap_or_default())}
                                disabled={true}
                            />
                        </div>
                    },
                    // TODO: Use nullable input field!
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", value.unwrap_or_default())}
                                onkeyup={ctx.link().callback(|event| match keyboard_event_target_as::<web_sys::HtmlInputElement>(event) {
                                    Ok(input) => match input.value().parse::<i32>() {
                                        Ok(i32) => Msg::Send(Value::OptionalI32(Some(i32))),
                                        Err(err) =>Msg::LogInputRetrievalErr(err.into()),
                                    }
                                    Err(err) => Msg::LogInputRetrievalErr(err.into())
                                })}
                                onchange={ctx.link().callback(|event| match event_target_as::<web_sys::HtmlInputElement>(event) {
                                    Ok(input) => match input.value().parse::<i32>() {
                                        Ok(i32) => Msg::Send(Value::OptionalI32(Some(i32))),
                                        Err(err) =>Msg::LogInputRetrievalErr(err.into()),
                                    }
                                    Err(err) => Msg::LogInputRetrievalErr(err.into())
                                })}
                                disabled={options.disabled}
                            />
                        </div>
                    },
                },
                Value::OptionalI64(value) => match &ctx.props().field_mode {
                    FieldMode::Display => match value {
                        Some(value) => html! { <div>{format!("{}", value)}</div> },
                        None => html! { "-" },
                    },
                    // TODO: Use nullable input field!
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", value.unwrap_or_default())}
                                disabled={true}
                            />
                        </div>
                    },
                    // TODO: Use nullable input field!
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", value.unwrap_or_default())}
                                onkeyup={ctx.link().callback(|event| match keyboard_event_target_as::<web_sys::HtmlInputElement>(event) {
                                    Ok(input) => match input.value().parse::<i64>() {
                                        Ok(i64) => Msg::Send(Value::OptionalI64(Some(i64))),
                                        Err(err) =>Msg::LogInputRetrievalErr(err.into()),
                                    }
                                    Err(err) => Msg::LogInputRetrievalErr(err.into())
                                })}
                                onchange={ctx.link().callback(|event| match event_target_as::<web_sys::HtmlInputElement>(event) {
                                    Ok(input) => match input.value().parse::<i64>() {
                                        Ok(i64) => Msg::Send(Value::OptionalI64(Some(i64))),
                                        Err(err) =>Msg::LogInputRetrievalErr(err.into()),
                                    }
                                    Err(err) => Msg::LogInputRetrievalErr(err.into())
                                })}
                                disabled={options.disabled}
                            />
                        </div>
                    },
                },
                Value::F32(value) => match &ctx.props().field_mode {
                    FieldMode::Display => html! {
                        <div>{format!("{}", value)}</div>
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", value)}
                                disabled={true}
                            />
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", value)}
                                onkeyup={ctx.link().callback(|event| match keyboard_event_target_as::<web_sys::HtmlInputElement>(event) {
                                    Ok(input) => match input.value().parse::<f32>() {
                                        Ok(value) => Msg::Send(Value::F32(value)),
                                        Err(err) =>Msg::LogInputRetrievalErr(err.into()),
                                    }
                                    Err(err) => Msg::LogInputRetrievalErr(err.into())
                                })}
                                onchange={ctx.link().callback(|event| match event_target_as::<web_sys::HtmlInputElement>(event) {
                                    Ok(input) => match input.value().parse::<f32>() {
                                        Ok(value) => Msg::Send(Value::F32(value)),
                                        Err(err) =>Msg::LogInputRetrievalErr(err.into()),
                                    }
                                    Err(err) => Msg::LogInputRetrievalErr(err.into())
                                })}
                                disabled={options.disabled}
                            />
                        </div>
                    },
                },
                Value::Bool(value) => match &ctx.props().field_mode {
                    FieldMode::Display => html! {
                        <div>{format!("{}", value)}</div>
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <div id={self.format_id()} class={"crud-input-field"}>
                                <CrudToggle
                                    state={value}
                                    disabled={true}
                                />
                            </div>
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <div id={self.format_id()} class={"crud-input-field"}>
                                <CrudToggle
                                    state={value}
                                    on_toggle={ctx.link().callback(|value| Msg::Send(Value::Bool(value)))}
                                    disabled={true}
                                />
                            </div>
                        </div>
                    },
                },
                Value::ValidationStatus(value) => match &ctx.props().field_mode {
                    FieldMode::Display => match value {
                        true => html! {
                            <CrudIcon variant={Bi::ExclamationTriangleFill} color={"#ffbd00"}/>
                        },
                        false => html! {
                            <CrudIcon variant={Bi::CheckLg}  color={"#469a46"}/>
                        },
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { "Only displayable..." }
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { "Only displayable..." }
                        </div>
                    },
                },
                Value::UtcDateTime(date_time) => match &ctx.props().field_mode {
                    FieldMode::Display => match options.date_time_display {
                        DateTimeDisplay::IsoUtc => html! {
                            <div>{date_time.to_rfc3339()}</div>
                        },
                        DateTimeDisplay::LocalizedLocal => html! {
                            <div>{date_time.format_local("%d.%m.%Y %H:%M")}</div>
                        },
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <CrudDatetime
                                id={self.format_id()}
                                value={date_time.clone()}
                                disabled={true}
                            />
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <CrudDatetime
                                id={self.format_id()}
                                value={date_time.clone()}
                                onchange={ctx.link().callback(|datetime: Option<UtcDateTime>| Msg::Send(Value::UtcDateTime(datetime.expect("Expected UtcDateTime to not be None!"))))}
                                disabled={options.disabled}
                            />
                        </div>
                    },
                },
                Value::OptionalUtcDateTime(optional_date_time) => match &ctx.props().field_mode {
                    FieldMode::Display => match options.date_time_display {
                        DateTimeDisplay::IsoUtc => match optional_date_time {
                            Some(date_time) => html! {
                                <div>{date_time.to_rfc3339()}</div>
                            },
                            None => html! {
                                <div>{""}</div>
                            },
                        },
                        DateTimeDisplay::LocalizedLocal => match optional_date_time {
                            Some(date_time) => html! {
                                <div>{date_time.format_local("%d.%m.%Y %H:%M")}</div>
                            },
                            None => html! {
                                <div>{""}</div>
                            },
                        },
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <CrudDatetime
                                id={self.format_id()}
                                value={optional_date_time.clone()}
                                disabled={true}
                            />
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            <CrudDatetime
                                id={self.format_id()}
                                value={optional_date_time.clone()}
                                onchange={ctx.link().callback(|datetime| Msg::Send(Value::OptionalUtcDateTime(datetime)))}
                                disabled={options.disabled}
                            />
                        </div>
                    },
                },
                Value::Select(selected) => match &ctx.props().field_mode {
                    FieldMode::Display => html!{format!("{:?}", selected)},
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            { render_select_child(&ctx) }
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            { render_select_child(&ctx) }
                        </div>
                    },
                },
                Value::Multiselect(selected) => match &ctx.props().field_mode {
                    FieldMode::Display => selected.into_iter()
                        .map(|value| html!{
                            format!("{:?}, ", value)
                        }).collect::<Html>(),
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            { render_select_child(&ctx) }
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            { render_select_child(&ctx) }
                        </div>
                    },
                },
                Value::OptionalSelect(selected) => match &ctx.props().field_mode {
                    FieldMode::Display => html!{format!("{:?}", selected)},
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            { render_select_child(&ctx) }
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            { render_select_child(&ctx) }
                        </div>
                    },
                },
                Value::OptionalMultiselect(selected) => match &ctx.props().field_mode {
                    FieldMode::Display => selected.into_iter()
                        .map(|value| html!{
                            format!("{:?}, ", value)
                        }).collect::<Html>(),
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            { render_select_child(&ctx) }
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            { render_label(&options) }
                            { render_select_child(&ctx) }
                        </div>
                    },
                },
            }
        }
    }
}

fn render_label(options: &FieldOptions) -> Html {
    html! {
        if let Some(label) = &options.label {
            <CrudFieldLabel label={label.clone()} />
        }
    }
}

fn render_select_child<T: CrudDataTrait + 'static>(ctx: &Context<CrudField<T>>) -> Html {
    ctx.props().children.iter().find(|child| {
        match child {
            Item::NestedInstance(_) => false,
            Item::Relation(_) => false,
            Item::Select(select) => select.props.name == ctx.props().field_type.get_name(),
        }
    }).map_or(html! {
        <CrudAlert variant={crate::crud_alert::Variant::Danger}>
            {"Could not find required 'Select' child. Help: You might be missing the <CrudSelectField> in your instance markup."}
        </CrudAlert>
    }, |it| it.into() )
}
