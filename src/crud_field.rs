use std::marker::PhantomData;

use super::prelude::*;
use wasm_bindgen::JsCast;
use yew::prelude::*;

pub enum Msg {
    KeyUp(KeyboardEvent),
    InputChanged(Event),
    BoolChanged(bool),
}

#[derive(Properties, PartialEq)]
pub struct Props<T: CrudDataTrait> {
    pub field_type: T::FieldType,
    pub field_options: FieldOptions,
    pub field_mode: FieldMode,
    pub entity: Option<T>,
    pub value_changed: Callback<(T::FieldType, String)>, // how can we handle all possible types? serialization?
}

pub struct CrudField<T> {
    phantom_data: PhantomData<T>,
    entity: T,
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
            Msg::KeyUp(event) => {
                match keyboard_event_target_as::<web_sys::HtmlInputElement>(event) {
                    Ok(input) => {
                        ctx.props()
                            .value_changed
                            .emit((ctx.props().field_type.clone(), input.value()));
                        false
                    }
                    Err(err) => {
                        log::error!("Could not get input value: {}", err);
                        false
                    }
                }
            }
            Msg::InputChanged(event) => match event_target_as::<web_sys::HtmlInputElement>(event) {
                Ok(input) => {
                    ctx.props()
                        .value_changed
                        .emit((ctx.props().field_type.clone(), input.value()));
                    false
                }
                Err(err) => {
                    log::error!("Could not get input value: {}", err);
                    false
                }
            },
            Msg::BoolChanged(value) => {
                ctx.props()
                    .value_changed
                    .emit((ctx.props().field_type.clone(), format!("{}", value)));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            match ctx.props().field_type.get_value(&self.entity) {
                Value::String(value) => match &ctx.props().field_mode {
                    FieldMode::Display => html! {
                        <div>{value}</div>
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            <CrudFieldLabel label={ctx.props().field_options.label.clone()} />
                            <input
                                class={"crud-input-field"}
                                type={"text"}
                                value={value}
                                onkeyup={ctx.link().callback(Msg::KeyUp)}
                                onchange={ctx.link().callback(Msg::InputChanged)}
                                disabled={true}
                            />
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            <CrudFieldLabel label={ctx.props().field_options.label.clone()} />
                            <input
                                class={"crud-input-field"}
                                type={"text"}
                                value={value}
                                onkeyup={ctx.link().callback(Msg::KeyUp)}
                                onchange={ctx.link().callback(Msg::InputChanged)}
                                disabled={ctx.props().field_options.disabled}
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
                            <CrudFieldLabel label={ctx.props().field_options.label.clone()} />
                            <input
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", value)}
                                onkeyup={ctx.link().callback(Msg::KeyUp)}
                                onchange={ctx.link().callback(Msg::InputChanged)}
                                disabled={true}
                            />
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            <CrudFieldLabel label={ctx.props().field_options.label.clone()} />
                            <input
                                class={"crud-input-field"}
                                type={"number"}
                                value={format!("{}", value)}
                                onkeyup={ctx.link().callback(Msg::KeyUp)}
                                onchange={ctx.link().callback(Msg::InputChanged)}
                                disabled={ctx.props().field_options.disabled}
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
                            <CrudFieldLabel label={ctx.props().field_options.label.clone()} />
                            <div class={"crud-input-field"}>
                                <CrudToggle
                                    state={value}
                                    on_toggle={ctx.link().callback(Msg::BoolChanged)}
                                    disabled={true}
                                />
                            </div>
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            <CrudFieldLabel label={ctx.props().field_options.label.clone()} />
                            <div class={"crud-input-field"}>
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
                    FieldMode::Display => html! {
                        <div>{date_time.to_rfc3339()}</div>
                    },
                    FieldMode::Readable => html! {
                        <div class="crud-field">
                            <CrudFieldLabel label={ctx.props().field_options.label.clone()} />
                            <input
                                class={"crud-input-field"}
                                type={"text"}
                                value={date_time.to_rfc3339()}
                                onkeyup={ctx.link().callback(Msg::KeyUp)}
                                onchange={ctx.link().callback(Msg::InputChanged)}
                                disabled={true}
                            />
                        </div>
                    },
                    FieldMode::Editable => html! {
                        <div class="crud-field">
                            <CrudFieldLabel label={ctx.props().field_options.label.clone()} />
                            <input
                                class={"crud-input-field"}
                                type={"text"}
                                value={date_time.to_rfc3339()}
                                onkeyup={ctx.link().callback(Msg::KeyUp)}
                                onchange={ctx.link().callback(Msg::InputChanged)}
                                disabled={ctx.props().field_options.disabled}
                            />
                        </div>
                    },
                },
            }
        }
    }
}

fn event_target_as<T: JsCast>(event: Event) -> Result<T, String> {
    event
        .target()
        .ok_or_else(|| format!("Unable to obtain target from event: {:?}", event))
        .and_then(|event_target| {
            event_target
                .dyn_into::<T>()
                .map_err(|err| format!("Unable to cast event_target to T: {:?}", err.to_string()))
        })
}

fn keyboard_event_target_as<T: JsCast>(event: KeyboardEvent) -> Result<T, String> {
    event
        .target()
        .ok_or_else(|| format!("Unable to obtain target from event: {:?}", event))
        .and_then(|event_target| {
            event_target
                .dyn_into::<T>()
                .map_err(|err| format!("Unable to cast event_target to T: {:?}", err.to_string()))
        })
}
