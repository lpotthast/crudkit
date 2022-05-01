use std::rc::Rc;

use gloo::timers::callback::Timeout;
use yew::{html::Scope, prelude::*};
use yewdux::prelude::*;

const AUTOMATIC_CLOSE_DELAY: u32 = 2500;

#[derive(Debug, Clone, PartialEq)]
pub enum ToastVariant {
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    pub variant: ToastVariant,
    pub heading: String,
    pub message: String,
    pub dismissible: bool,
    pub automatically_closing: bool,
    pub automatic_close_delay: u32,
}

impl Default for Toast {
    fn default() -> Self {
        Self {
            variant: ToastVariant::Info,
            heading: String::new(),
            message: String::new(),
            dismissible: true,
            automatically_closing: true,
            automatic_close_delay: AUTOMATIC_CLOSE_DELAY,
        }
    }
}

#[derive(Clone, Default)]
pub struct ToastWithTimeout {
    toast: Toast,
    _timeout: Option<Rc<Timeout>>,
}

// Custom implementation, as Timeout can not be compared..
impl PartialEq for ToastWithTimeout {
    fn eq(&self, other: &Self) -> bool {
        self.toast == other.toast
    }
}

impl ToastWithTimeout {
    pub fn new<COMP, MSG, F>(toast: Toast, scope: Scope<COMP>, message_provider: F) -> Self
    where
        COMP: Component,
        MSG: Into<COMP::Message>,
        F: Fn(Toast) -> MSG + 'static,
    {
        let timeout = if toast.automatically_closing {
            Some({
                let toast = toast.clone();
                Rc::new(Timeout::new(toast.automatic_close_delay, move || {
                    let message = message_provider(toast).into();
                    scope.send_message(message)
                }))
            })
        } else {
            None
        };

        Self {
            toast,
            _timeout: timeout,
        }
    }
}

#[derive(Clone, Default, PartialEq, Store)]
pub struct Toasts {
    toasts: Vec<ToastWithTimeout>,
}

impl Toasts {
    pub fn get_toasts(&self) -> impl Iterator<Item = &Toast> {
        self.toasts.iter().map(|it| &it.toast)
    }

    pub fn push_toast(&mut self, toast_with_timeout: ToastWithTimeout) {
        self.toasts.push(toast_with_timeout);
    }

    pub fn remove_toast(&mut self, toast: Toast) {
        if let Some(position) = self.toasts.iter().position(|it| it.toast == toast) {
            self.toasts.remove(position);
        }
    }
}
