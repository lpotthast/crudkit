//! UI-independent navigation scopes, dirty guards and navigation attempts.
//!
//! `crudkit-leptos` wraps this state machine in reactive navigation handles. Keeping it free of any UI
//! framework allows testing the navigation semantics described in `design/views-and-navigation.md` directly.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// The scope that every [`NavigationStateMachine`] starts with.
pub const ROOT_SCOPE: ScopeId = ScopeId(0);

/// Identifies a navigation scope inside one [`NavigationStateMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeId(u64);

/// Identifies a dirty guard inside one [`NavigationStateMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GuardId(u64);
/// Effect run when a navigation attempt is approved or cancelled.
pub type PendingEffect = Arc<dyn Fn() + Send + Sync>;

#[derive(Clone)]
struct GuardRegistration {
    scope: ScopeId,
    is_dirty: Arc<dyn Fn() -> bool + Send + Sync>,
}

struct PendingAttempt {
    target: ScopeId,
    confirmation_host: ScopeId,
    on_approved: PendingEffect,
    on_cancelled: PendingEffect,
}

/// Result of [`NavigationStateMachine::attempt`].
pub enum AttemptResult {
    /// No dirty guard participates; the caller must run the approval effect now.
    Approved(PendingEffect),
    /// The attempt waits for [`NavigationStateMachine::accept`] or [`NavigationStateMachine::cancel`].
    Pending,
    /// The attempt cannot be confirmed; the caller must run the cancellation effect now.
    Rejected(PendingEffect),
}

/// UI-independent state machine for navigation scope and navigation attempt semantics.
///
/// Scope [`ROOT_SCOPE`] always exists. Owners of a machine (e.g. the Leptos navigation handles in
/// `crudkit-leptos`) are responsible for running returned effects.
pub struct NavigationStateMachine {
    next_scope: u64,
    next_guard: u64,
    scopes: HashMap<ScopeId, Option<ScopeId>>,
    guards: HashMap<GuardId, GuardRegistration>,
    confirmation_hosts: HashSet<ScopeId>,
    pending: Option<PendingAttempt>,
    disposed: bool,
}

impl NavigationStateMachine {
    /// Creates a machine containing only the root scope.
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_scope: 1,
            next_guard: 0,
            scopes: HashMap::from([(ROOT_SCOPE, None)]),
            guards: HashMap::new(),
            confirmation_hosts: HashSet::new(),
            pending: None,
            disposed: false,
        }
    }

    /// Adds a child scope below `parent`.
    pub fn add_scope(&mut self, parent: ScopeId) -> ScopeId {
        let id = ScopeId(self.next_scope);
        self.next_scope += 1;
        self.scopes.insert(id, Some(parent));
        id
    }

    /// Registers a dirty guard in `scope`. The guard is sampled when an attempt starts.
    pub fn add_guard(
        &mut self,
        scope: ScopeId,
        is_dirty: Arc<dyn Fn() -> bool + Send + Sync>,
    ) -> GuardId {
        let id = GuardId(self.next_guard);
        self.next_guard += 1;
        self.guards
            .insert(id, GuardRegistration { scope, is_dirty });
        id
    }

    /// Removes a dirty guard.
    pub fn remove_guard(&mut self, guard: GuardId) {
        self.guards.remove(&guard);
    }

    fn is_in_subtree(&self, scope: ScopeId, ancestor: ScopeId) -> bool {
        let mut current = Some(scope);
        while let Some(id) = current {
            if id == ancestor {
                return true;
            }
            current = self.scopes.get(&id).copied().flatten();
        }
        false
    }

    fn dirty_scopes(&self, target: ScopeId) -> Vec<ScopeId> {
        self.guards
            .values()
            .filter(|guard| self.is_in_subtree(guard.scope, target) && (guard.is_dirty)())
            .map(|guard| guard.scope)
            .collect()
    }

    /// Marks `scope` as able to host a leave confirmation.
    pub fn add_confirmation_host(&mut self, scope: ScopeId) {
        self.confirmation_hosts.insert(scope);
    }

    fn nearest_confirmation_host(&self, scope: ScopeId) -> Option<ScopeId> {
        let mut current = Some(scope);
        while let Some(id) = current {
            if self.confirmation_hosts.contains(&id) {
                return Some(id);
            }
            current = self.scopes.get(&id).copied().flatten();
        }
        None
    }

    fn confirmation_host_for(&self, target: ScopeId, dirty_scopes: &[ScopeId]) -> Option<ScopeId> {
        self.nearest_confirmation_host(target)
            .or_else(|| {
                dirty_scopes
                    .iter()
                    .filter_map(|scope| self.nearest_confirmation_host(*scope))
                    .min()
            })
            .or_else(|| {
                self.confirmation_hosts
                    .iter()
                    .copied()
                    .filter(|host| self.is_in_subtree(*host, target))
                    .min()
            })
    }

    /// Starts a navigation attempt for the subtree of `target`.
    pub fn attempt(
        &mut self,
        target: ScopeId,
        on_approved: PendingEffect,
        on_cancelled: PendingEffect,
    ) -> AttemptResult {
        if self.pending.is_some() {
            return AttemptResult::Rejected(on_cancelled);
        }
        let dirty_scopes = self.dirty_scopes(target);
        if dirty_scopes.is_empty() {
            return AttemptResult::Approved(on_approved);
        }
        let Some(confirmation_host) = self.confirmation_host_for(target, &dirty_scopes) else {
            return AttemptResult::Rejected(on_cancelled);
        };
        self.pending = Some(PendingAttempt {
            target,
            confirmation_host,
            on_approved,
            on_cancelled,
        });
        AttemptResult::Pending
    }

    /// Approves the pending attempt and returns its approval effect.
    pub fn accept(&mut self) -> Option<PendingEffect> {
        self.pending.take().map(|pending| pending.on_approved)
    }

    /// Cancels the pending attempt and returns its cancellation effect.
    pub fn cancel(&mut self) -> Option<PendingEffect> {
        self.pending.take().map(|pending| pending.on_cancelled)
    }

    /// Removes `scope` and its descendants, returning the cancellation effect of an attempt that
    /// targeted or was hosted by the removed subtree.
    pub fn remove_scope(&mut self, scope: ScopeId) -> Option<PendingEffect> {
        let descendants = self
            .scopes
            .keys()
            .copied()
            .filter(|candidate| self.is_in_subtree(*candidate, scope))
            .collect::<Vec<_>>();
        self.guards
            .retain(|_, guard| !descendants.contains(&guard.scope));
        self.confirmation_hosts
            .retain(|host| !descendants.contains(host));
        for descendant in &descendants {
            self.scopes.remove(descendant);
        }

        self.pending
            .take_if(|pending| {
                descendants.contains(&pending.target)
                    || descendants.contains(&pending.confirmation_host)
            })
            .map(|pending| pending.on_cancelled)
    }

    /// Disposes the machine without running any pending effect.
    pub fn clear(&mut self) {
        self.guards.clear();
        self.confirmation_hosts.clear();
        self.pending = None;
        self.scopes.clear();
        self.disposed = true;
    }

    /// Returns whether an attempt is pending.
    #[must_use]
    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// Returns the confirmation host of the pending attempt.
    #[must_use]
    pub fn pending_host(&self) -> Option<ScopeId> {
        self.pending
            .as_ref()
            .map(|pending| pending.confirmation_host)
    }

    /// Returns whether [`Self::clear`] disposed the machine.
    #[must_use]
    pub fn is_disposed(&self) -> bool {
        self.disposed
    }
}

impl Default for NavigationStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    fn counter() -> (Arc<AtomicUsize>, PendingEffect) {
        let count = Arc::new(AtomicUsize::new(0));
        let count_for_effect = count.clone();
        let effect = Arc::new(move || {
            count_for_effect.fetch_add(1, Ordering::SeqCst);
        });
        (count, effect)
    }

    #[test]
    fn clean_attempt_approves_immediately() {
        let mut machine = NavigationStateMachine::new();
        let (approved, approve) = counter();
        let (_, cancel) = counter();
        let result = machine.attempt(ROOT_SCOPE, approve, cancel);
        assert_that!(matches!(result, AttemptResult::Approved(_))).is_true();
        let AttemptResult::Approved(effect) = result else {
            unreachable!("result was asserted to be approved");
        };
        effect();
        assert_that!(approved.load(Ordering::SeqCst)).is_equal_to(1);
        assert_that!(machine.has_pending()).is_false();
    }

    #[test]
    fn dirty_attempt_accepts_or_cancels_exactly_once() {
        for accept in [true, false] {
            let mut machine = NavigationStateMachine::new();
            machine.add_confirmation_host(ROOT_SCOPE);
            machine.add_guard(ROOT_SCOPE, Arc::new(|| true));
            let (approved, approve) = counter();
            let (cancelled, cancel) = counter();
            assert_that!(matches!(
                machine.attempt(ROOT_SCOPE, approve, cancel),
                AttemptResult::Pending
            ))
            .is_true();
            let effect = if accept {
                machine.accept()
            } else {
                machine.cancel()
            };
            effect.expect("pending navigation attempt should resolve")();
            assert_that!(approved.load(Ordering::SeqCst)).is_equal_to(usize::from(accept));
            assert_that!(cancelled.load(Ordering::SeqCst)).is_equal_to(usize::from(!accept));
            assert_that!(machine.accept().is_none()).is_true();
            assert_that!(machine.cancel().is_none()).is_true();
        }
    }

    #[test]
    fn root_navigation_scope_aggregates_descendants_but_child_navigation_scope_excludes_sibling() {
        let mut machine = NavigationStateMachine::new();
        machine.add_confirmation_host(ROOT_SCOPE);
        let left = machine.add_scope(ROOT_SCOPE);
        let right = machine.add_scope(ROOT_SCOPE);
        machine.add_guard(left, Arc::new(|| true));
        machine.add_guard(right, Arc::new(|| true));
        let (_, approve) = counter();
        let (_, cancel) = counter();
        assert_that!(matches!(
            machine.attempt(ROOT_SCOPE, approve.clone(), cancel.clone()),
            AttemptResult::Pending
        ))
        .is_true();
        machine.cancel();
        assert_that!(matches!(
            machine.attempt(left, approve, cancel),
            AttemptResult::Pending
        ))
        .is_true();
    }

    #[test]
    fn clean_child_navigation_scope_ignores_dirty_sibling() {
        let mut machine = NavigationStateMachine::new();
        let left = machine.add_scope(ROOT_SCOPE);
        let right = machine.add_scope(ROOT_SCOPE);
        machine.add_guard(right, Arc::new(|| true));
        let (_, approve) = counter();
        let (_, cancel) = counter();
        assert_that!(matches!(
            machine.attempt(left, approve, cancel),
            AttemptResult::Approved(_)
        ))
        .is_true();
    }

    #[test]
    fn second_attempt_is_cancelled_without_replacing_first() {
        let mut machine = NavigationStateMachine::new();
        machine.add_confirmation_host(ROOT_SCOPE);
        machine.add_guard(ROOT_SCOPE, Arc::new(|| true));
        let (first_approved, first_approve) = counter();
        let (_, first_cancel) = counter();
        machine.attempt(ROOT_SCOPE, first_approve, first_cancel);

        let (second_approved, second_approve) = counter();
        let (second_cancelled, second_cancel) = counter();
        let second = machine.attempt(ROOT_SCOPE, second_approve, second_cancel);
        assert_that!(matches!(second, AttemptResult::Rejected(_))).is_true();
        let AttemptResult::Rejected(effect) = second else {
            unreachable!("result was asserted to be rejected");
        };
        effect();
        machine
            .accept()
            .expect("first navigation attempt should remain")();
        assert_that!(first_approved.load(Ordering::SeqCst)).is_equal_to(1);
        assert_that!(second_approved.load(Ordering::SeqCst)).is_equal_to(0);
        assert_that!(second_cancelled.load(Ordering::SeqCst)).is_equal_to(1);
    }

    #[test]
    fn removing_targeted_nested_navigation_scope_cancels_its_attempt_only() {
        let mut machine = NavigationStateMachine::new();
        machine.add_confirmation_host(ROOT_SCOPE);
        let child = machine.add_scope(ROOT_SCOPE);
        machine.add_guard(child, Arc::new(|| true));
        let (_, approve) = counter();
        let (cancelled, cancel) = counter();
        machine.attempt(child, approve, cancel);
        machine
            .remove_scope(child)
            .expect("targeted navigation attempt should cancel")();
        assert_that!(cancelled.load(Ordering::SeqCst)).is_equal_to(1);
        assert_that!(machine.has_pending()).is_false();
    }

    #[test]
    fn removing_the_selected_confirmation_host_cancels_the_attempt() {
        let mut machine = NavigationStateMachine::new();
        let child = machine.add_scope(ROOT_SCOPE);
        machine.add_confirmation_host(child);
        machine.add_guard(child, Arc::new(|| true));
        let (_, approve) = counter();
        let (cancelled, cancel) = counter();
        machine.attempt(ROOT_SCOPE, approve, cancel);

        machine
            .remove_scope(child)
            .expect("hosted navigation attempt should cancel")();

        assert_that!(cancelled.load(Ordering::SeqCst)).is_equal_to(1);
        assert_that!(machine.has_pending()).is_false();
    }

    #[test]
    fn root_navigation_scope_guard_uses_a_descendant_confirmation_host() {
        let mut machine = NavigationStateMachine::new();
        let child = machine.add_scope(ROOT_SCOPE);
        machine.add_confirmation_host(child);
        machine.add_guard(ROOT_SCOPE, Arc::new(|| true));
        let (_, approve) = counter();
        let (_, cancel) = counter();

        machine.attempt(ROOT_SCOPE, approve, cancel);

        assert_that!(machine.pending_host()).is_equal_to(Some(child));
    }

    #[test]
    fn dirty_snapshot_is_evaluated_when_navigation_is_attempted() {
        let mut machine = NavigationStateMachine::new();
        machine.add_confirmation_host(ROOT_SCOPE);
        let dirty = Arc::new(AtomicBool::new(false));
        let dirty_for_guard = dirty.clone();
        machine.add_guard(
            ROOT_SCOPE,
            Arc::new(move || dirty_for_guard.load(Ordering::SeqCst)),
        );
        let (_, approve) = counter();
        let (_, cancel) = counter();
        assert_that!(matches!(
            machine.attempt(ROOT_SCOPE, approve.clone(), cancel.clone()),
            AttemptResult::Approved(_)
        ))
        .is_true();
        dirty.store(true, Ordering::SeqCst);
        assert_that!(matches!(
            machine.attempt(ROOT_SCOPE, approve, cancel),
            AttemptResult::Pending
        ))
        .is_true();
    }

    #[test]
    fn dirty_attempt_without_confirmation_host_is_rejected() {
        let mut machine = NavigationStateMachine::new();
        machine.add_guard(ROOT_SCOPE, Arc::new(|| true));
        let (approved, approve) = counter();
        let (cancelled, cancel) = counter();

        let result = machine.attempt(ROOT_SCOPE, approve, cancel);

        let AttemptResult::Rejected(effect) = result else {
            panic!("dirty navigation attempt without a host should be rejected");
        };
        effect();
        assert_that!(approved.load(Ordering::SeqCst)).is_equal_to(0);
        assert_that!(cancelled.load(Ordering::SeqCst)).is_equal_to(1);
        assert_that!(machine.has_pending()).is_false();
    }
}
