use yew::prelude::*;

pub struct CrudProgressBar {}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub percent: f64,
    #[prop_or(false)]
    pub show_percentage: bool,
}

impl Component for CrudProgressBar {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let background_gradient = format!("background: linear-gradient(90deg, rgba(57,46,242,1) 0%, rgba(136,186,254,1) {0}%, rgba(255,255,255,1) {0}%);", ctx.props().percent * 100.0);
        let formatted_percentage = format!("{:.0} %", ctx.props().percent * 100.0);
        html! {
            <div class="crud-progress-bar" style={background_gradient}>
                if ctx.props().show_percentage {
                    <span>{formatted_percentage}</span>
                }
            </div>
        }
    }
}
