use std::{collections::BTreeMap, rc::Rc};

use gloo::timers::callback::Timeout;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::stores;
use crate::types::toasts::{Toast, ToastAutomaticallyClosing, AUTOMATIC_CLOSE_DELAY};

use super::prelude::*;

pub enum Msg {
    ToastsUpdated(Rc<stores::toasts::Toasts>),
    ToastTimedOut(Toast),
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

pub struct ToastWithTimeout {
    toast: Toast,
    timeout: Option<Timeout>,
}

pub struct CrudToasts {
    _toasts_dispatch: Dispatch<stores::toasts::Toasts>,

    toasts: BTreeMap<time::OffsetDateTime, ToastWithTimeout>,
}

impl Component for CrudToasts {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            _toasts_dispatch: Dispatch::subscribe(ctx.link().callback(Msg::ToastsUpdated)),

            toasts: BTreeMap::new(),
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        let keys: Vec<time::OffsetDateTime> = self.toasts.keys().cloned().collect();
        for key in keys {
            if let Some(toast_with_timeout) = self.toasts.remove(&key) {
                if let Some(timeout) = toast_with_timeout.timeout {
                    timeout.cancel();
                }
                if let Some(close_callback) = toast_with_timeout.toast.close_callback {
                    close_callback.call();
                }
            }
        }
        self._toasts_dispatch
            .reduce_mut(|state| state.remove_all_toasts())
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToastsUpdated(state) => {
                // Remove old toasts no longer present.
                let mut keys_to_remove = vec![];
                for toast_with_timeout in self.toasts.values() {
                    if !state.get_toasts().any(|it| it == &toast_with_timeout.toast) {
                        keys_to_remove.push(toast_with_timeout.toast.created_at.clone());
                    }
                }
                for key in keys_to_remove {
                    // Just drop the toast, not calling its callback.
                    self.toasts.remove(&key);
                }

                // Add new toasts.
                for toast in state.get_toasts() {
                    if !self.toasts.contains_key(&toast.created_at) {
                        // Create timeout
                        let timeout = (toast.automatically_closing
                            != ToastAutomaticallyClosing::No)
                            .then(|| {
                                let link = ctx.link().clone();
                                let toast_clone = toast.clone();
                                let delay = match toast.automatically_closing {
                                    ToastAutomaticallyClosing::No => unreachable!(),
                                    ToastAutomaticallyClosing::WithDefaultDelay => {
                                        AUTOMATIC_CLOSE_DELAY
                                    }
                                    ToastAutomaticallyClosing::WithDelay { millis } => millis,
                                };
                                Timeout::new(delay, move || {
                                    link.send_message(Msg::ToastTimedOut(toast_clone))
                                })
                            });

                        // Add to map
                        self.toasts.insert(
                            toast.created_at.clone(),
                            ToastWithTimeout {
                                toast: toast.clone(),
                                timeout,
                            },
                        );
                    }
                }
                true
            }
            Msg::ToastTimedOut(toast) => {
                if self.toasts.contains_key(&toast.created_at) {
                    if let Some(close_callback) = toast.close_callback {
                        close_callback.call();
                    }
                    self.toasts.remove(&toast.created_at);

                    self._toasts_dispatch
                        .reduce_mut(move |state| state.remove_toast_with_id(&toast.id))
                }
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class={"crud-toasts"}>
                {
                    self.toasts.values().map(|toast_with_timeout| {
                        html! {
                            <CrudToast toast={ toast_with_timeout.toast.clone() } />
                        }
                    }).collect::<Html>()
                }
            </div>
        }
    }
}
