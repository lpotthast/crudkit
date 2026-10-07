//! Tab selection of field layouts.

use crate::hooks::instance::use_crud_instance;
use crudkit_web::layout::TabId;
use leptonic::hooks::Key;
use leptos::prelude::*;

/// Output of [`use_crud_tab_selection`]. Spread both onto Leptonic's `Tabs` atom.
#[derive(Debug, Clone, Copy)]
pub struct UseCrudTabSelectionReturn {
    /// The key of the selected tab, which is its tab ID.
    pub selected_key: Signal<Key>,
    /// Receives the tab the user selected.
    pub set_selected_key: Callback<Key>,
}

/// Binds the tab group consisting of `tabs` to the instance's tab selection, for Leptonic's
/// `Tabs` atom (`selected_key` and `set_selected_key`).
///
/// The selection is the one remembered by [`crate::instance::CrudInstanceContext::selected_tab`],
/// or the first tab while none was selected. Keys are the tab IDs.
///
/// # Panics
///
/// Panics when called outside of a CrudKit instance.
#[must_use]
pub fn use_crud_tab_selection(tabs: Vec<TabId>) -> UseCrudTabSelectionReturn {
    let ctx = use_crud_instance();
    let tabs = StoredValue::new(tabs);
    UseCrudTabSelectionReturn {
        selected_key: Signal::derive(move || {
            let selected = tabs.with_value(|tabs| {
                ctx.selected_tab(tabs)
                    .or_else(|| tabs.first().cloned())
                    .unwrap_or_default()
            });
            Key::from(selected.into_owned())
        }),
        set_selected_key: Callback::new(move |key: Key| {
            if let Some(tab) = key.as_str() {
                tabs.with_value(|tabs| ctx.select_tab(tabs, TabId::Owned(tab.to_owned())));
            }
        }),
    }
}
