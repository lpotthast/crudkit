use log::trace;
use yew::prelude::*;

use super::prelude::*;
use yewbi::Bi;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Small,
    Normal,
    Big,
}

// TODO: This con be computed statically!
impl From<Size> for Classes {
    fn from(size: Size) -> Self {
        match size {
            Size::Small => classes!("small"),
            Size::Normal => classes!("normal"), // TODO: is this even necessary?
            Size::Big => classes!("big"),
        }
    }
}

#[derive(Debug, PartialEq, Properties)]
pub struct CrudBtnProps {
    pub name: String,
    #[prop_or(Variant::Primary)]
    pub variant: Variant,
    #[prop_or(Size::Normal)]
    pub size: Size,
    #[prop_or(false)]
    pub active: bool,
    #[prop_or(false)]
    pub disabled: bool,
    #[prop_or_default]
    pub icon: Option<Bi>,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub children: Children,
}

pub struct CrudBtn {}

impl Component for CrudBtn {
    type Message = ();
    type Properties = CrudBtnProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        trace!("render");
        html! {
            <div
                class={classes!(
                    "crud-btn",
                    ctx.props().variant,
                    ctx.props().size,
                    ctx.props().active.then(|| "active"),
                    ctx.props().disabled.then(|| "disabled")
                )}
                onclick={&ctx.props().onclick}
            >
                if let Some(bi) = ctx.props().icon {
                    <CrudIcon variant={bi}/>
                }
                if ctx.props().children.is_empty() {
                    { &ctx.props().name }
                } else {
                    { for ctx.props().children.iter() }
                }
            </div>
        }
    }
}
