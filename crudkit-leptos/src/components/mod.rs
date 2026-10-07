//! CrudKit's built-in, composed UI.
//!
//! Components render `crudkit-*` classes and `data-*` attributes, the DOM contract targeted by
//! `crudkit-leptos-theme`. Each built-in view is composed from the parts in this module, which
//! applications can recompose in their own views.

// Leptos' `#[component]` macro drops `#[must_use]` from the hidden body function it generates, so
// `clippy::must_use_candidate` cannot be satisfied for exported components.
#![allow(clippy::must_use_candidate)]

use leptonic::utils::classes::{Classes, MergeStrategy};

pub mod action_buttons;
mod builtin_views;
pub mod button;
pub mod dialogs;
pub mod field;
mod field_renderers;
pub mod form_layout;
pub mod form_views;
pub mod inputs;
pub mod instance;
pub mod notifications;
pub mod pagination;
pub mod table_view;

pub(crate) use builtin_views::register_builtin_views;

/// Returns CrudKit's `class` of a component's root element together with the caller's `classes`.
pub(crate) fn with_classes(class: &'static str, classes: Classes) -> Classes {
    Classes::from(class).merge(classes, MergeStrategy::default())
}

/// The built-in components, e.g. to recompose a built-in view.
pub mod prelude {
    pub use super::action_buttons::{CrudEntityActionButtons, CrudResourceActionButtons};
    pub use super::button::CrudButton;
    pub use super::dialogs::{CrudConfirmDialog, CrudInstanceDialogs};
    pub use super::field::CrudFormField;
    pub use super::form_layout::CrudFormLayout;
    pub use super::form_views::{
        CrudActionSlot, CrudActionsOutlet, CrudCreateView, CrudEditView, CrudReadView,
    };
    pub use super::instance::CrudInstance;
    pub use super::notifications::CrudNotificationRegion;
    pub use super::pagination::CrudPagination;
    pub use super::table_view::{CrudSelectionBar, CrudTableToolbar, CrudTableView};
}
