# Actions

CrudKit actions let applications attach behavior and optional UI to a resource or entity without adding
domain-specific operations to the CRUD engine.

Create/save/navigation controls supplied by CrudKit itself are built-in controls. This document concerns
application actions and the outlet used to relocate create controls.

## Resource and Entity Actions

`CrudAction` operates on the resource as a whole. `CrudEntityAction` operates on one current `UpdateModel`
and declares whether it is valid in create, update, or read state.

The state enum includes create, but the built-in create view does not currently mount `CrudActionButtons`.
Declaring an entity action valid for create therefore does not expose it through the built-in UI.

An entity action receives the current update model, including unsaved edits. CrudKit does not save that
model before invoking the action. The action decides whether the current draft is input, whether
persistence is required, and whether later navigation should use `CrudNavigation` to create a navigation
attempt.

Each action has an ID, display name, optional icon, button color, execution callback, and optional
application-rendered view. Entity actions also have their valid states.

IDs are used as UI keys and entries in requested/executing state and are expected to be unique within one
action set. Current configuration does not reject duplicates, so uniqueness is an application invariant.

## Optional Payload UI

An action without a view executes immediately with no payload.

An action with a view first marks itself requested and calls the application renderer with:

- a reactive visibility signal;
- a cancel callback;
- an execute callback accepting an optional type-erased payload;
- for entity actions, the current optional model state.

The application may render a modal, drawer, confirmation panel, or input form. CrudKit owns
requested/executing bookkeeping, not the presentation.

`AnyActionPayload` stores an application payload behind the `ActionPayload` trait. The matching derive
provides cloning, equality, serialization, and downcasting support. A downcast to the wrong concrete
payload panics and represents an action/configuration mismatch.

## Execution and Completion

When execution begins, CrudKit removes the requested marker and adds the executing marker. Buttons use
that state to avoid starting the same action again while its callback is running.

Every action callback receives an `and_then` completion callback. The application must invoke it exactly
once when its asynchronous or synchronous work is complete. Until then, CrudKit keeps the action in its
executing state.

Completion carries `Result<CrudActionAftermath, CrudActionAftermath>`. The current instance handler
applies the same aftermath fields to both branches:

- optionally show a Leptonic toast;
- optionally trigger instance data reload.

The `Ok`/`Err` distinction is currently available to the callback protocol but does not change built-in
aftermath handling. Domain-specific success or failure presentation should be encoded in the provided
toast or surrounding application UI.

Action execution does not automatically create a navigation attempt. If an action closes or replaces a
surface, it must create a navigation attempt through the relevant navigation rather than unmounting the
surface directly.

## Application-Placed Create Controls

`CrudBuiltinViewControls::create_actions_placement` chooses whether built-in create controls render inline
or are published through the instance context.

With `CrudActionsPlacement::External`, `CrudActionsOutlet` can render a selected `CrudActionSlot` in an
application-chosen location. Current slots divide create controls into primary, navigation, and toolbar
groups.

The outlet accepts an optional `CrudInstanceContext`, which lets a parent capture the context through
`on_context_created` and place the controls outside the instance's DOM position. Leaving the create view
clears the published controls.

`CrudCreateActionsPlacement` and `CrudCreateActionsOutlet` remain compatibility aliases for the
generalized names. Despite those generalized names, the current published action bundle is still
create-view-specific.
