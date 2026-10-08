# Actions

CrudKit actions let applications attach behavior and optional UI to a resource or entity without adding
domain-specific operations to the CRUD engine.

Saving, deleting, and navigating are CrudKit's own operations, bound to atoms such as `CrudSaveButton`. This document
concerns application actions.

## Resource and Entity Actions

`CrudAction` operates on the resource as a whole. `CrudEntityAction` operates on one current `UpdateModel`
and declares whether it is valid in create, update, or read state.

`CrudEntityActions` renders the actions valid in the surrounding form's kind. The state enum includes create, but
entity actions act on an update model, which a create form does not have yet: in a create form their buttons stay
disabled.

An entity action receives the current update model, including unsaved edits. CrudKit does not save that
model before invoking the action. The action decides whether the current draft is input, whether
persistence is required, and whether later navigation should use `CrudNavigation` to create a navigation
attempt.

Each action has an ID, display name, optional icon, `CrudActionIntent`, execution callback, and optional
application-rendered view. Entity actions also have their valid states. The intent describes meaning
(primary, secondary, success, warning, danger); renderers decide how to present it.

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
requested/executing bookkeeping, not the presentation. `use_crud_actions_state` exposes that bookkeeping, the configured
actions, and their execution to custom views.

`DynActionPayload` stores an application payload behind the object-safe `ErasedActionPayload` trait; the typed
`ActionPayload` trait carries the serialization bounds. The `CkActionPayload` derive implements both and provides
cloning, equality, serialization, and downcasting support. A downcast to the wrong concrete
payload panics and represents an action/configuration mismatch.

## Execution and Completion

When execution begins, CrudKit removes the requested marker and adds the executing marker. Buttons use
that state to avoid starting the same action again while its callback is running.

Every action callback receives an `and_then` completion callback. The application must invoke it exactly
once when its asynchronous or synchronous work is complete. Until then, CrudKit keeps the action in its
executing state.

Completion carries `Result<CrudActionAftermath, CrudActionAftermath>`. The current instance handler
applies the same aftermath fields to both branches:

- optionally emit a `CrudNotification` through the instance's `CrudNotifier`, which names the instance as its
  `origin` unless the notification already has one;
- optionally trigger instance data reload.

The `Ok`/`Err` distinction is currently available to the callback protocol but does not change CrudKit's
aftermath handling. Domain-specific success or failure presentation should be encoded in the provided
notification or surrounding application UI.

Action execution does not automatically create a navigation attempt. If an action closes or replaces a
surface, it must create a navigation attempt through the relevant navigation rather than unmounting the
surface directly.

`CrudActionsState::bind_resource_action` and `bind_entity_action` return a `CrudActionHandle` for custom buttons: its
`press` requests an action that has a view and executes one without, `is_disabled` covers execution and missing entity
input, and `view` is the action's view to render next to the button. `CrudActionButton` and `CrudEntityActionButton`
use the same handles: they render a Leptonic `Button` and the action's view, an overlay, after it.
