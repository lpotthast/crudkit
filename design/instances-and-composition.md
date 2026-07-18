# Instances and Composition

`CrudInstance` is CrudKit's non-generic Leptos host. It combines a concrete resource configuration,
type-erased model handling, reactive UI state, a view registry, navigation, field renderers, actions,
and optional parent-resource scoping.

View and navigation behavior belongs to
[Views, Navigation Scopes, and Dirty Guards](views-and-navigation.md). This document owns the mounted instance and
component-composition boundary.

## Configuration and Mounted State

`CrudInstanceConfig` contains two kinds of input:

- values represented as mounted signals or reset defaults, such as API URL, initial view, list columns,
  form elements, pagination, ordering, and base condition;
- configuration treated as static for the mount, such as resource name, request executor,
  `ModelHandler`, actions, controls, view registry, and field-renderer registries.

The split is an implementation boundary, not yet a serialization promise. The mutable subset is not
currently persisted or restored across browser sessions, and most mounted setters are intentionally
private.

When `CrudInstance` creates default navigation below its manager, `config.initial_view` becomes the first accepted
view. When the caller supplies navigation, that navigation's accepted view wins; `config.initial_view` remains the
destination used by instance reset.

The view registry is cloned from static configuration once during mount. Built-in renderers and application
replacements are already complete registry entries; `CrudInstance` simply renders the current description.
Adding to the original registry afterward is not reactive.

## Required Manager and Instance Names

Every `CrudInstance` expects a `CrudInstanceMgrContext`, normally supplied by rendering
`CrudInstanceMgr` above all instances in that component tree. The manager records each instance name and current
view and provides the `CrudNavigationScope` used by default child instances.

A top-level manager creates and owns a root navigation scope. A nested manager creates a child of the nearest manager
navigation scope, so ancestor attempts include nested dirty editors while attempts through the nested manager remain
local. The manager context exposes `navigation_scope()` for application-owned route changes, modal closure, and other
actions that may discard descendant drafts.

An application may supply `navigation_scope` to `CrudInstanceMgr`. The manager mounts a private child below the
supplied navigation scope, leaving that scope caller-owned. This supports shells that must retain a scope outside the
manager's descendant context.

Names are lookup keys for nested-resource configuration and are expected to be unique within one manager.
Each registration belongs to the Leptos owner that registered the instance and is removed when that owner is cleaned
up. Registering the same name replaces the previous manager entry; cleanup of the previous owner does not remove the
newer registration. A manager therefore exposes only mounted instances and does not create a multi-value namespace.

## Context as a Shared Handle

`CrudInstanceContext` is `Copy`, but a copy is not a state snapshot. Its signals and stored values still
point to the same arena-owned instance state.

Copying a context therefore:

- does not duplicate pagination, ordering, deletion requests, or reload state;
- does not clone a form draft;
- does not fork navigation or callbacks;
- remains valid only within the Leptos owner lifetime of the values it references.

New context fields that own non-`Copy` data belong behind a signal or stored value so this handle meaning
stays explicit.

## Owner and Context Isolation

Each instance renders its subtree inside `<Provider value=CrudInstanceContext>`. Every registered view
gets another child owner whose context contains the same scoped navigation passed to its renderer. Each
list view similarly provides its own `CrudListViewContext`. These owner boundaries keep same-typed
contexts from sibling instances and views from overwriting or leaking into one another.

Custom field rendering gets another child closure/owner boundary. A custom field may itself provide
contexts or mount a nested CrudKit instance without affecting a sibling custom field.

This isolation is a correctness requirement, not incidental markup. Replacing the providers with direct
`provide_context` calls in a shared owner can make sibling instances observe the wrong resource or list
state.

## List State Ownership

Paging, items per page, ordering, base condition, and the reload token belong to the instance. The current
page and count requests both observe the reload token, so reloading refreshes the list and its count.

Selection belongs to the mounted `CrudListViewContext`, not `CrudInstanceContext`. Select-all reads only the
currently loaded page. Selection clears on the reload token, but not when page, ordering, condition, or loaded data
changes. `all_selected` currently compares counts rather than entity membership, so stale selection can cross pages.

Mass deletion derives one equality condition per selected entity ID and combines multiple IDs with
`Condition::Any`. After the operation, the instance reload token refreshes list data and count.

Current list controls are incomplete: filtering is disabled and does not affect requests; read, edit, and delete
availability is hard-coded to allowed; and the legacy additional-row-action input is discarded before its unfinished
trigger path. `CrudBuiltinViewControls` does not govern table controls.

## Concrete Models to Reactive Fields

`ModelHandler` captures concrete create, read, and update types before mounting the non-generic instance. It owns:

- raw JSON deserialization for reads and writes;
- conversion from read model to update model;
- construction of the default create model;
- creation of per-field `ReactiveField` maps;
- lookup of a create field by name for parent prefill.

Create starts from `CreateModel::default()`. Edit and read fetch a `ReadModel`, convert it to
`UpdateModel`, and build update-field signals.

A form keeps both a concrete erased model and one `ReactiveField` signal per field. Input changes update
both. This gives individual controls fine-grained reactivity while preserving a complete DTO for save,
action callbacks, equality, and dirty comparison.

Create compares its current DTO with the original default DTO. Edit compares its current DTO with the
last accepted server entity. Those dirty signals are registered with navigation as described in
[Views, Navigation Scopes, and Dirty Guards](views-and-navigation.md).

## Field Renderer Resolution

Create, read, and update fields have separate renderer registries. For a rendered field, current resolution is:

1. use the per-instance renderer registered for that exact erased field;
2. otherwise choose the default renderer for its `ValueKind`.

There is no implemented per-layout-position renderer tier, even though an older source comment describes one.

Default rendering covers common primitives. `OffsetDateTime` and array defaults currently render placeholder
text. A field classified as `Other` renders a visible invalid-configuration alert with its current
value; the application must register a custom renderer.

Missing fields in a signal map and incompatible model or field downcasts violate typed configuration
invariants and may panic rather than render a recoverable error.

Form layout descriptions and value semantics are defined in [Data Contracts](data-contracts.md).

## Parent and Nested Resources

`CrudParentConfig` relates a child instance to another currently registered instance by:

- parent instance name;
- one field from the parent's ID;
- the child field that stores that reference.

The parent ID is derived from the parent view's `CrudView::subject`, not from a hard-coded read/edit view
name. Custom entity-oriented parent views therefore participate by setting a subject.

When the subject contains the configured referenced field, the child:

- builds an equality condition on its referencing field;
- ANDs that condition with its configured base condition for scoped reads and updates;
- copies the ID component into the default create model.

If the parent has no subject, the referenced ID field is missing, or the value cannot become a condition,
the child has no parent condition and emits a diagnostic. Create prefill is less defensive: once a parent ID
exists, a missing referenced ID field or unknown child create field violates configuration and may panic.
This mechanism is UI composition, not server-side authorization.

`CrudParentConfig` is appropriate only when the parent is an active instance under the same manager.
Applications with route-owned or otherwise application-owned parent state should construct the condition and
create defaults explicitly.

## Reset

`CrudInstanceContext::reset()` creates a navigation attempt. If any targeted dirty guard reports unsaved state, it uses
the shared leave confirmation before applying reset.

The current reset clears deletion requests and mounted create actions, restores page number, items per
page, and ordering from the initial config, and navigates to `config.initial_view` through the private
committed path. Other static configuration and values without public mutation do not get rebuilt.

## Navigation Scope Composition and Supplied Navigation Lifetime

Every instance creates a private mounted navigation scope. By default, it creates navigation with an independent
accepted view in a child of the nearest manager navigation scope. Internal navigation attempts target that child, so
dirty siblings do not block one another. Attempts through the manager navigation scope include every default instance
and nested manager below it.

A caller may instead supply another `CrudNavigation`, including navigation created below an application-owned
navigation scope. Supplied navigation deliberately overrides the manager navigation scope. The navigation remains
caller-owned and reusable after the instance unmounts; cleanup removes only the private mounted navigation scope and
its descendants. Sibling dirty guards and navigation remain registered.

Root `CrudNavigationScope` and `CrudNavigation` constructors register navigation scope tree disposal with the current
Leptos owner. Child navigation scope and child navigation constructors similarly register navigation subtree removal.
Applications do not call a separate disposal API.

If a navigation attempt targeted the removed navigation scope or selected it as the confirmation host, cleanup
resolves the attempt through its cancellation effect. Unrelated pending navigation attempts remain registered.

## Actions and Styling

Resource and entity actions, including application-placed create controls, are defined in
[Actions](actions.md). Generated SCSS ownership is defined in
[Theming and Generated Assets](theming-and-generated-assets.md).
