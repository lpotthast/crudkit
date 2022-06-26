use yew::prelude::*;

use crate::Label;

use super::prelude::CrudTab;

pub enum Msg {
    TabSelected(Label),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub active_tab_name: Option<Label>,
    pub children: ChildrenWithProps<CrudTab>,
}

pub struct CrudTabs {
    pub active_tab: Option<Label>,
}

impl CrudTabs {
    fn compute_active_tab_label(ctx: &Context<Self>) -> Option<Label> {
        ctx.props()
            .active_tab_name
            .clone()
            .or_else(|| CrudTabs::get_first_tab_label(ctx))
    }

    fn get_first_tab_label(ctx: &Context<Self>) -> Option<Label> {
        ctx.props()
            .children
            .iter()
            .next()
            .map(|tab| tab.props.label.clone())
    }
}

impl Component for CrudTabs {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            active_tab: CrudTabs::compute_active_tab_label(ctx),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::TabSelected(tab_label) => {
                self.active_tab = Some(tab_label);
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.active_tab = CrudTabs::compute_active_tab_label(ctx);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={"crud-tabs"}>
                <div class={"crud-tab-selectors"}>
                    {
                        for ctx.props().children.iter().map(|tab| tab.props.label.clone()).map(|tab_label| {
                            let tab_clone = tab_label.clone();
                            let is_active = match &self.active_tab {
                                Some(active_tab_label) => active_tab_label == &tab_label,
                                None => false,
                            };
                            html! {
                                <div class={classes!("crud-tab-selector", is_active.then(|| "active"))} 
                                     onclick={ctx.link().callback(move |_| Msg::TabSelected(tab_clone.clone()))}>
                                    {tab_label.name.clone()}
                                </div>
                            }
                        })
                    }
                </div>
                {
                    match &self.active_tab {
                        Some(active_tab_label) => html! {
                            for ctx.props().children.iter().filter(|tab| &tab.props.label == active_tab_label)
                        },
                        None => html! {
                            <div>{"No tab selected..."}</div>
                        },
                    }
                }
            </div>
        }
    }
}
