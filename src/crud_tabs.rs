use yew::prelude::*;

use super::prelude::CrudTab;

pub enum Msg {
    TabSelected(String),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub active_tab_name: Option<String>,
    pub children: ChildrenWithProps<CrudTab>,
}

pub struct CrudTabs {
    pub active_tab: Option<String>,
}

impl CrudTabs {
    fn compute_active_tab_name(ctx: &Context<Self>) -> Option<String> {
        ctx.props()
            .active_tab_name
            .clone()
            .or_else(|| CrudTabs::get_first_tab_name(ctx))
    }

    fn get_first_tab_name(ctx: &Context<Self>) -> Option<String> {
        ctx.props()
            .children
            .iter()
            .next()
            .map(|tab| tab.props.name.clone())
    }
}

impl Component for CrudTabs {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            active_tab: CrudTabs::compute_active_tab_name(ctx),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::TabSelected(tab_name) => {
                self.active_tab = Some(tab_name);
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.active_tab = CrudTabs::compute_active_tab_name(ctx);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={"crud-tabs"}>
                <div class={"crud-tab-selectors"}>
                    {
                        for ctx.props().children.iter().map(|tab| tab.props.name.clone()).map(|tab_name| {
                            let tab_clone = tab_name.clone();
                            html! {
                                <div class={"crud-tab-selector"} onclick={ctx.link().callback(move |_| Msg::TabSelected(tab_clone.clone()))}>{tab_name.clone()}</div>
                            }
                        })
                    }
                </div>
                {
                    match &self.active_tab {
                        Some(active_tab_name) => html! {
                            for ctx.props().children.iter().filter(|tab| &tab.props.name == active_tab_name)
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
