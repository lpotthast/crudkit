use yew::prelude::*;

#[derive(Debug, Properties, PartialEq)]
pub struct Props {
    pub variant: Variant,
    pub children: Children,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Primary,
    Secondary,
    Success,
    Info,
    Warn,
    Danger,
}

// TODO: This con be computed statically!
impl From<Variant> for Classes {
    fn from(variant: Variant) -> Self {
        match variant {
            Variant::Primary => classes!("type-primary"),
            Variant::Secondary => classes!("type-secondary"),
            Variant::Success => classes!("type-success"),
            Variant::Info => classes!("type-info"),
            Variant::Warn => classes!("type-warn"),
            Variant::Danger => classes!("type-danger"),
        }
    }
}

pub struct CrudAlert {}

impl Component for CrudAlert {
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
            <div class={classes!("crud-alert", ctx.props().variant)}>
                { for ctx.props().children.iter() }
            </div>
        }
    }
}
