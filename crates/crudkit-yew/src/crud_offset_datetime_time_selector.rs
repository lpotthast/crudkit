use time::format_description::well_known::Rfc3339;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub value: time::OffsetDateTime,
}

pub struct CrudOffsetDatetimeTimeSelector {}

impl Component for CrudOffsetDatetimeTimeSelector {
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
            <div class={"time-selector"}>
                {"TimeSelector"}
                {ctx.props().value.format(&Rfc3339).unwrap()}
            </div>
        }
    }
}
