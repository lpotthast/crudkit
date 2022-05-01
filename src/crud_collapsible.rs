use yew::prelude::*;

pub enum Msg {}

pub struct CrudCollapsible {}

impl Component for CrudCollapsible {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div>
                { "CRUD instance" }
            </div>
        }
    }
}
