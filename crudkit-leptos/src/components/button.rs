//! Buttons of CrudKit's built-in views.

use crate::config::CrudActionIntent;
use leptonic::atoms::prelude::Button;
use leptonic::hooks::PressEvent;
use leptonic::utils::classes::{Classes, MergeStrategy};
use leptos::prelude::*;

impl CrudActionIntent {
    /// Returns the CSS class presenting this intent.
    #[must_use]
    pub fn class(self) -> &'static str {
        match self {
            Self::Primary => "crudkit-intent-primary",
            Self::Secondary => "crudkit-intent-secondary",
            Self::Success => "crudkit-intent-success",
            Self::Warning => "crudkit-intent-warning",
            Self::Danger => "crudkit-intent-danger",
        }
    }
}

/// A `button.crudkit-button` with an intent class, built on Leptonic's `Button` atom.
#[component]
pub fn CrudButton(
    /// Meaning of the button, presented through its intent class. Defaults to
    /// [`CrudActionIntent::Secondary`].
    #[prop(optional)]
    intent: CrudActionIntent,
    /// Called when the button is pressed by pointer, touch, or keyboard.
    #[prop(into)]
    on_press: Callback<PressEvent>,
    /// Whether the button ignores presses. Defaults to `false`.
    #[prop(into, optional)]
    is_disabled: Signal<bool>,
    /// Additional classes of the button.
    #[prop(into, optional)]
    classes: Classes,
    children: Children,
) -> impl IntoView {
    let classes =
        Classes::from(["crudkit-button", intent.class()]).merge(classes, MergeStrategy::default());
    view! {
        <Button classes is_disabled on_press>
            {children()}
        </Button>
    }
}
