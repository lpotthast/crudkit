use ulid::Ulid;
use yew::prelude::*;

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

#[derive(Debug, PartialEq, Properties)]
pub struct CrudCheckboxProps {
    #[prop_or(false)]
    pub state: bool,
    #[prop_or(Size::Normal)]
    pub size: Size,
    #[prop_or(false)]
    pub active: bool,
    #[prop_or(false)]
    pub disabled: bool,
    #[prop_or_default]
    pub on_toggle: Callback<bool>,
}

pub struct CrudCheckbox {
    id: Ulid,
    state: bool,
}

impl Component for CrudCheckbox {
    type Message = Msg;
    type Properties = CrudCheckboxProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            id: Ulid::new(),
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

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.state = ctx.props().state;
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="crud-checkbox-wrapper">
                <input
                    type={"checkbox"}
                    class={classes!(
                        "crud-checkbox",
                        ctx.props().size,
                        ctx.props().active.then(|| "active"),
                        ctx.props().disabled.then(|| "disabled")
                    )}
                    id={self.id.to_string()}
                    onclick={&ctx.link().callback(|_| Msg::Toggle)}
                    checked={self.state}
                />
            </div>
        }
    }
}
