//! A CrudKit instance without any of CrudKit's markup.

use crate::marauder::Noun;
use crate::marauder::dialogs::MarauderDialogs;
use crudkit_leptos::prelude::*;
use leptos::context::Provider;
use leptos::prelude::*;

/// Mounts `config` like `CrudInstance`, but renders only Marauder markup.
///
/// `CrudInstanceProvider` and `CrudViewOutlet` render no markup; the current view comes from the
/// view registry that [`crate::marauder::MarauderShell`] provides, see [`crate::marauder::view_registry`].
#[component]
pub fn MarauderInstance(
    name: &'static str,
    config: CrudInstanceConfig,
    /// How one record is called in headings and buttons, e.g. "club".
    singular: &'static str,
    /// How several records are called, e.g. "clubs".
    plural: &'static str,
    #[prop(optional_no_strip)] parent: Option<CrudParentConfig>,
) -> impl IntoView {
    view! {
        <CrudInstanceProvider name config parent>
            <Provider value=Noun { singular, plural }>
                <section class="mrd-instance">
                    <CrudViewOutlet />
                    <MarauderDialogs />
                </section>
            </Provider>
        </CrudInstanceProvider>
    }
}
