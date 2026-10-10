# Views, Navigation Scopes, and Dirty Guards

CrudKit view composition has four public concepts:

- `CrudView` is a serializable description of a presentation;
- `CrudViewRegistry` resolves descriptions to renderers;
- `CrudNavigationScope` coordinates dirty guards and navigation attempts for a navigation subtree;
- `CrudNavigation` owns an accepted view and return action inside one navigation scope.

Renderer hosting, pending effects, and committed navigation methods remain private implementation details of
`crudkit-leptos`. The navigation state machine, including scope and dirty guard IDs, is the public, UI-independent
`crudkit_web::navigation::NavigationStateMachine`; applications normally use it only through the Leptos handles.

## Terminology

- A **navigation scope** defines the navigation subtree inspected by one navigation attempt.
- **Navigation** means a `CrudNavigation` value that owns one accepted view and return action.
- An **accepted view** is the `CrudView` currently committed by navigation.
- A **dirty guard** reports unsaved state within one navigation scope.
- A **navigation attempt** is an action waiting for the applicable dirty guards to approve leaving.
- A **confirmation host** is the mounted `CrudInstance` that renders the leave confirmation for a navigation attempt.
- A **leave confirmation** asks whether a pending navigation attempt may discard unsaved state.
- A **return action** is either a `CrudView` destination or an application-owned return callback.

## Open View Descriptions

`CrudView` contains a stable string `name`, a JSON `payload`, and an optional `SerializableId` `subject`.

Applications may define any stable name and payload. Typed payload helpers serialize and deserialize
application data without requiring CrudKit to know the concrete type. An entity-oriented custom view sets
`subject` so nested instances can resolve a parent entity without recognizing its view name.

CrudKit's standard descriptions are:

| Constructor | Stable name | Subject |
| --- | --- | --- |
| `CrudView::table()` | `crudkit.table` | none |
| `CrudView::create()` | `crudkit.create` | none |
| `CrudView::read(id)` | `crudkit.read` | `id` |
| `CrudView::edit(id)` | `crudkit.edit` | `id` |

Renderers registered for them with the registry's typed methods reject payloads; read and edit require a subject;
table and create reject one. Registering a standard name with `register` or `replace` instead transfers payload and
subject validation responsibility to that renderer.

## Renderer Registry and Hosting

A `CrudViewRegistry` is empty by default; CrudKit ships no views. Applications register the views they render in
one map and dispatch path. Each entry is only a renderer. There is no separate host, mount-time binding step,
validator registry, renderer enum, or kind dispatch.

- `table`, `create`, `read`, and `edit` register a standard view; `read` and `edit` renderers receive the subject id.
- `register(name, renderer)` adds an application-defined view, which receives the opened `CrudView`, and rejects an
  occupied name.
- `replace(name, renderer)` deliberately replaces any renderer.

Unknown names and standard descriptions with the wrong subject or a payload produce a visible `role="alert"` region
and a tracing error. They do not silently select another view.

`CrudViewOutlet` renders the accepted view in a child owner with private navigation for it. That owner provides a
`CrudInstanceContext` containing the same navigation, so renderers, hooks, and atoms observe it through
`use_crud_navigation`; renderers take no navigation argument. The view's navigation scope also becomes the enclosing
navigation scope of everything the view renders (see
[Navigation Scope Hierarchy and Dirty Guards](#navigation-scope-hierarchy-and-dirty-guards)). An outlet given another
navigation, such as a child navigation, renders that navigation's views the same way.

The registry is configuration-time state for a mounted instance. It is cloned once and is not reactive.

## Initial and Accepted View

`CrudNavigationScope` has no accepted view. It is a navigation scope for application-owned actions that may discard
one or more descendant drafts.

`CrudNavigation::new(initial)` creates a root navigation scope and owns one accepted-view signal inside it. It has no
second signal for an unaccepted destination. `CrudNavigationScope::child_navigation(initial)` creates an independent
accepted view in a new child navigation scope.

An application route parser may choose or overwrite `CrudInstanceConfig::initial_view` before mounting the
instance. After mounting, the accepted view stays owned by CrudKit and changes only through navigation methods.
CrudKit does not observe a router or mirror an application-owned view signal.

`CrudInstanceConfig::initial_view` initializes navigation created below the enclosing navigation scope. If
navigation is supplied, its accepted view is rendered instead; the configured initial view remains the instance
reset destination.

A table is the default return destination of a newly created navigation. It is not a mandatory root presentation.

## Navigation Scopes and Navigation

All `CrudNavigationScope` values are copyable.

- `CrudNavigationScope::new()` creates a root navigation scope owned by the current Leptos owner.
- `child()` creates a child navigation scope with no accepted view.
- `child_navigation(initial)` creates child navigation with an independent accepted view and return action.
- `attempt(approved, cancelled)` creates a navigation attempt for an application-owned action and inspects dirty
  guards in that complete navigation subtree.
- `guard(is_dirty)` registers a dirty guard in that navigation scope.

Navigation scope cleanup is automatic. Cleaning up a root navigation scope disposes its complete navigation scope
tree without running pending callbacks. Cleaning up a child navigation scope removes only that navigation subtree and
resolves a navigation attempt targeting that subtree through its cancellation callback unless ancestor cleanup
disposes the complete navigation scope tree in the same cleanup turn.

All `CrudNavigation` values are copyable.

- A plain copy shares its navigation scope, accepted view, and return action.
- `scope()` returns the navigation scope used by the navigation.
- The private `scoped()` creates a child navigation scope while sharing the accepted view and return action.
  `CrudInstance` uses one for its mount and another for each rendered view.
- `child(initial)` creates child navigation with an independent accepted view and return action.

Child navigation participates in the ancestor's navigation scope tree and shares its signal indicating a pending
navigation attempt. Its independent accepted view allows a drawer or inline editor to navigate without replacing the
parent's accepted view.

## Application-Owned Actions

URL changes, browser history, drawer selection, and other application-owned state are not alternate
owners of a CrudKit view. Application code attempts those changes through `CrudNavigationScope::attempt` when the
action covers a navigation subtree, or `CrudNavigation::attempt` when it covers one navigation subtree. Approval may
update application state; cancellation may restore application state when an application-owned change was already
observed.

When the destination is another registered CrudKit view, application code uses `navigate(view)` instead.
That method keeps the accepted view inside navigation and changes it only after dirty guards approve.

## Return Actions

`return_to(view)` configures a view destination. `return_with(callback)` configures an application-owned return
callback such as closing a drawer or changing an application route. `return_from_current()` creates a navigation
attempt for the configured return action. The navigation attempt snapshots that return action before the leave
confirmation; reconfiguration while the leave confirmation is open affects later returns, not the pending navigation
attempt.

An application-owned return callback is not an unmount callback. It runs only after an explicit approved return or
a private persistence follow-up. Removing a view, child navigation scope, or whole instance does not itself mean
“return.”

## Navigation Scope Hierarchy and Dirty Guards

Every public navigation scope and navigation carries a private scope identity. A navigation attempt inspects dirty
guards in its target navigation scope and descendants:

- a root attempt includes all mounted descendant editors and drawers;
- a drawer attempt includes only that drawer subtree;
- an inline-editor attempt includes only that editor subtree;
- a dirty sibling does not block a clean child-only attempt.

Default instances and nested managers create their navigation scope below the **enclosing navigation scope**: the
scope of the instance view they are rendered in, or else the scope of the nearest `CrudInstanceMgr`. An instance
nested in a view, such as a related resource shown in an edit view, therefore belongs to that view's subtree.
Leaving the view asks about the nested instance's drafts, while the nested instance's own navigation stays local.

`guard(is_dirty)` registers a `Signal<bool>` for the current navigation scope and unregisters through Leptos cleanup.
The dirty value is sampled when the navigation attempt begins. It is not re-evaluated while the leave confirmation is
open, so the pending navigation attempt resolves according to that original attempt-time decision.

Navigation scope and dirty guard IDs are private to prevent callers from forging ancestry or manually retaining a
registration beyond its owner.

## Leave Confirmation State Machine

One navigation scope tree permits one pending navigation attempt at a time:

1. If the targeted subtree is clean, approval runs immediately.
2. If it is dirty, the accepted view remains mounted and the navigation state machine selects one mounted instance to
   host the leave confirmation. Only that instance's view of the pending navigation attempt shows the leave
   confirmation.
3. A dirty navigation attempt with no mounted confirmation host is rejected through its cancellation effect.
4. Accept or cancel removes the pending navigation attempt before invoking its effect.
5. A navigation attempt resolves at most once.
6. A second navigation attempt while one is pending runs its cancellation effect and cannot replace the first.
7. Removing the pending navigation attempt's target navigation scope or selected confirmation host defers its
   cancellation effect until
   the current cleanup turn finishes, allowing navigation scope tree disposal to suppress the callback.
8. Disposing the complete navigation scope tree drops registrations and pending work without invoking approval,
   cancellation, return, or application callbacks.

The state machine lives in `crudkit-web`, is UI-independent, and is unit-tested separately from the reactive
navigation handles and the leave confirmation.

In a browser, publication of a newly pending navigation attempt is deferred to the next microtask. This keeps a
containing modal's Escape event from also reaching and immediately cancelling the leave confirmation it just opened.
The state machine records the pending navigation attempt synchronously, so another navigation attempt still cannot
replace it during that interval.

`CrudInstanceMgr` owns the default root navigation scope for its navigation subtree. Default instances and nested
managers create child navigation scopes of the enclosing navigation scope. An instance using supplied navigation
deliberately uses its supplied navigation scope instead. [Instances and Composition](instances-and-composition.md)
owns the mounted-instance lifetime contract.

## Built-In Behavior

Table-row activation follows the table's `CrudRowAction` (read, edit, nothing, or a callback). `CrudReadButton` and
`CrudEditButton` navigate to read or edit, and `CrudCreateButton` navigates to create.

`CrudReturnButton` calls `return_from_current()` and uses return-oriented behavior rather than assuming the destination
is a list. Create and edit forms register their drafts through `guard`; they do not own separate signals for pending
navigation attempts or leave confirmations. Instance reset also creates a navigation attempt.

A form's `CrudSaveFollowUp` can, after a successful save:

- open the edit view of the saved entity (`Edit`, the create form's default);
- resolve the saved entity's ID to an arbitrary view (`View`);
- perform the configured return (`Return`);
- open a fresh create view (`CreateAnother`);
- stay (`Stay`, the edit form's default).

## Committed Persistence Follow-Ups

Navigation triggered after successful persistence uses private committed methods and bypasses dirty guards; otherwise
the just-completed save would immediately prompt again.

- edit save-and-return performs the configured return;
- create performs its chosen view or return follow-up;
- successful single deletion performs the configured return on the navigation used for deletion when that
  navigation's accepted view has the deleted entity as its subject; a view return reloads the remaining data, while
  a return callback leaves refresh or unmount behavior to the application. Other views, such as a table, stay and
  reload;
- mass deletion stays in the accepted view and reloads data.

`CrudSaveFollowUp::Stay` performs no navigation. A create form then starts a fresh draft from the default model, so it
is clean again; an edit form takes the saved entity as its new baseline.
