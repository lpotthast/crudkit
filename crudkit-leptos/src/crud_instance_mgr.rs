//! Mounted CrudKit instance registration and navigation composition.

#![deny(missing_docs)]

use crate::crud_navigation::{CrudNavigation, CrudNavigationScope};
use crudkit_web::view::CrudView;
use leptos::prelude::*;

/// Reactive state published for one mounted [`crate::crud_instance::CrudInstance`].
#[derive(Debug, Clone)]
pub struct InstanceState {
    /// Stable lookup name supplied by the mounted instance.
    pub name: &'static str,
    /// Last view accepted by the instance's navigation.
    pub view: Signal<CrudView>,
}

#[derive(Debug, Clone)]
struct InstanceRegistration {
    id: u64,
    state: InstanceState,
}

/// Shared context for instances mounted below a [`CrudInstanceMgr`].
///
/// The context combines the instance registry used by parent-child resources
/// with a navigation scope covering the complete manager navigation subtree.
/// Application components inside the manager can use [`Self::navigation_scope`]
/// to create navigation attempts for route changes, modal closure, or other
/// application-owned actions. Those attempts include every dirty guard
/// registered by a descendant editor.
#[derive(Debug, Clone, Copy)]
pub struct CrudInstanceMgrContext {
    instances: StoredValue<Vec<InstanceRegistration>>,
    next_registration_id: StoredValue<u64>,
    navigation_scope: CrudNavigationScope,
}

impl CrudInstanceMgrContext {
    /// Returns the registered instance with `name`, if one is mounted.
    pub fn get_by_name(&self, name: &'static str) -> Option<InstanceState> {
        self.instances
            .read_value()
            .iter()
            .find(|registration| registration.state.name == name)
            .map(|registration| registration.state.clone())
    }

    /// Registers mounted instance state until the current Leptos owner is cleaned up.
    ///
    /// Instance names are expected to be unique inside one manager. A repeated
    /// name replaces the previous registration. Cleanup removes this registration
    /// only if it has not already been replaced by a newer registration.
    pub fn register(&self, name: &'static str, instance: InstanceState) {
        debug_assert_eq!(
            name, instance.name,
            "instance registration names must match"
        );
        let registration_id = self.next_registration_id.get_value();
        self.next_registration_id.set_value(
            registration_id
                .checked_add(1)
                .expect("CrudInstanceMgr registration id overflow"),
        );
        let registration = InstanceRegistration {
            id: registration_id,
            state: instance,
        };
        self.instances.update_value(|instances| {
            match instances
                .iter_mut()
                .find(|registration| registration.state.name == name)
            {
                Some(current) => *current = registration,
                None => instances.push(registration),
            }
        });
        let instances = self.instances;
        on_cleanup(move || {
            instances.update_value(|instances| {
                instances.retain(|registration| registration.id != registration_id);
            });
        });
    }

    /// Returns the navigation scope covering this manager's navigation subtree.
    ///
    /// Attempts through this navigation scope include dirty guards registered
    /// by every default child instance and nested manager. An instance's own
    /// navigation still targets only that instance subtree and ignores dirty
    /// siblings.
    pub fn navigation_scope(&self) -> CrudNavigationScope {
        self.navigation_scope
    }

    pub(crate) fn child_navigation(&self, initial_view: CrudView) -> CrudNavigation {
        self.navigation_scope.child_navigation(initial_view)
    }
}

/// Provides an instance registry and navigation scope to descendant components.
///
/// A top-level manager creates and owns a root [`CrudNavigationScope`]. A
/// nested manager creates a child of the nearest manager navigation scope, so
/// attempts through an ancestor navigation scope include its descendants while
/// nested attempts remain local.
///
/// Supplying `navigation_scope` mounts this manager below an application-owned
/// navigation scope instead. The supplied navigation scope remains
/// caller-owned; the manager creates and cleans up a private child navigation
/// scope beneath it.
#[component]
pub fn CrudInstanceMgr(
    /// Optional application-owned navigation scope under which this manager mounts.
    #[prop(optional)]
    navigation_scope: Option<CrudNavigationScope>,
    /// Descendant application and CrudKit views managed by this boundary.
    children: Children,
) -> impl IntoView {
    let parent = use_context::<CrudInstanceMgrContext>();
    let navigation_scope = manager_navigation_scope(navigation_scope, parent);
    provide_context(manager_context(navigation_scope));
    children()
}

fn manager_context(navigation_scope: CrudNavigationScope) -> CrudInstanceMgrContext {
    CrudInstanceMgrContext {
        instances: StoredValue::new(Vec::new()),
        next_registration_id: StoredValue::new(0),
        navigation_scope,
    }
}

fn manager_navigation_scope(
    supplied: Option<CrudNavigationScope>,
    parent: Option<CrudInstanceMgrContext>,
) -> CrudNavigationScope {
    supplied
        .or_else(|| parent.map(|parent| parent.navigation_scope()))
        .map(|scope| scope.child())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use leptos::reactive::owner::Owner;

    fn instance_state(name: &'static str, view: CrudView) -> InstanceState {
        InstanceState {
            name,
            view: RwSignal::new(view).into(),
        }
    }

    #[test]
    fn instance_registration_is_removed_on_owner_cleanup() {
        let manager_owner = Owner::new();
        let context = manager_owner.with(|| manager_context(CrudNavigationScope::new()));
        let instance_owner = manager_owner.with(Owner::new);
        instance_owner.with(|| {
            context.register(
                "example",
                instance_state("example", CrudView::new("app.example")),
            );
        });

        assert_that!(context.get_by_name("example").is_some()).is_true();

        instance_owner.cleanup();

        assert_that!(context.get_by_name("example").is_none()).is_true();
    }

    #[test]
    fn replaced_registration_survives_cleanup_of_previous_owner() {
        let manager_owner = Owner::new();
        let context = manager_owner.with(|| manager_context(CrudNavigationScope::new()));
        let previous_owner = manager_owner.with(Owner::new);
        previous_owner.with(|| {
            context.register(
                "example",
                instance_state("example", CrudView::new("app.previous")),
            );
        });
        let current_owner = manager_owner.with(Owner::new);
        current_owner.with(|| {
            context.register(
                "example",
                instance_state("example", CrudView::new("app.current")),
            );
        });

        previous_owner.cleanup();

        assert_that!(
            context
                .get_by_name("example")
                .expect("current registration should remain")
                .view
                .get_untracked()
        )
        .is_equal_to(CrudView::new("app.current"));

        current_owner.cleanup();

        assert_that!(context.get_by_name("example").is_none()).is_true();
    }

    #[test]
    fn nested_manager_navigation_scope_is_a_descendant_of_its_parent() {
        let owner = Owner::new();
        owner.with(|| {
            let root = manager_navigation_scope(None, None);
            let parent = manager_context(root);
            let nested = manager_navigation_scope(None, Some(parent));
            let navigation = nested.child_navigation(CrudView::table());
            navigation.register_confirmation_host();
            navigation.guard(RwSignal::new(true).into());

            root.attempt(|| {}, || {});

            assert_that!(navigation.requires_leave_confirmation().get_untracked()).is_true();
        });
    }

    #[test]
    fn supplied_navigation_scope_overrides_the_inherited_manager_navigation_scope() {
        let owner = Owner::new();
        owner.with(|| {
            let inherited = manager_navigation_scope(None, None);
            let supplied = CrudNavigationScope::new();
            let parent = manager_context(inherited);
            let mounted = manager_navigation_scope(Some(supplied), Some(parent));
            let navigation = mounted.child_navigation(CrudView::table());
            navigation.register_confirmation_host();
            navigation.guard(RwSignal::new(true).into());

            inherited.attempt(|| {}, || {});
            assert_that!(navigation.requires_leave_confirmation().get_untracked()).is_false();

            supplied.attempt(|| {}, || {});
            assert_that!(navigation.requires_leave_confirmation().get_untracked()).is_true();
        });
    }
}
