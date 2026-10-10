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
  `ModelHandler`, actions, view registry, and field-renderer registries.

`CrudInstanceConfig::new::<R>(api_base_url, executor)` derives the resource name and model handler from a
`crudkit_web::resource::Resource` and starts everything else empty or at its default; applications complete it with
struct update syntax.

The split is an implementation boundary, not yet a serialization promise. The mutable subset is not
currently persisted or restored across browser sessions, and most mounted setters are intentionally
private.

When `CrudInstance` creates default navigation below its enclosing navigation scope, `config.initial_view` becomes
the first accepted view. When the caller supplies navigation, that navigation's accepted view wins;
`config.initial_view` remains the destination used by instance reset.

A configuration without a view registry uses the registry provided with `provide_crud_view_registry` above the
instance, or an empty registry, in which every opened view renders a visible error. This lets an application render
all its instances with its own views. The view registry is cloned from static configuration once during mount; its
renderers are complete entries, and `CrudViewOutlet` simply renders the current description. Adding to the original
registry afterward is not reactive.

## Required Manager and Instance Names

Every `CrudInstance` expects a `CrudInstanceMgrContext`, normally supplied by rendering
`CrudInstanceMgr` above all instances in that component tree. The manager records each instance name and current
view and provides the `CrudNavigationScope` used by default child instances that are not rendered in another
instance's view.

A top-level manager creates and owns a root navigation scope. A nested manager creates a child of the enclosing
navigation scope (the scope of the instance view it is rendered in, or else of the nearest manager), so ancestor
attempts include nested dirty editors while attempts through the nested manager remain local. The manager context
exposes `navigation_scope()` for application-owned route changes, modal closure, and other actions that may discard
descendant drafts.

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

Each instance renders its subtree inside `<Provider value=CrudInstanceContext>`. `CrudInstance` renders no markup, so
applications lay out an instance themselves with `CrudViewOutlet`, atoms, and hooks. Every rendered view gets another
child owner whose context carries the view's scoped navigation. Each `CrudList` similarly provides its own
`CrudListState`. These owner boundaries keep same-typed contexts from sibling instances and views from overwriting or
leaking into one another. An instance also hides the table row, form, and entity contexts of its surroundings, so the
atoms of an instance nested in a row or form act on the nested instance, see
[Leptos Hooks and Atoms](leptos-hooks-and-atoms.md).

Field rendering gets another child owner boundary. A field renderer may itself provide contexts or mount a nested
CrudKit instance without affecting a sibling field. The field's `CrudFieldState` is passed to its renderer and
provided in that owner for `use_crud_field`, together with its type-erased `CrudFieldBinding`.

This isolation is a correctness requirement, not incidental markup. Replacing the providers with direct
`provide_context` calls in a shared owner can make sibling instances observe the wrong resource or list
state.

## List State Ownership

Paging, items per page, ordering, base condition, and the reload token belong to the instance. The current
page and count requests both observe the reload token, so reloading refreshes the list and its count. When the count
shrinks below the current page, e.g. after deletions, the list moves to its last page.

An edit form keeps its unsaved changes when its entity reloads: the reloaded version only becomes what the draft is
compared against. Other forms and clean drafts take the reloaded version.

Selection belongs to the mounted list's `CrudListState`, not `CrudInstanceContext`. It is a
`crudkit_web::list::Selection`. Select-all reads only the currently loaded page. Whenever the loaded page changes,
through paging, ordering, condition changes, or a reload, entities that are no longer displayed are deselected, and
a failed or pending load clears the selection. `all_selected` checks membership of every displayed entity.

Page arithmetic (`page_count`, `PageNr::clamp_to`, `PageOptions`, and the offered page sizes) and the ordering toggle
(`toggle_order`) live in `crudkit_web::list`. Changing the page size clamps the current page into the new page range.

Mass deletion derives one equality condition per selected entity ID through
`crudkit_core::condition::condition_matching_any_id` and combines multiple IDs with `Condition::Any`. The conversion
is fallible: no IDs, an ID without components, or an ID value that cannot be compared for equality aborts the
operation with `RequestError::InvalidRequest` before a request is sent, because a weakened condition could delete
unrelated entities. After the operation, the
instance reload token refreshes list data and count.

Current list controls are incomplete: there is no filtering beyond the instance's base condition, and read, edit,
and delete availability is hard-coded to allowed.

## Concrete Models to Reactive Fields

`crudkit_web::model_handler::ModelHandler` captures concrete create, read, and update types before mounting the
non-generic instance. It is a `Copy` value of function pointers that only `ModelHandler::new::<Create, Read, Update>`
builds, so its operations always agree on the same three types. It owns:

- deserialization of read and update models, used by the dynamic provider;
- conversion from read model to update model;
- construction of the default create model;
- every field of a model together with its current value;
- lookup of a create field by name for parent prefill.

The instance builds its per-field `ReactiveField` maps from those field values; that conversion is the only
Leptos-specific part.

Create starts from `CreateModel::default()`. Edit and read fetch a `ReadModel`, convert it to
`UpdateModel`, and build update-field signals.

A form keeps both a concrete erased model and one `ReactiveField` signal per field. Input changes update
both. This gives individual controls fine-grained reactivity while preserving a complete DTO for save,
action callbacks, equality, and dirty comparison. Applications can only read a `ReactiveField` and change values
through the form, so the two cannot diverge.

Create compares its current DTO with the original default DTO. Edit compares its current DTO with the
last accepted server entity. Those dirty signals are registered with navigation as described in
[Views, Navigation Scopes, and Dirty Guards](views-and-navigation.md).

## Field Renderer Resolution

Create, read, and update fields have separate renderer registries. For a field rendered by `CrudFieldControl`,
resolution is:

1. use the per-instance renderer registered for that exact erased field;
2. otherwise render the markup the application chose for the field's input kind, or the default composition of field
   atoms for it.

The input kinds follow `CrudInputKind::of`, which maps every `ValueKind` to a toggle, a number field of a
`CrudNumberKind`, a text field with a codec, nothing, or a custom renderer. Custom inputs match on it exhaustively.

A renderer receives one `CrudFieldState`: the field, its `ReactiveField`, mode, layout options, the reactive
values of all fields of the entity, a setter, whether the form saves, and a unique DOM id. `FieldRenderer::render`
runs the renderer once, untracked, in its own reactive owner with that state as context. Inputs are read-only while
their form saves, so the saved entity cannot overwrite input made during the request. Values reported through the
setter update the form; rejected input becomes an input error of the field. Field atoms bind fields to Leptonic's
field atoms through CrudKit's input hooks.

The default compositions cover primitives, strings, JSON (a text area), UUIDs, date-times with and without offset, and
durations (`HH:MM:SS.cc`, preceded by `-` when negative). Fields holding lists or a custom type (`Other`) have no
default and render a visible invalid-configuration alert; the application must register a renderer.

A layout field missing from the model renders a visible alert. Incompatible model or field downcasts violate typed
configuration invariants and may panic rather than render a recoverable error.

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

The current reset clears deletion requests, restores page number, items per page, and ordering from the initial
config, and navigates to `config.initial_view` through the private committed path. `CrudResetButton` triggers it.
Other static configuration and values without public mutation do not get rebuilt.

## Navigation Scope Composition and Supplied Navigation Lifetime

Every instance creates a private mounted navigation scope. By default, it creates navigation with an independent
accepted view in a child of the enclosing navigation scope: the scope of the instance view it is rendered in, or else
the nearest manager navigation scope. Internal navigation attempts target that child, so dirty siblings do not block
one another. Attempts through an enclosing scope include every default instance and nested manager below it; leaving
an edit view, for example, also asks about drafts of instances nested in that view.

A caller may instead supply another `CrudNavigation`, including navigation created below an application-owned
navigation scope. Supplied navigation deliberately overrides the enclosing navigation scope. The navigation remains
caller-owned and reusable after the instance unmounts; cleanup removes only the private mounted navigation scope and
its descendants. Sibling dirty guards and navigation remain registered.

Root `CrudNavigationScope` and `CrudNavigation` constructors register navigation scope tree disposal with the current
Leptos owner. Child navigation scope and child navigation constructors similarly register navigation subtree removal.
Applications do not call a separate disposal API.

If a navigation attempt targeted the removed navigation scope or selected it as the confirmation host, cleanup
resolves the attempt through its cancellation effect. Unrelated pending navigation attempts remain registered.

## Actions

Resource and entity actions are defined in [Actions](actions.md).
