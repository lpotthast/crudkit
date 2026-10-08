//! The atoms CrudKit screens are built from. The [crate documentation](crate) explains how they
//! combine.
//!
//! Every atom renders exactly one element, or none (providers, outlets, iterators). Action buttons
//! also render their action's view, an overlay, after their button. An atom's element carries a
//! default class, `crudkit-` plus the atom's name without its `Crud` prefix, and its
//! state as `data-*` attributes, absent when false. `classes` and `styles` add to them.
//!
//! An atom names the container it belongs in, e.g. a [`CrudTableCell`] inside a [`CrudTableRow`].
//! Rendered elsewhere, it panics with a message naming the atom, like Leptonic's atoms: that is a
//! mistake in the application's markup, not a runtime condition.

// Leptos' `#[component]` macro drops `#[must_use]` from the hidden body function it generates, so
// `clippy::must_use_candidate` cannot be satisfied for exported components.
#![allow(clippy::must_use_candidate)]
// Components receive their props by value, whether they consume them or not.
#![allow(clippy::needless_pass_by_value)]

mod actions;
mod content;
mod context;
mod dialogs;
mod entity;
mod field;
mod form;
mod icon;
mod layout;
mod list;
mod pagination;
mod status;
mod table;
mod value;

pub use actions::{
    CrudActionButton, CrudEntityActionButton, CrudEntityActions, CrudResourceActions,
};
pub use dialogs::{
    CrudCancelButton, CrudConfirmButton, CrudConfirmationMessage, CrudConfirmationTitle,
    CrudDeleteDialog, CrudDeleteManyDialog, CrudLeaveDialog,
};
pub use entity::{CrudCreateButton, CrudDeleteButton, CrudEditButton, CrudReadButton};
pub use field::{
    CrudCheckbox, CrudDisplayField, CrudField, CrudFieldControl, CrudFieldError, CrudFieldLabel,
    CrudFieldValue, CrudNumberField, CrudSwitch, CrudTextField,
};
pub use form::{
    CrudCreateForm, CrudDetails, CrudEditForm, CrudEntityStatus, CrudReturnButton, CrudSaveButton,
};
pub use icon::CrudIcon;
pub use layout::{
    CrudFormCard, CrudFormGroup, CrudFormLayout, CrudFormSeparator, CrudFormTab, CrudFormTabPanel,
    CrudFormTabs, CrudTab,
};
pub use list::{
    CrudClearSelectionButton, CrudDeleteSelectedButton, CrudList, CrudListStatus, CrudResetButton,
    CrudSelectionCount,
};
pub use pagination::{
    CrudItemsPerPage, CrudItemsPerPageOptions, CrudNextPageButton, CrudPageButton, CrudPageButtons,
    CrudPageGap, CrudPagination, CrudPreviousPageButton,
};
pub use table::{
    CrudRowAction, CrudRowCheckbox, CrudSelectAllCheckbox, CrudTable, CrudTableActionsCell,
    CrudTableActionsHeader, CrudTableBody, CrudTableCell, CrudTableCells, CrudTableColumnHeader,
    CrudTableColumnHeaders, CrudTableHeader, CrudTableHeaderRow, CrudTableRow, CrudTableRows,
    CrudTableSelectAllHeader, CrudTableSelectionCell,
};
pub use value::CrudValue;

pub(crate) use context::{InstanceBoundary, provide_instance_boundary};

// Markup is rendered to HTML on the server only, so these tests need the `ssr` feature.
#[cfg(all(test, not(target_family = "wasm"), feature = "ssr"))]
mod tests;
