use yew::prelude::*;

pub enum Msg {
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub children: Children,
    pub name: String,
}

pub struct CrudNestedInstance {}

impl Component for CrudNestedInstance {
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
            { for ctx.props().children.iter() }
        }
    }
}
