use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

use crate::stores;

use super::prelude::*;

pub enum Msg {
    ToastsUpdated(Rc<stores::Toasts>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum HorizontalPosition {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum VerticalPosition {
    Top,
    Bottom,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub horizontal_position: HorizontalPosition,
    pub vertical_position: VerticalPosition,
}

pub struct CrudToasts {
    _toasts_dispatch: Dispatch<stores::Toasts>,
    toasts_state: Rc<stores::Toasts>,
}

impl Component for CrudToasts {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            _toasts_dispatch: Dispatch::subscribe(ctx.link().callback(Msg::ToastsUpdated)),
            toasts_state: Default::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToastsUpdated(state) => {
                self.toasts_state = state;
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class={"crud-toasts"}>
                {
                    self.toasts_state.get_toasts().map(|toast| {
                        html! {
                            <CrudToast toast={ toast.clone() } />
                        }
                    }).collect::<Html>()
                }
            </div>
        }
    }
}
