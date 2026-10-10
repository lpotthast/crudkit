//! A CrudKit instance shown in the app's views and dialogs.

use crate::ui::dialogs::Dialogs;
use crate::ui::{Noun, ReadOnly};
use crudkit_leptos::prelude::*;
use leptos::context::Provider;
use leptos::prelude::*;

/// Mounts `config` with the app's views and dialogs.
///
/// `CrudInstance` and `CrudViewOutlet` render no markup; the current view comes from the view registry that
/// [`crate::ui::Shell`] provides, see [`crate::ui::view_registry`].
#[component]
pub fn Instance(
    name: &'static str,
    config: CrudInstanceConfig,
    /// How one record is called in headings and buttons, e.g. "club".
    singular: &'static str,
    /// How several records are called, e.g. "clubs".
    plural: &'static str,
    #[prop(optional_no_strip)] parent: Option<CrudParentConfig>,
    /// Only shows the records: no creating, selecting, editing, or deleting.
    #[prop(optional)]
    read_only: bool,
) -> impl IntoView {
    view! {
        <CrudInstance name config parent>
            <Provider value=Noun { singular, plural }>
                <Provider value=ReadOnly(read_only)>
                    <section class="ui-instance">
                        <CrudViewOutlet />
                        <Dialogs />
                    </section>
                </Provider>
            </Provider>
        </CrudInstance>
    }
}
