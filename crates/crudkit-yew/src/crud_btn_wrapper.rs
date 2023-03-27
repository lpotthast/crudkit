use yew::prelude::*;

pub struct CrudBtnWrapper {}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub children: Children,
}

impl Component for CrudBtnWrapper {
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
            <div class="crud-btn-wrapper">
                { for ctx.props().children.iter() }
            </div>
        }
    }
}
