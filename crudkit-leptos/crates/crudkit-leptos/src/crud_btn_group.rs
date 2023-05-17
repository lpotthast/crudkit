use yew::prelude::*;

pub struct CrudBtnGroup {}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub children: Children,
}

impl Component for CrudBtnGroup {
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
            <div class="crud-btn-group">
                { for ctx.props().children.iter() }
            </div>
        }
    }
}
