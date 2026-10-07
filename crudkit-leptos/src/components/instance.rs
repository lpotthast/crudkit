//! The built-in instance: an instance with CrudKit's markup and confirmation dialogs.

use crate::components::dialogs::CrudInstanceDialogs;
use crate::config::{CrudInstanceConfig, CrudParentConfig};
use crate::instance::{
    CrudInstanceContext, CrudNavigation, CrudViewOutlet, create_instance_context,
};
use leptos::context::Provider;
use leptos::prelude::*;

/// Mounts one configured CrudKit resource and renders its current registered view.
///
/// Must be rendered below a [`crate::instance::CrudInstanceMgr`]; it panics otherwise.
///
/// Without `navigation`, the instance creates navigation in a child of the
/// enclosing navigation scope, initialized from
/// [`CrudInstanceConfig::initial_view`]. The enclosing navigation scope is the
/// scope of the view of another instance this instance is rendered in, or else
/// the nearest manager's, so leaving a view includes the drafts of instances
/// nested in it. Supplied navigation remains caller-owned and deliberately
/// overrides the enclosing navigation scope. In both cases, the instance mounts a
/// private child navigation scope so unmounting does not invalidate supplied
/// navigation or remove the enclosing navigation scope.
#[component]
// Leptos' `#[component]` macro drops `#[must_use]` from its hidden body function.
#[allow(clippy::must_use_candidate)]
pub fn CrudInstance(
    /// Stable name used to register this instance with its manager.
    name: &'static str,
    /// Resource, view, renderer, control, and request configuration for this mount.
    config: CrudInstanceConfig,
    /// Optional parent-resource relationship used to scope child data.
    #[prop(optional_no_strip)]
    parent: Option<CrudParentConfig>,
    /// Optional caller-owned navigation that replaces manager-created navigation.
    #[prop(optional_no_strip)]
    navigation: Option<CrudNavigation>,
    /// Optional callback invoked once after the instance context is created.
    #[prop(optional)]
    on_context_created: Option<Callback<CrudInstanceContext>>,
) -> impl IntoView {
    let ctx = create_instance_context(name, config, parent, navigation, on_context_created);

    view! {
        <Provider value=ctx>
            <div class="crudkit-instance" data-instance=name>
                <CrudViewOutlet />
                <CrudInstanceDialogs />
            </div>
        </Provider>
    }
}
