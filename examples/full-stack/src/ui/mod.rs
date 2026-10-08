//! The styled CRUD components of "Keeper", the example's Quidditch league registry.
//!
//! CrudKit ships no screens and no styles, so this is the example's whole UI. Everything here renders the
//! application's own markup and classes (`ui-*`), composed from CrudKit's atoms (with the app's markup inside,
//! e.g. the empty state of `CrudListStatus`), Leptonic's atoms, and CrudKit's hooks where the app wants markup the
//! atoms do not cover (the page summary, the form sections).

mod dialogs;
mod forms;
mod inputs;
mod instance;
mod shell;
mod table;

pub use instance::Instance;
pub use shell::{PageHeader, Shell};

use crudkit_leptos::prelude::*;
use leptos::prelude::*;
use leptos_icons::Icon;

/// The app's list, create, read, and edit views.
pub fn view_registry() -> CrudViewRegistry {
    CrudViewRegistry::default()
        .table(|| view! { <table::TableView /> })
        .create(|| view! { <forms::CreateView /> })
        .read(|id| view! { <forms::ReadView id /> })
        .edit(|id| view! { <forms::EditView id /> })
}

/// Renders a "validation errors exist" flag as a status badge.
pub fn validation_status_renderer() -> FieldRenderer<DynReadField> {
    FieldRenderer::new(|state: CrudFieldState<DynReadField>| {
        let value = state.value;
        move || match value.get() {
            Value::Bool(true) => view! {
                <span class="ui-badge" data-tone="warning">
                    "Needs review"
                </span>
            }
            .into_any(),
            Value::Bool(false) => view! {
                <span class="ui-badge" data-tone="good">
                    "Valid"
                </span>
            }
            .into_any(),
            _ => ().into_any(),
        }
    })
}

/// How the records of the surrounding [`Instance`] are called, e.g. "club".
#[derive(Debug, Clone, Copy)]
struct Noun {
    singular: &'static str,
    plural: &'static str,
}

impl Noun {
    /// Returns `count` of the noun, e.g. "1 player" or "3 players".
    fn count(&self, count: usize) -> String {
        let noun = if count == 1 { self.singular } else { self.plural };
        format!("{count} {noun}")
    }
}

/// Whether the records of the surrounding [`Instance`] are only shown, e.g. in a read view
/// of their parent: no creating, selecting, editing, or deleting.
#[derive(Debug, Clone, Copy)]
struct ReadOnly(bool);

fn is_read_only() -> bool {
    use_context::<ReadOnly>().is_some_and(|ReadOnly(read_only)| read_only)
}

fn use_noun() -> Noun {
    use_context::<Noun>().unwrap_or(Noun {
        singular: "record",
        plural: "records",
    })
}

/// Renders a decorative icon. The surrounding control carries the accessible name.
fn icon(icon: icondata::Icon) -> impl IntoView {
    view! { <Icon icon attr:class="ui-icon" attr:aria-hidden="true" /> }
}
