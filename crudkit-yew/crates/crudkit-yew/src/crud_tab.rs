use yew::prelude::*;

use crudkit_web::Label;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub label: Label,
    pub children: Children,
}

pub struct CrudTab {}

impl Component for CrudTab {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={"crud-tab"}>
                {
                    for ctx.props().children.iter()
                }
            </div>
        }
    }
}
