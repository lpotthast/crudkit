use uuid::Uuid;
use yewdux::prelude::*;

use crate::types::toasts::Toast;

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
