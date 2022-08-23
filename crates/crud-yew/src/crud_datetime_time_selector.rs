use chrono_utc_date_time::UtcDateTime;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub value: UtcDateTime,
}

pub struct CrudDatetimeTimeSelector {
}

impl Component for CrudDatetimeTimeSelector {
    type Message = ();
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={"time-selector"}>
                {"TimeSelector"}
                {ctx.props().value.to_rfc3339()}
            </div>
        }
    }
}
