use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Clone, Default, PartialEq, Eq, Store)]
pub struct GlobalKeyUp {
    event: Option<KeyboardEvent>,
}

impl GlobalKeyUp {
    /// Access the latest event by reference.
    #[allow(dead_code)]
    pub fn latest_event(&self) -> Option<&KeyboardEvent> {
        self.event.as_ref()
    }

    pub fn push_event(&mut self, event: KeyboardEvent) {
        self.event = Some(event);
    }
}
