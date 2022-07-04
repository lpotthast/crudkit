use yew::prelude::*;

use crate::stores::toasts::Toast;

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
            <div id={format!("id-{}", ctx.props().toast.id)}
                class={classes!("crud-toast", ctx.props().toast.variant)}>
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
