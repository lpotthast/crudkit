use uuid::Uuid;
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Clone, Default, PartialEq, Eq, Store)]
pub struct GlobalMouseMove {
    event: Option<MouseEvent>,
}

impl GlobalMouseMove {
    /// Access the latest event by reference.
    pub fn latest_event(&self) -> Option<&MouseEvent> {
        self.event.as_ref()
    }

    pub fn push_event(&mut self, event: MouseEvent) {
        self.event = Some(event);
    }
}

#[derive(Clone, Default, PartialEq, Eq, Store)]
pub struct GlobalMouseMoveRequired {
    required_by: Vec<Uuid>,
}

impl GlobalMouseMoveRequired {
    pub fn updates_required(&self) -> bool {
        !self.required_by.is_empty()
    }

    pub fn require_by(&mut self, uuid: Uuid) {
        if !self.required_by.contains(&uuid) {
            self.required_by.push(uuid);
        }
    }

    pub fn not_require_by(&mut self, uuid: Uuid) {
        if let Some(pos) = self.required_by.iter().position(|it| it == &uuid) {
            self.required_by.remove(pos);
        }
    }
}
