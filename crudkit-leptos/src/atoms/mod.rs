//! Headless atoms: single-element components where CrudKit adds behavior to Leptonic's hooks.
//!
//! Atoms render no CSS classes of their own. They accept `classes` and expose their state through
//! `data-*` attributes.

// Leptos' `#[component]` macro drops `#[must_use]` from the hidden body function it generates, so
// `clippy::must_use_candidate` cannot be satisfied for exported components.
#![allow(clippy::must_use_candidate)]

pub mod icon;
pub mod table;

/// The headless atoms, e.g. to render a custom table.
pub mod prelude {
    pub use super::icon::CrudIcon;
    pub use super::table::{
        CrudTable, CrudTableBody, CrudTableCell, CrudTableHeader, CrudTableRow, CrudTableRows,
        use_crud_table_row,
    };
}
