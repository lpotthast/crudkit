use log::trace;
use yew::prelude::*;
use yewbi::Bi;

use super::prelude::*;

pub enum Msg {
    Toggle,
}

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

#[derive(Debug, PartialEq)]
pub struct CrudToggleIcons {
    pub off: Bi,
    pub on: Bi,
}

#[derive(Debug, PartialEq, Properties)]
pub struct CrudToggleProps {
    #[prop_or(false)]
    pub state: bool,
    #[prop_or(Size::Normal)]
    pub size: Size,
    #[prop_or(false)]
    pub active: bool,
    #[prop_or(false)]
    pub disabled: bool,
    #[prop_or_default]
    pub icons: Option<CrudToggleIcons>,
    #[prop_or_default]
    pub on_toggle: Callback<bool>,
}

pub struct CrudToggle {
    state: bool,
}

impl Component for CrudToggle {
    type Message = Msg;
    type Properties = CrudToggleProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            state: ctx.props().state,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Toggle => {
                self.state = !self.state;
                ctx.props().on_toggle.emit(self.state);
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.state = ctx.props().state;
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        trace!("render");
        html! {
            <div class="crud-toggle-wrapper">
                <label
                    class={classes!(
                        "crud-toggle",
                        ctx.props().size,
                        ctx.props().active.then(|| "active"),
                        ctx.props().disabled.then(|| "disabled")
                    )}
                    onclick={&ctx.link().callback(|_| Msg::Toggle)}
                >
                    <span class={classes!("slider", "round", self.state.then(|| "on"))}>
                        if let Some(icons) = &ctx.props().icons {
                            <span class={"icon-positioner"}>
                                if self.state {
                                    <CrudIcon variant={icons.on}/>
                                } else {
                                    <CrudIcon variant={icons.off}/>
                                }
                            </span>
                        }
                    </span>
                </label>
            </div>
        }
    }
}
