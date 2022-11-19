use crate::crud_instance::Item;

use super::prelude::*;
use std::marker::PhantomData;
use yew::{html::ChildrenRenderer, prelude::*};

pub enum Msg<T: CrudDataTrait> {
    ValueChanged((T::Field, Result<Value, String>)),
    TabSelected(Label),
}

#[derive(Properties, PartialEq)]
pub struct Props<T: CrudDataTrait> {
    pub children: ChildrenRenderer<Item>,
    pub api_base_url: String,
    pub elements: Vec<Elem<T>>,
    pub entity: Option<T>,
    pub mode: FieldMode,
    pub current_view: CrudSimpleView,
    pub value_changed: Callback<(T::Field, Result<Value, String>)>,
    pub active_tab: Option<Label>,
    pub on_tab_selection: Callback<Label>,
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
            Msg::TabSelected(label) => {
                ctx.props().on_tab_selection.emit(label);
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
                                        active_tab={ctx.props().active_tab.clone()}
                                        on_tab_selection={ctx.link().callback(|label| Msg::TabSelected(label))}
                                    />
                                },
                                Enclosing::Tabs(tabs) => html! {
                                    <CrudTabs
                                        active_tab={ctx.props().active_tab.clone()}
                                        on_tab_selection={ctx.link().callback(|label| Msg::TabSelected(label))}
                                    >
                                        {
                                            for tabs.iter().map(|tab| {
                                                html_nested! {
                                                    <CrudTab label={tab.label.clone()}>
                                                        <CrudFields<T>
                                                            children={ctx.props().children.clone()}
                                                            api_base_url={ctx.props().api_base_url.clone()}
                                                            elements={tab.group.children.clone()}
                                                            entity={ctx.props().entity.clone()}
                                                            mode={ctx.props().mode.clone()}
                                                            current_view={ctx.props().current_view.clone()}
                                                            value_changed={ctx.props().value_changed.clone()}
                                                            active_tab={ctx.props().active_tab.clone()}
                                                            on_tab_selection={ctx.link().callback(|label| Msg::TabSelected(label))}
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
                                            active_tab={ctx.props().active_tab.clone()}
                                            on_tab_selection={ctx.link().callback(|label| Msg::TabSelected(label))}
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
