use yew::prelude::*;

use crate::Label;

pub enum Msg {}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub label: Label,
}

pub struct CrudFieldLabel {}

impl Component for CrudFieldLabel {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <span class={"crud-field-label"}>
                {ctx.props().label.name.clone()}
            </span>
        }
    }
}
