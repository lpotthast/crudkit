use web_sys::Element;
use yew::prelude::*;

pub struct CrudModal {}

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Children are required as this component is just a simple wrapper taking care of spawning the children in the appropriate position.
    pub children: Children,
}

impl Component for CrudModal {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        yew::create_portal(
            html! {
                <>
                    <div class={"crud-modal-backdrop"}></div>
                    <div class={"crud-modal"}>
                        { for ctx.props().children.iter() }
                    </div>
                </>
            },
            get_modal_host(),
        )
    }
}

fn get_modal_host() -> Element {
    web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("modal-host"))
        .expect("Expected to find element with id \"modal-host\"!")
}
