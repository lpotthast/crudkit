use yew::prelude::*;
use yew_bootstrap_icons::Bi;

pub struct CrudIcon {}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub variant: Bi,
    #[prop_or_default]
    pub class: Option<String>,
    #[prop_or_default]
    pub color: Option<String>,
}

impl Component for CrudIcon {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let classes = classes!("crud-icon", ctx.props().variant, ctx.props().class.clone());
        match &ctx.props().color {
            Some(color) =>  html! { <i class={ classes } style={ format!("color: {}", color.clone()) }></i> },
            None =>  html! { <i class={ classes }></i> },
        }
    }
}