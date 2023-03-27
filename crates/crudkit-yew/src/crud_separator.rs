use yew::prelude::*;

pub struct CrudSeparator {}

impl Component for CrudSeparator {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <hr class={"crud-separator solid"} />
        }
    }
}
