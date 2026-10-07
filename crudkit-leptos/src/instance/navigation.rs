//! Navigation scopes and dirty guards for CrudKit instances and nested application views.
//!
//! Internal table-to-edit navigation keeps the accepted view in CrudKit:
//!
//! ```no_run
//! # use crudkit_leptos::prelude::*;
//! # use crudkit_core::id::SerializableId;
//! let navigation = CrudNavigation::new(CrudView::table());
//! navigation.navigate(CrudView::edit(SerializableId(Vec::new())));
//! ```
//!
//! An application navigation scope can create a navigation attempt for an
//! application-owned action without inventing a view for the application shell:
//!
//! ```no_run
//! # use crudkit_leptos::prelude::*;
//! let application = CrudNavigationScope::new();
//! let editor = application.child_navigation(CrudView::create());
//! application.attempt(
//!     || { /* change the application route */ },
//!     || { /* keep or restore application-owned state */ },
//! );
//! # _ = editor;
//! ```
//!
//! A custom collection can give each inline editor an independent child. A
//! collection-level navigation attempt includes both children, while a navigation attempt on
//! one child ignores its dirty sibling:
//!
//! ```no_run
//! # use crudkit_leptos::prelude::*;
//! # use leptos::prelude::*;
//! let collection = CrudNavigation::new(CrudView::new("app.collection"));
//! let first = collection.child(CrudView::new("app.inline-editor"));
//! let second = collection.child(CrudView::new("app.inline-editor"));
//! first.guard(RwSignal::new(true).into());
//! second.guard(RwSignal::new(true).into());
//! first.navigate(CrudView::new("app.inline-summary"));
//! ```
//!
//! A drawer nested under a page uses child navigation and an explicit return
//! action. The callback runs only for an approved return attempt, never merely
//! because the drawer or its parent is unmounted:
//!
//! ```no_run
//! # use crudkit_leptos::prelude::*;
//! let page = CrudNavigation::new(CrudView::table());
//! let drawer = page.child(CrudView::new("app.drawer"));
//! drawer.return_with(|| { /* close the drawer */ });
//! drawer.return_from_current();
//! ```

#![deny(missing_docs)]

use crudkit_web::navigation::{
    AttemptResult, NavigationStateMachine, PendingEffect, ROOT_SCOPE, ScopeId,
};
use crudkit_web::view::CrudView;
use leptos::prelude::*;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

#[derive(Clone)]
enum ReturnAction {
    View(CrudView),
    Callback(PendingEffect),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CommittedReturnDestination {
    View,
    Callback,
}

#[derive(Clone)]
struct NavigationData {
    accepted: RwSignal<CrudView>,
    return_action: Arc<Mutex<ReturnAction>>,
}

/// Coordinates dirty guards and navigation attempts for a navigation subtree.
///
/// A navigation scope does not own an accepted [`CrudView`]. It only defines
/// which dirty guards participate in a navigation attempt. An attempt through a
/// parent navigation scope includes all descendant dirty guards, while an
/// attempt through a child navigation scope ignores dirty guards in sibling
/// navigation scopes.
///
/// Root navigation scopes created with [`Self::new`] are disposed automatically when the
/// current Leptos owner is cleaned up. Children created with [`Self::child`]
/// are removed from their parent automatically on owner cleanup.
#[derive(Clone, Copy)]
#[must_use = "navigation scopes must be retained by their Leptos owner"]
pub struct CrudNavigationScope {
    state_machine: StoredValue<Arc<Mutex<NavigationStateMachine>>>,
    pending: RwSignal<bool>,
    scope: ScopeId,
}

impl fmt::Debug for CrudNavigationScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrudNavigationScope")
            .field("scope", &self.scope)
            .field("pending", &self.pending.get_untracked())
            .finish_non_exhaustive()
    }
}

impl Default for CrudNavigationScope {
    fn default() -> Self {
        Self::new()
    }
}

impl CrudNavigationScope {
    /// Creates a root navigation scope owned by the current Leptos reactive owner.
    ///
    /// The root permits one pending navigation attempt across its complete
    /// navigation subtree. Cleanup disposes the navigation scope tree without invoking
    /// any pending approval or cancellation callback.
    pub fn new() -> Self {
        let root = Self {
            state_machine: StoredValue::new(Arc::new(Mutex::new(NavigationStateMachine::new()))),
            pending: RwSignal::new(false),
            scope: ROOT_SCOPE,
        };
        root.register_root_navigation_scope_cleanup();
        root
    }

    /// Creates a child navigation scope owned by the current owner.
    ///
    /// Attempts made through the returned navigation scope inspect only that
    /// navigation subtree. Attempts made through this parent navigation scope
    /// include the returned child navigation scope.
    pub fn child(&self) -> Self {
        let state_machine = self.state_machine();
        let scope = lock(&state_machine).add_scope(self.scope);
        let child = Self {
            state_machine: self.state_machine,
            pending: self.pending,
            scope,
        };
        child.register_navigation_scope_cleanup();
        child
    }

    /// Creates independent navigation in a new child navigation scope.
    ///
    /// The returned navigation owns its accepted view and return action. Its
    /// dirty guards remain visible to attempts made through this navigation scope.
    pub fn child_navigation(&self, initial_view: CrudView) -> CrudNavigation {
        CrudNavigation::from_scope(self.child(), initial_view)
    }

    /// Creates a navigation attempt for an application-owned action.
    ///
    /// The navigation attempt inspects dirty guards in this navigation subtree.
    /// `on_approved` runs immediately when the subtree is clean, or after the
    /// user accepts the leave confirmation. `on_cancelled` runs when the user
    /// cancels, when another navigation attempt is already pending, or when a
    /// dirty navigation attempt has no mounted confirmation host.
    pub fn attempt(
        &self,
        on_approved: impl Fn() + Send + Sync + 'static,
        on_cancelled: impl Fn() + Send + Sync + 'static,
    ) {
        let state_machine = self.state_machine();
        let result =
            lock(&state_machine).attempt(self.scope, Arc::new(on_approved), Arc::new(on_cancelled));
        let attempt_is_pending = matches!(result, AttemptResult::Pending);
        if attempt_is_pending {
            self.publish_pending_after_current_event(state_machine.clone());
        } else {
            self.pending.set(lock(&state_machine).has_pending());
        }
        match result {
            AttemptResult::Approved(effect) | AttemptResult::Rejected(effect) => effect(),
            AttemptResult::Pending => {}
        }
    }

    /// Registers a dirty guard in this navigation scope until owner cleanup.
    pub fn guard(&self, is_dirty: Signal<bool>) {
        let state_machine = self.state_machine();
        let guard =
            lock(&state_machine).add_guard(self.scope, Arc::new(move || is_dirty.get_untracked()));
        let state_machine = self.state_machine();
        on_cleanup(move || lock(&state_machine).remove_guard(guard));
    }

    // The wasm build moves `state_machine` into a task.
    #[cfg_attr(not(target_arch = "wasm32"), allow(clippy::needless_pass_by_value))]
    fn publish_pending_after_current_event(
        &self,
        state_machine: Arc<Mutex<NavigationStateMachine>>,
    ) {
        let pending = self.pending;
        #[cfg(target_arch = "wasm32")]
        leptos::task::spawn_local(async move {
            // A containing modal may have initiated the navigation attempt with Escape.
            // Publishing on the next microtask prevents the newly shown leave confirmation
            // from observing and cancelling that same key event in modal implementations
            // with independent listeners.
            pending.set(lock(&state_machine).has_pending());
        });
        #[cfg(not(target_arch = "wasm32"))]
        pending.set(lock(&state_machine).has_pending());
    }

    fn register_root_navigation_scope_cleanup(&self) {
        let root = *self;
        on_cleanup(move || root.dispose_owned_root_navigation_scope());
    }

    fn register_navigation_scope_cleanup(&self) {
        let state_machine = self.state_machine();
        let pending = self.pending;
        let scope = self.scope;
        on_cleanup(move || {
            let effect = lock(&state_machine).remove_scope(scope);
            pending.set(lock(&state_machine).has_pending());
            if let Some(effect) = effect {
                defer_navigation_scope_cancellation(state_machine, effect);
            }
        });
    }

    pub(crate) fn accept_pending(&self) {
        let state_machine = self.state_machine();
        let effect = lock(&state_machine).accept();
        self.pending.set(lock(&state_machine).has_pending());
        if let Some(effect) = effect {
            effect();
        }
    }

    pub(crate) fn cancel_pending(&self) {
        let state_machine = self.state_machine();
        let effect = lock(&state_machine).cancel();
        self.pending.set(lock(&state_machine).has_pending());
        if let Some(effect) = effect {
            effect();
        }
    }

    pub(crate) fn requires_leave_confirmation(&self) -> Signal<bool> {
        let scope = *self;
        Signal::derive(move || {
            let _ = scope.pending.get();
            lock(&scope.state_machine()).pending_host() == Some(scope.scope)
        })
    }

    pub(crate) fn register_confirmation_host(&self) {
        lock(&self.state_machine()).add_confirmation_host(self.scope);
    }

    fn dispose_owned_root_navigation_scope(&self) {
        debug_assert_eq!(
            self.scope, ROOT_SCOPE,
            "only root navigation scopes own their state machine"
        );
        let state_machine = self.state_machine();
        lock(&state_machine).clear();
        self.pending.set(false);
    }

    fn state_machine(&self) -> Arc<Mutex<NavigationStateMachine>> {
        self.state_machine.get_value()
    }
}

/// Owns the accepted view and return action for a CrudKit surface.
///
/// The initial view may be selected by application or route state before
/// construction. From that point onward, CrudKit owns the accepted view and
/// changes it only through navigation methods. Dirty guards are
/// scoped: attempts made by a parent include all descendants, while attempts
/// made by a child ignore dirty siblings.
///
/// `return_with` configures an explicit action. It is invoked only by an
/// approved call to [`Self::return_from_current`] (or CrudKit's private
/// committed follow-up after persistence); disposing the component never runs
/// it.
#[derive(Clone, Copy)]
#[must_use = "navigation must be retained or passed to a CrudKit surface"]
pub struct CrudNavigation {
    scope: CrudNavigationScope,
    data: StoredValue<NavigationData>,
}

impl fmt::Debug for CrudNavigation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrudNavigation")
            .field("scope", &self.scope.scope)
            .field("current", &self.data().accepted.get_untracked())
            .field("pending", &self.scope.pending.get_untracked())
            .finish_non_exhaustive()
    }
}

impl CrudNavigation {
    /// Creates root navigation owned by the current Leptos reactive owner.
    ///
    /// Use [`CrudNavigationScope::child_navigation`] when the navigation must
    /// participate in a larger application or manager navigation scope.
    pub fn new(initial_view: CrudView) -> Self {
        Self::from_scope(CrudNavigationScope::new(), initial_view)
    }

    fn from_scope(scope: CrudNavigationScope, initial_view: CrudView) -> Self {
        Self {
            scope,
            data: StoredValue::new(NavigationData {
                accepted: RwSignal::new(initial_view),
                return_action: Arc::new(Mutex::new(ReturnAction::View(CrudView::table()))),
            }),
        }
    }

    /// Returns the last accepted view as a reactive signal.
    #[must_use]
    pub fn current(&self) -> Signal<CrudView> {
        self.data().accepted.into()
    }

    /// Returns the navigation scope used by this navigation.
    pub fn scope(&self) -> CrudNavigationScope {
        self.scope
    }

    /// Sets the view used by [`Self::return_from_current`].
    pub fn return_to(&self, view: CrudView) {
        *lock(&self.data().return_action) = ReturnAction::View(view);
    }

    /// Configures an application-owned return callback, such as closing a modal.
    pub fn return_with(&self, callback: impl Fn() + Send + Sync + 'static) {
        *lock(&self.data().return_action) = ReturnAction::Callback(Arc::new(callback));
    }

    /// Attempts navigation to a registered view.
    pub fn navigate(&self, view: CrudView) {
        let navigation = *self;
        self.attempt(move || navigation.navigate_committed(view.clone()), || {});
    }

    /// Attempts the configured return action.
    pub fn return_from_current(&self) {
        let navigation = *self;
        let action = self.return_action();
        self.attempt(
            move || {
                navigation.commit_return_action(action.clone());
            },
            || {},
        );
    }

    /// Creates a navigation attempt for an application-owned action.
    ///
    /// The navigation attempt inspects dirty guards in this navigation scope
    /// and all descendant navigation scopes.
    pub fn attempt(
        &self,
        on_approved: impl Fn() + Send + Sync + 'static,
        on_cancelled: impl Fn() + Send + Sync + 'static,
    ) {
        self.scope.attempt(on_approved, on_cancelled);
    }

    /// Registers a dirty guard for this navigation scope.
    ///
    /// The registration is removed automatically when the current Leptos owner
    /// is cleaned up.
    pub fn guard(&self, is_dirty: Signal<bool>) {
        self.scope.guard(is_dirty);
    }

    /// Creates independently navigable child navigation.
    pub fn child(&self, initial_view: CrudView) -> Self {
        self.scope.child_navigation(initial_view)
    }

    /// Creates a private child navigation scope sharing the accepted view.
    pub(crate) fn scoped(&self) -> Self {
        Self {
            scope: self.scope.child(),
            data: self.data,
        }
    }

    pub(crate) fn navigate_committed(&self, view: CrudView) {
        self.data().accepted.set(view);
    }

    pub(crate) fn return_committed(&self) -> CommittedReturnDestination {
        self.commit_return_action(self.return_action())
    }

    fn return_action(&self) -> ReturnAction {
        lock(&self.data().return_action).clone()
    }

    fn commit_return_action(&self, action: ReturnAction) -> CommittedReturnDestination {
        match action {
            ReturnAction::View(view) => {
                self.navigate_committed(view);
                CommittedReturnDestination::View
            }
            ReturnAction::Callback(callback) => {
                callback();
                CommittedReturnDestination::Callback
            }
        }
    }

    pub(crate) fn accept_pending(&self) {
        self.scope.accept_pending();
    }

    pub(crate) fn cancel_pending(&self) {
        self.scope.cancel_pending();
    }

    pub(crate) fn requires_leave_confirmation(&self) -> Signal<bool> {
        self.scope.requires_leave_confirmation()
    }

    pub(crate) fn register_confirmation_host(&self) {
        self.scope.register_confirmation_host();
    }

    /// Whether the owner of this navigation was cleaned up, e.g. while a request was in flight.
    pub(crate) fn is_disposed(&self) -> bool {
        self.data.is_disposed()
    }

    fn data(&self) -> NavigationData {
        self.data.get_value()
    }
}

/// The navigation scope below which descendants create their default navigation: the scope of
/// the nearest rendered view of an instance, or else the nearest manager's scope.
///
/// Default instances and nested managers descend from it, so an attempt to leave a view includes
/// the dirty guards of instances nested inside of it.
#[derive(Clone, Copy)]
pub(crate) struct EnclosingNavigationScope(pub(crate) CrudNavigationScope);

/// Makes `scope` the [`EnclosingNavigationScope`] of the current owner's descendants.
pub(crate) fn provide_enclosing_navigation_scope(scope: CrudNavigationScope) {
    provide_context(EnclosingNavigationScope(scope));
}

/// Returns the nearest [`EnclosingNavigationScope`], if any.
pub(crate) fn use_enclosing_navigation_scope() -> Option<CrudNavigationScope> {
    use_context::<EnclosingNavigationScope>().map(|enclosing| enclosing.0)
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn defer_navigation_scope_cancellation(
    state_machine: Arc<Mutex<NavigationStateMachine>>,
    effect: PendingEffect,
) {
    leptos::task::spawn_local(async move {
        if !lock(&state_machine).is_disposed() {
            effect();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use leptos::reactive::owner::Owner;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[cfg(not(target_family = "wasm"))]
    fn initialize_executor() {
        crate::test_support::init_executor();
    }

    #[test]
    #[cfg(not(target_family = "wasm"))]
    fn mounted_navigation_scope_cleanup_defers_then_runs_its_cancellation_effect() {
        initialize_executor();
        let root_owner = Owner::new();
        let root = root_owner.with(|| CrudNavigation::new(CrudView::table()));
        let scope_owner = root_owner.with(Owner::new);
        let cancelled = Arc::new(AtomicUsize::new(0));
        let cancelled_for_effect = cancelled.clone();

        scope_owner.with(|| {
            let mounted = root.scoped();
            mounted.register_confirmation_host();
            mounted.guard(RwSignal::new(true).into());
            mounted.attempt(
                || {},
                move || {
                    cancelled_for_effect.fetch_add(1, Ordering::SeqCst);
                },
            );
        });

        scope_owner.cleanup();
        assert_that!(cancelled.load(Ordering::SeqCst)).is_equal_to(0);
        any_spawner::Executor::poll_local();
        assert_that!(cancelled.load(Ordering::SeqCst)).is_equal_to(1);
    }

    #[test]
    #[cfg(not(target_family = "wasm"))]
    fn automatic_root_navigation_scope_cleanup_suppresses_deferred_cancellation() {
        initialize_executor();
        let owner = Owner::new();
        let cancelled = Arc::new(AtomicUsize::new(0));
        let cancelled_for_effect = cancelled.clone();

        owner.with(|| {
            let root = CrudNavigation::new(CrudView::table());
            let mounted = root.scoped();
            mounted.register_confirmation_host();

            let view_owner = Owner::new();
            view_owner.with(|| {
                let view = mounted.scoped();
                view.guard(RwSignal::new(true).into());
                view.attempt(
                    || {},
                    move || {
                        cancelled_for_effect.fetch_add(1, Ordering::SeqCst);
                    },
                );
            });
            on_cleanup(move || drop(view_owner));
        });

        owner.cleanup();
        any_spawner::Executor::poll_local();
        assert_that!(cancelled.load(Ordering::SeqCst)).is_equal_to(0);
    }

    #[test]
    fn navigation_retains_accepted_view_until_approval() {
        let owner = Owner::new();
        owner.with(|| {
            let navigation = CrudNavigation::new(CrudView::table());
            navigation.register_confirmation_host();
            navigation.navigate(CrudView::new("app.clean"));
            assert_that!(navigation.current().get_untracked())
                .is_equal_to(CrudView::new("app.clean"));

            let dirty = RwSignal::new(true);
            navigation.guard(dirty.into());
            navigation.navigate(CrudView::new("app.target"));
            assert_that!(navigation.current().get_untracked())
                .is_equal_to(CrudView::new("app.clean"));
            assert_that!(navigation.requires_leave_confirmation().get_untracked()).is_true();

            navigation.accept_pending();
            assert_that!(navigation.current().get_untracked())
                .is_equal_to(CrudView::new("app.target"));
            assert_that!(navigation.requires_leave_confirmation().get_untracked()).is_false();
        });
    }

    #[test]
    fn return_targets_callbacks_and_committed_followups_have_distinct_semantics() {
        let owner = Owner::new();
        owner.with(|| {
            let navigation = CrudNavigation::new(CrudView::new("app.editor"));
            navigation.register_confirmation_host();
            navigation.return_to(CrudView::new("app.collection"));
            navigation.return_from_current();
            assert_that!(navigation.current().get_untracked())
                .is_equal_to(CrudView::new("app.collection"));

            navigation.navigate_committed(CrudView::new("app.editor"));
            assert_that!(navigation.return_committed())
                .is_equal_to(CommittedReturnDestination::View);
            assert_that!(navigation.current().get_untracked())
                .is_equal_to(CrudView::new("app.collection"));

            navigation.navigate_committed(CrudView::new("app.editor"));
            let dirty = RwSignal::new(true);
            navigation.guard(dirty.into());
            let returned = Arc::new(AtomicUsize::new(0));
            let returned_for_callback = returned.clone();
            navigation.return_with(move || {
                returned_for_callback.fetch_add(1, Ordering::SeqCst);
            });

            navigation.return_from_current();
            assert_that!(returned.load(Ordering::SeqCst)).is_equal_to(0);
            navigation.cancel_pending();
            assert_that!(returned.load(Ordering::SeqCst)).is_equal_to(0);

            assert_that!(navigation.return_committed())
                .is_equal_to(CommittedReturnDestination::Callback);
            assert_that!(returned.load(Ordering::SeqCst)).is_equal_to(1);

            navigation.return_from_current();
            navigation.scope.dispose_owned_root_navigation_scope();
            assert_that!(returned.load(Ordering::SeqCst)).is_equal_to(1);
            assert_that!(navigation.requires_leave_confirmation().get_untracked()).is_false();
        });
    }

    #[test]
    fn return_attempt_uses_the_return_action_selected_before_leave_confirmation() {
        let owner = Owner::new();
        owner.with(|| {
            let navigation = CrudNavigation::new(CrudView::new("app.editor"));
            navigation.register_confirmation_host();
            navigation.guard(RwSignal::new(true).into());
            let first_returned = Arc::new(AtomicUsize::new(0));
            let first_returned_for_callback = first_returned.clone();
            navigation.return_with(move || {
                first_returned_for_callback.fetch_add(1, Ordering::SeqCst);
            });

            navigation.return_from_current();

            let replacement_returned = Arc::new(AtomicUsize::new(0));
            let replacement_returned_for_callback = replacement_returned.clone();
            navigation.return_with(move || {
                replacement_returned_for_callback.fetch_add(1, Ordering::SeqCst);
            });
            navigation.accept_pending();

            assert_that!(first_returned.load(Ordering::SeqCst)).is_equal_to(1);
            assert_that!(replacement_returned.load(Ordering::SeqCst)).is_equal_to(0);
        });
    }

    #[test]
    fn application_return_callback_can_reconfigure_return_action() {
        let owner = Owner::new();
        owner.with(|| {
            let navigation = CrudNavigation::new(CrudView::new("app.editor"));
            let navigation_for_callback = navigation;
            navigation.return_with(move || {
                navigation_for_callback.return_to(CrudView::new("app.collection"));
            });

            navigation.return_committed();
            navigation.return_from_current();

            assert_that!(navigation.current().get_untracked())
                .is_equal_to(CrudView::new("app.collection"));
        });
    }

    #[test]
    fn child_navigation_ignores_dirty_siblings_but_root_navigation_includes_them() {
        let owner = Owner::new();
        owner.with(|| {
            let root = CrudNavigation::new(CrudView::new("app.collection"));
            root.register_confirmation_host();
            let left = root.child(CrudView::new("app.left"));
            let right = root.child(CrudView::new("app.right"));
            right.guard(RwSignal::new(true).into());

            left.navigate(CrudView::new("app.left-summary"));
            assert_that!(left.current().get_untracked())
                .is_equal_to(CrudView::new("app.left-summary"));
            assert_that!(root.requires_leave_confirmation().get_untracked()).is_false();

            root.navigate(CrudView::new("app.other-collection"));
            assert_that!(root.requires_leave_confirmation().get_untracked()).is_true();
            assert_that!(root.current().get_untracked())
                .is_equal_to(CrudView::new("app.collection"));
        });
    }

    #[test]
    fn mounted_navigation_scope_cleanup_preserves_supplied_navigation_for_remount() {
        let owner = Owner::new();
        owner.with(|| {
            let root = CrudNavigation::new(CrudView::new("app.collection"));
            let child = root.child(CrudView::new("app.child"));
            let first_mount = child.scoped();
            let state_machine = child.scope.state_machine();
            lock(&state_machine).remove_scope(first_mount.scope.scope);

            let second_mount = child.scoped();
            second_mount.register_confirmation_host();
            second_mount.guard(RwSignal::new(true).into());

            root.navigate(CrudView::new("app.other-collection"));

            assert_that!(second_mount.requires_leave_confirmation().get_untracked()).is_true();
            assert_that!(root.current().get_untracked())
                .is_equal_to(CrudView::new("app.collection"));
        });
    }

    #[test]
    fn shared_state_machine_selects_only_one_confirmation_host() {
        let owner = Owner::new();
        owner.with(|| {
            let root = CrudNavigation::new(CrudView::new("app.collection"));
            let left = root.child(CrudView::new("app.left"));
            let right = root.child(CrudView::new("app.right"));
            left.register_confirmation_host();
            right.register_confirmation_host();
            right.guard(RwSignal::new(true).into());

            root.navigate(CrudView::new("app.other-collection"));

            assert_that!(left.requires_leave_confirmation().get_untracked()).is_false();
            assert_that!(right.requires_leave_confirmation().get_untracked()).is_true();
            assert_that!(root.current().get_untracked())
                .is_equal_to(CrudView::new("app.collection"));
        });
    }
}
