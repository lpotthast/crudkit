//! "Marauder", the design system of the example's Quidditch league registry, named after the Marauder's Map.
//!
//! Everything here renders the application's own markup and classes (`mrd-*`). CrudKit only
//! contributes behavior: state hooks (`use_crud_list`, `use_crud_edit_form`, ...), input bindings
//! (`use_crud_text_input`, ...), its headless table atoms, and Leptonic's unstyled atoms. CrudKit's
//! theme does not style any of it.

mod dialogs;
mod forms;
mod inputs;
mod instance;
mod shell;
mod table;

pub use instance::MarauderInstance;
pub use shell::{MarauderPageHeader, MarauderShell};

use crudkit_leptos::prelude::*;
use leptos::prelude::*;
use leptos_icons::Icon;

/// A view registry rendering CrudKit's built-in view names with Marauder's views.
pub fn view_registry() -> CrudViewRegistry {
    let mut registry = CrudViewRegistry::default();
    registry.replace(TABLE_VIEW, |_view, navigation| {
        view! { <table::MarauderTableView navigation /> }.into_any()
    });
    registry.replace(CREATE_VIEW, |_view, navigation| {
        view! { <forms::MarauderCreateView navigation /> }.into_any()
    });
    registry.replace(READ_VIEW, |view, navigation| match view.subject {
        Some(id) => view! { <forms::MarauderReadView id navigation /> }.into_any(),
        None => view! { <p class="mrd-error">"Missing entity."</p> }.into_any(),
    });
    registry.replace(EDIT_VIEW, |view, navigation| match view.subject {
        Some(id) => view! { <forms::MarauderEditView id navigation /> }.into_any(),
        None => view! { <p class="mrd-error">"Missing entity."</p> }.into_any(),
    });
    registry
}

/// Renders a "validation errors exist" flag as a Marauder status badge.
pub fn validation_status_renderer() -> FieldRenderer<DynReadField> {
    FieldRenderer::new(|state: CrudFieldState<DynReadField>| {
        let value = state.value;
        move || match value.get() {
            Value::Bool(true) => view! {
                <span class="mrd-badge" data-tone="warning">
                    "Needs review"
                </span>
            }
            .into_any(),
            Value::Bool(false) => view! {
                <span class="mrd-badge" data-tone="good">
                    "Valid"
                </span>
            }
            .into_any(),
            _ => ().into_any(),
        }
    })
}

/// How the records of the surrounding [`MarauderInstance`] are called, e.g. "club".
#[derive(Debug, Clone, Copy)]
struct Noun {
    singular: &'static str,
    plural: &'static str,
}

fn use_noun() -> Noun {
    use_context::<Noun>().unwrap_or(Noun {
        singular: "record",
        plural: "records",
    })
}

/// Renders a decorative icon. The surrounding control carries the accessible name.
fn icon(icon: icondata::Icon) -> impl IntoView {
    view! { <Icon icon attr:class="mrd-icon" attr:aria-hidden="true" /> }
}
