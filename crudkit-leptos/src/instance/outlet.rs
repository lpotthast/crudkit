//! Mounting an instance and rendering its current view. Neither renders an element of its own.

use crate::atoms::InstanceBoundary;
use crate::config::{CrudInstanceConfig, CrudParentConfig};
use crate::hooks::instance::use_crud_instance;
use crate::instance::{
    CrudInstanceContext, CrudNavigation, EnclosingNavigationScope, create_instance_context,
    provide_view_context,
};
use leptos::context::Provider;
use leptos::prelude::*;

/// Mounts one configured resource. Renders no element of its own.
///
/// Descendants reach the instance through context: the hooks (e.g.
/// [`use_crud_instance`](crate::hooks::use_crud_instance)), the atoms, and
/// [`CrudViewOutlet`], which renders the instance's current view. Place the instance's
/// confirmation dialogs (e.g. [`CrudDeleteDialog`](crate::atoms::CrudDeleteDialog)) next to the
/// outlet, so they serve every view.
///
/// The instance registers with the surrounding [`CrudInstanceMgr`](crate::instance::CrudInstanceMgr)
/// under `name`. With a `parent`, it shows only the entities referencing the entity shown by the
/// parent instance.
///
/// # Panics
///
/// Panics when rendered outside of a [`CrudInstanceMgr`](crate::instance::CrudInstanceMgr).
#[component]
// Leptos' `#[component]` macro drops `#[must_use]` from its hidden body function.
#[allow(clippy::must_use_candidate)]
pub fn CrudInstance(
    /// Stable name used to register this instance with its manager. Unique per manager.
    name: &'static str,
    /// What the instance shows and how it reaches its resource.
    config: CrudInstanceConfig,
    /// Restricts the instance to the entities referencing another instance's entity.
    #[prop(optional_no_strip)]
    parent: Option<CrudParentConfig>,
    /// Navigation to use instead of one the manager creates, e.g. to control the instance's views
    /// from outside.
    #[prop(optional_no_strip)]
    navigation: Option<CrudNavigation>,
    /// Called once after the instance's context is created.
    #[prop(optional)]
    on_context_created: Option<Callback<CrudInstanceContext>>,
    /// Content with access to the instance, typically a [`CrudViewOutlet`] and dialogs.
    children: Children,
) -> impl IntoView {
    let ctx = create_instance_context(name, config, parent, navigation, on_context_created);
    // The row or form of an outer instance is not this instance's.
    view! {
        <Provider value=ctx>
            <Provider value=EnclosingNavigationScope(ctx.navigation.scope())>
                <InstanceBoundary>{children()}</InstanceBoundary>
            </Provider>
        </Provider>
    }
}

/// Renders the current view of the surrounding instance with the renderer registered for it in the
/// instance's [`CrudViewRegistry`](crate::config::CrudViewRegistry). Renders no element of its
/// own.
///
/// Each rendered view gets its own reactive owner and navigation scope. Inside, hooks and atoms see
/// the view's navigation, see [`use_crud_navigation`](crate::hooks::use_crud_navigation).
///
/// With `navigation`, the outlet renders the views of that navigation instead, e.g. a
/// [`CrudNavigation::child`] shown in a drawer while the instance's own view stays. The
/// navigation must descend from the instance's, as children of
/// [`use_crud_navigation`](crate::hooks::use_crud_navigation) do:
/// leave confirmations of its views are shown by the instance's
/// [`CrudLeaveDialog`](crate::atoms::CrudLeaveDialog).
#[component]
// Leptos' `#[component]` macro drops `#[must_use]` from its hidden body function.
#[allow(clippy::must_use_candidate)]
pub fn CrudViewOutlet(
    /// The navigation whose current view is rendered. Defaults to the instance's.
    #[prop(optional)]
    navigation: Option<CrudNavigation>,
) -> impl IntoView {
    let ctx = use_crud_instance();
    let navigation = navigation.unwrap_or(ctx.navigation);
    let registry = ctx.static_config.read_value().view_registry.clone();
    let current = navigation.current();
    // Every accepted view mounts afresh, also when the current view is accepted again, e.g. a
    // create view after saving with `CreateAnother`. Rebuilding the previous markup in place would
    // keep it bound to the previous view's state.
    let acceptances = StoredValue::new(0_u64);
    let accepted = move || {
        let view = current.get();
        acceptances.update_value(|count| *count += 1);
        [(acceptances.get_value(), view)]
    };
    view! {
        <For
            each=accepted
            key=|(acceptance, _)| *acceptance
            children=move |(_, view)| {
                // Each view renders in a reactive owner of its own, which receives its context.
                provide_view_context(&ctx, navigation);
                registry.render(view)
            }
        />
    }
}
