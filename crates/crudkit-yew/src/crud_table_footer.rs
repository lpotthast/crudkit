use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
}

pub struct CrudTableFooter {}

impl Component for CrudTableFooter {
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
            <tfoot class={"crud-table-footer"}>
            </tfoot>
        }
    }
}
