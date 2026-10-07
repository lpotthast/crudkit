//! Confirmation of navigation attempts that would discard unsaved input.

use crate::hooks::instance::use_crud_instance;
use crate::instance::CrudNavigation;
use leptos::prelude::*;

/// A navigation attempt of one instance that waits for the user to confirm discarding unsaved
/// input.
#[derive(Debug, Clone, Copy)]
pub struct CrudLeaveConfirmation {
    navigation: CrudNavigation,
    /// Whether this instance must currently ask the user for confirmation.
    pub is_pending: Signal<bool>,
}

impl CrudLeaveConfirmation {
    /// Registers `navigation` as host of leave confirmations for its navigation subtree.
    pub(crate) fn host(navigation: CrudNavigation) -> Self {
        navigation.register_confirmation_host();
        Self {
            navigation,
            is_pending: navigation.requires_leave_confirmation(),
        }
    }

    /// Approves the pending navigation attempt, discarding unsaved input.
    pub fn accept(&self) {
        self.navigation.accept_pending();
    }

    /// Cancels the pending navigation attempt, keeping the current view and its input.
    pub fn cancel(&self) {
        self.navigation.cancel_pending();
    }
}

/// Returns the leave confirmation hosted by the surrounding instance.
///
/// # Panics
///
/// Panics when called outside of a CrudKit instance.
#[must_use]
pub fn use_crud_leave_confirmation() -> CrudLeaveConfirmation {
    use_crud_instance().leave_confirmation
}
