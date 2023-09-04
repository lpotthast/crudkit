use yew::prelude::*;

pub struct CrudTree {}

#[derive(Properties, PartialEq)]
pub struct Props {
    children: Children,
}

impl Component for CrudTree {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            "test"
        }
    }
}
