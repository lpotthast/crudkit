use crate::crud_instance::Item;

use super::prelude::*;
use std::marker::PhantomData;
use yew::{prelude::*, html::ChildrenRenderer};

pub enum Msg<T: CrudDataTrait> {
    ValueChanged((T::FieldType, String)),
}

#[derive(Properties, PartialEq)]
pub struct Props<T: CrudDataTrait> {
    pub children: ChildrenRenderer<Item>,
    pub api_base_url: String,
    pub elements: Vec<Elem<T>>,
    pub entity: Option<T>,
    pub mode: FieldMode,
    pub current_view: CrudView,
    pub value_changed: Callback<(T::FieldType, String)>,
}

pub struct CrudFields<T> {
    phantom_data: PhantomData<T>,
}

impl<T: 'static + CrudDataTrait> Component for CrudFields<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            phantom_data: PhantomData {},
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ValueChanged((field_type, value)) => {
                ctx.props().value_changed.emit((field_type, value));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            ctx.props().elements.iter().map(|elem| {
                html! {
                    match elem {
                        Elem::Enclosing(enclosing) => {
                            match enclosing {
                                Enclosing::None(group) => html! {
                                    <CrudFields<T>
                                        children={ctx.props().children.clone()}
                                        api_base_url={ctx.props().api_base_url.clone()}
                                        elements={group.children.clone()}
                                        entity={ctx.props().entity.clone()}
                                        mode={ctx.props().mode.clone()}
                                        current_view={ctx.props().current_view.clone()}
                                        value_changed={ctx.props().value_changed.clone()}
                                    />
                                },
                                Enclosing::Tabs(tabs) => html! {
                                    <CrudTabs>
                                        {
                                            for tabs.iter().map(|(tab_name, group)| {
                                                html_nested! {
                                                    <CrudTab name={tab_name.clone()}>
                                                        <CrudFields<T>
                                                            children={ctx.props().children.clone()}
                                                            api_base_url={ctx.props().api_base_url.clone()}
                                                            elements={group.children.clone()}
                                                            entity={ctx.props().entity.clone()}
                                                            mode={ctx.props().mode.clone()}
                                                            current_view={ctx.props().current_view.clone()}
                                                            value_changed={ctx.props().value_changed.clone()}
                                                        />
                                                    </CrudTab>
                                                }
                                            })
                                        }
                                    </CrudTabs>
                                },
                                Enclosing::Card(group) => html! {
                                    <div class={"crud-card"}>
                                        <CrudFields<T>
                                            children={ctx.props().children.clone()}
                                            api_base_url={ctx.props().api_base_url.clone()}
                                            elements={group.children.clone()}
                                            entity={ctx.props().entity.clone()}
                                            mode={ctx.props().mode.clone()}
                                            current_view={ctx.props().current_view.clone()}
                                            value_changed={ctx.props().value_changed.clone()}
                                        />
                                    </div>
                                },
                            }
                        },
                        Elem::Field((field_type, field_options)) => {
                            html! {
                                <CrudField<T>
                                    children={ctx.props().children.clone()}
                                    api_base_url={ctx.props().api_base_url.clone()}
                                    current_view={ctx.props().current_view.clone()}
                                    field_type={field_type.clone()}
                                    field_options={field_options.clone()}
                                    field_mode={ctx.props().mode.clone()}
                                    entity={ctx.props().entity.clone()}
                                    value_changed={ctx.link().callback(Msg::ValueChanged)}
                                />
                            }
                        },
                        Elem::Separator => {
                            html! {
                                <CrudSeparator />
                            }
                        },
                    }
                }
            }).collect::<Html>()
        }
    }
}
