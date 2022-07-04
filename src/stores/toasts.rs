use std::rc::Rc;

use chrono_utc_date_time::UtcDateTime;
use uuid::Uuid;
use yew::{html::Scope, prelude::*};
use yewdux::prelude::*;

pub mod prelude {
    pub use super::{Toast, ToastVariant, Toasts, AutomaticallyClosing};
}

pub const AUTOMATIC_CLOSE_DELAY: u32 = 2500;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastVariant {
    Info,
    Success,
    Warn,
    Error,
}

impl From<ToastVariant> for Classes {
    fn from(variant: ToastVariant) -> Self {
        match variant {
            ToastVariant::Info => classes!("info"),
            ToastVariant::Success => classes!("success"),
            ToastVariant::Warn => classes!("warn"),
            ToastVariant::Error => classes!("error"),
        }
    }
}

#[derive(Clone)]
pub struct CloseCallback(Rc<dyn Fn()>);

impl CloseCallback {
    pub fn call(self) {
        self.0();
    }
}

impl std::fmt::Debug for CloseCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CloseCallback").finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomaticallyClosing {
    No,
    WithDefaultDelay,
    /// Accepts a delay in milliseconds.
    WithDelay(u32)
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub id: Uuid,
    pub created_at: UtcDateTime,
    pub variant: ToastVariant,
    pub heading: String,
    pub message: String,
    pub dismissible: bool,
    pub automatically_closing: AutomaticallyClosing,
    pub close_callback: Option<CloseCallback>,
}

impl PartialEq for Toast {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Default for Toast {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: UtcDateTime::now(),
            variant: ToastVariant::Info,
            heading: String::new(),
            message: String::new(),
            dismissible: true,
            automatically_closing: AutomaticallyClosing::WithDefaultDelay,
            close_callback: None,
        }
    }
}

impl CloseCallback {
    //pub fn new<COMP, MSG, F>(toast: Toast, link: Scope<COMP>, message_provider: F) -> Self
    //where
    //    COMP: Component,
    //    MSG: Into<COMP::Message>,
    //    F: Fn(Toast) -> MSG + 'static,
    //{
    //    let toast = toast.clone();
    //    let timeout = Rc::new(Timeout::new(toast.automatic_close_delay, move || {
    //        let message = message_provider(toast).into();
    //        link.send_message(message)
    //    }));
    //    Self(timeout)
    //}
    pub fn new<COMP, MSG, F>(toast: Toast, link: Scope<COMP>, message_provider: F) -> Self
    where
        COMP: Component,
        MSG: Into<COMP::Message>,
        F: Fn(Toast) -> MSG + 'static,
    {
        Self(Rc::new(move || {
            let message = message_provider(toast.clone()).into();
            link.send_message(message)
        }))
    }
}

#[derive(Clone, Default, PartialEq, Store)]
pub struct Toasts {
    toasts: Vec<Toast>,
}

impl Toasts {
    pub fn get_toasts(&self) -> impl Iterator<Item = &Toast> {
        self.toasts.iter()
    }

    pub fn push_toast(&mut self, toast: Toast) {
        self.toasts.push(toast);
    }

    pub fn remove_all_toasts(&mut self) {
        self.toasts.clear();
    }

    pub fn remove_toast(&mut self, toast: Toast) {
        if let Some(position) = self.toasts.iter().position(|it| it == &toast) {
            self.toasts.remove(position);
        }
    }

    pub fn remove_toast_with_id(&mut self, id: &Uuid) {
        if let Some(position) = self.toasts.iter().position(|it| &it.id == id) {
            self.toasts.remove(position);
        }
    }
}
