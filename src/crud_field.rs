use std::marker::PhantomData;

use crate::{crud_instance::Item, DateTimeDisplay, keyboard_event_target_as, event_target_as};

use super::prelude::*;
use chrono_utc_date_time::UtcDateTime;
use uuid::Uuid;
use yew::{prelude::*, html::ChildrenRenderer};

pub enum Msg {
    KeyUp(KeyboardEvent),
    InputChanged(Event),
    TextInputChanged(String),
    DatetimeInputChanged(UtcDateTime),
    OptionalDatetimeInputChanged(Option<UtcDateTime>),
    BoolChanged(bool),
}

#[derive(Properties, PartialEq)]
pub struct Props<T: CrudDataTrait> {
    pub children: ChildrenRenderer<Item>,
    pub api_base_url: String,
    pub current_view: CrudView,
    pub field_type: T::Field,
    pub field_options: FieldOptions,
    pub field_mode: FieldMode,
    pub entity: Option<T>,
    pub value_changed: Callback<(T::Field, Value)>, // how can we handle all possible types? serialization?
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

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.entity = ctx
            .props()
            .entity
            .clone()
            .unwrap_or_else(|| Default::default());
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            // Store the current input value at every keystroke.
            Msg::KeyUp(event) => match keyboard_event_target_as::<web_sys::HtmlInputElement>(event) {
                Ok(input) => {
                    ctx.props()
                        .value_changed
                        .emit((ctx.props().field_type.clone(), Value::String(input.value())));
                    false
                }
                Err(err) => {
                    log::error!("Could not get input value: {}", err);
                    false
                }
            }
            // Store the current input on every input-change event.
            Msg::InputChanged(event) => match event_target_as::<web_sys::HtmlInputElement>(event) {
                Ok(input) => {
                    ctx.props()
                        .value_changed
                        .emit((ctx.props().field_type.clone(), Value::String(input.value())));
                    false
                }
                Err(err) => {
                    log::error!("Could not get input value: {}", err);
                    false
                }
            },
            Msg::TextInputChanged(text) => {
                ctx.props()
                    .value_changed
                    .emit((ctx.props().field_type.clone(), Value::String(text)));
                false
            },
            Msg::DatetimeInputChanged(datetime) => {
                ctx.props()
                    .value_changed
                    .emit((ctx.props().field_type.clone(), Value::UtcDateTime(datetime)));
                false
            },
            Msg::OptionalDatetimeInputChanged(datetime) => {
                ctx.props()
                    .value_changed
                    .emit((ctx.props().field_type.clone(), Value::OptionalUtcDateTime(datetime)));
                false
            },
            Msg::BoolChanged(value) => {
                ctx.props()
                    .value_changed
                    .emit((ctx.props().field_type.clone(), Value::Bool(value)));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let options = &ctx.props().field_options;
        html! {
            match ctx.props().field_type.get_value(&self.entity) {
                Value::OneToOneRelation(_referenced_id) => match &ctx.props().field_mode {
                    FieldMode::Display => html! {
                        <div>{"FieldMode::Display wird von Feldern des Typs OneToOneRelation aktuell nicht unterstützt."}</div>
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            if let Some(label) = &options.label {
                                <CrudFieldLabel label={label.clone()} />
                            }
                            { ctx.props().children.iter().filter(|child| {
                                match child {
                                    Item::NestedInstance(_) => false,
                                    Item::Relation(related_field) => related_field.props.name == ctx.props().field_type.get_name(),
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
                                    Item::NestedInstance(_) => false,
                                    Item::Relation(related_field) => related_field.props.name == ctx.props().field_type.get_name(),
                                    Item::Select(_) => false,
                                }
                            }).collect::<Html>() }
                        </div>
                    },
                },
                Value::NestedTable(_referenced_id) => {
                    match ctx.props().current_view {
                        CrudView::List => html! {
                            <div>{"Felder des Typs NestedTable können aktuell nicht in der Listenansicht dargestellt werden."}</div>
                        },
                        CrudView::Create => html! {
                            <>
                                if let Some(label) = &options.label {
                                    <CrudFieldLabel label={label.clone()} />
                                }
                                <div>{"Diese Informationen können erst bearbeitet werden, nachdem der Eintrag gespeichert wurde."}</div>
                            </>
                        },
                        CrudView::Read(_) | CrudView::Edit(_) => match &ctx.props().field_mode {
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
                Value::String(value) => match &ctx.props().field_mode {
                    FieldMode::Display => html! {
                        <div>{value}</div>
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            if let Some(label) = &options.label {
                                <CrudFieldLabel label={label.clone()} />
                            }
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
                            if let Some(label) = &options.label {
                                <CrudFieldLabel label={label.clone()} />
                            }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"text"}
                                value={value}
                                onkeyup={ctx.link().callback(Msg::KeyUp)}
                                onchange={ctx.link().callback(Msg::InputChanged)}
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
                            if let Some(label) = &options.label {
                                <CrudFieldLabel label={label.clone()} />
                            }
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
                            if let Some(label) = &options.label {
                                <CrudFieldLabel label={label.clone()} />
                            }
                            <CrudTipTapEditor 
                                api_base_url={ctx.props().api_base_url.clone()}
                                id={self.format_id()}
                                class={"crud-input-field"}
                                value={value}
                                onchange={ctx.link().callback(Msg::TextInputChanged)}
                                disabled={options.disabled}
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
                            if let Some(label) = &options.label {
                                <CrudFieldLabel label={label.clone()} />
                            }
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
                            if let Some(label) = &options.label {
                                <CrudFieldLabel label={label.clone()} />
                            }
                            <input
                                id={self.format_id()}
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", value)}
                                onkeyup={ctx.link().callback(Msg::KeyUp)}
                                onchange={ctx.link().callback(Msg::InputChanged)}
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
                            if let Some(label) = &options.label {
                                <CrudFieldLabel label={label.clone()} />
                            }
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
                            if let Some(label) = &options.label {
                                <CrudFieldLabel label={label.clone()} />
                            }
                            <div id={self.format_id()} class={"crud-input-field"}>
                                <CrudToggle
                                    state={value}
                                    on_toggle={ctx.link().callback(Msg::BoolChanged)}
                                    disabled={true}
                                />
                            </div>
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
                            if let Some(label) = &options.label {
                                <CrudFieldLabel label={label.clone()} />
                            }
                            <CrudDatetime
                                id={self.format_id()}
                                value={date_time.clone()}
                                disabled={true}
                            />
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            if let Some(label) = &options.label {
                                <CrudFieldLabel label={label.clone()} />
                            }
                            <CrudDatetime
                                id={self.format_id()}
                                value={date_time.clone()}
                                onchange={ctx.link().callback(|datetime: Option<UtcDateTime>| Msg::DatetimeInputChanged(datetime.unwrap()))}
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
                            if let Some(label) = &options.label {
                                <CrudFieldLabel label={label.clone()} />
                            }
                            <CrudDatetime
                                id={self.format_id()}
                                value={optional_date_time.clone()}
                                disabled={true}
                            />
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            if let Some(label) = &options.label {
                                <CrudFieldLabel label={label.clone()} />
                            }
                            <CrudDatetime
                                id={self.format_id()}
                                value={optional_date_time.clone()}
                                onchange={ctx.link().callback(Msg::OptionalDatetimeInputChanged)}
                                disabled={options.disabled}
                            />
                        </div>
                    },
                },
                Value::Select(optional_value) => match &ctx.props().field_mode {
                    FieldMode::Display => match optional_value {
                        Some(value) => html! { format!("{:?}", value) },
                        None => html! {"NULL"},
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            if let Some(label) = &options.label {
                                <CrudFieldLabel label={label.clone()} />
                            }
                            { ctx.props().children.iter().filter(|child| {
                                match child {
                                    Item::NestedInstance(_) => false,
                                    Item::Relation(_) => false,
                                    Item::Select(select) => select.props.name == ctx.props().field_type.get_name(),
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
                                    Item::NestedInstance(_) => false,
                                    Item::Relation(_) => false,
                                    Item::Select(select) => select.props.name == ctx.props().field_type.get_name(),
                                }
                            }).collect::<Html>() }
                        </div>
                    },
                },
            }
        }
    }
}
