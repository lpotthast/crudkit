use yew::prelude::*;

use crate::stores::Toast;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub toast: Toast,
}

pub struct CrudToast {}

impl Component for CrudToast {
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
            <div class={"crud-toast"}>
                <div class={"crud-toast-heading"}>
                    {ctx.props().toast.heading.clone()}
                </div>
                <div class={"crud-toast-message"}>
                    {ctx.props().toast.message.clone()}
                </div>
            </div>
        }
    }
}
