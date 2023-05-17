use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Clone, Default, PartialEq, Eq, Store)]
pub struct GlobalMouseUp {
    event: Option<MouseEvent>,
}

impl GlobalMouseUp {
    /// Access the latest event by reference.
    #[allow(dead_code)]
    pub fn latest_event(&self) -> Option<&MouseEvent> {
        self.event.as_ref()
    }

    pub fn push_event(&mut self, event: MouseEvent) {
        self.event = Some(event);
    }
}
