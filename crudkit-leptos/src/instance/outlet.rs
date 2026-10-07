//! Structural components mounting an instance and rendering its current view, without markup of
//! their own.

use crate::config::{CrudInstanceConfig, CrudParentConfig, CrudViewRegistry};
use crate::hooks::instance::use_crud_instance;
use crate::instance::{
    CrudInstanceContext, CrudNavigation, EnclosingNavigationScope, create_instance_context,
    provide_view_context,
};
use crudkit_web::view::CrudView;
use leptos::context::Provider;
use leptos::prelude::*;

/// Mounts one configured CrudKit resource without rendering anything of its own.
///
/// Descendants can use CrudKit's hooks, e.g. [`crate::hooks::list::use_crud_list`], and
/// [`CrudViewOutlet`] to render the instance's current view. Unlike [`crate::components::instance::CrudInstance`], this renders
/// no confirmation dialogs; use [`crate::hooks::delete::use_crud_delete`] and
/// [`crate::hooks::leave::use_crud_leave_confirmation`] to present pending confirmations.
///
/// Navigation, registration, and parent scoping behave exactly as for [`crate::components::instance::CrudInstance`]. Like it,
/// this must be rendered below a [`crate::instance::CrudInstanceMgr`].
#[component]
// Leptos' `#[component]` macro drops `#[must_use]` from its hidden body function.
#[allow(clippy::must_use_candidate)]
pub fn CrudInstanceProvider(
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
    /// Content rendered with access to the instance.
    children: Children,
) -> impl IntoView {
    let ctx = create_instance_context(name, config, parent, navigation, on_context_created);
    view! {
        <Provider value=ctx>
            <Provider value=EnclosingNavigationScope(ctx.navigation.scope())>{children()}</Provider>
        </Provider>
    }
}

/// Renders the current view of the surrounding instance through its view registry.
///
/// Each rendered view gets its own reactive owner and a child navigation scope.
#[component]
// Leptos' `#[component]` macro drops `#[must_use]` from its hidden body function.
#[allow(clippy::must_use_candidate)]
pub fn CrudViewOutlet() -> impl IntoView {
    let ctx = use_crud_instance();
    let registry = ctx.static_config.read_value().view_registry.clone();
    let view = ctx.navigation.current();
    move || {
        view! {
            <RegisteredCrudView
                context=ctx
                registry=registry.clone()
                view=view.get()
                navigation=ctx.navigation
            />
        }
    }
}

#[component]
fn RegisteredCrudView(
    context: CrudInstanceContext,
    registry: CrudViewRegistry,
    view: CrudView,
    navigation: CrudNavigation,
) -> impl IntoView {
    let navigation = provide_view_context(&context, navigation);
    registry.render(view, navigation)
}
