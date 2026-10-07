# Validation and Collaboration

CrudKit models validation as structured findings with provenance, and collaboration as best-effort notifications
about entity identity and validation state. Neither subsystem is a transaction or a concurrency-control protocol.

Complete placement in create, update, and delete pipelines is owned by [Backend Operations](backend-operations.md).

## Validator Types

An `EntityValidator<R>` has a stable `name()` and numeric `version()`, plus three independent entry points:

- `validate_create(CreateModel, trigger)` checks creation input before insertion;
- `validate_model(Model, trigger)` checks a persisted entity after create or before delete;
- `validate_updated(old Model, UpdateModel, trigger)` checks a proposed update without requiring the repository to
  construct the resulting model first.

Default implementations return no findings. Each operation invokes every registered entity validator and retains its
name and version with the internal result.

An `AggregateValidator<R>` describes validation over all entities of a resource. Its public shape is present, but its
execution status is explicitly limited below.

`ValidationTrigger` records whether validation came from a CRUD action or a global run. CRUD triggers also record the
action and whether the check is before or after it.

## Severity and Timing

`Violation` has `Major` and `Critical` severities. Blocking is determined by both severity and where the validation
runs:

- a critical result checked before create, update, or delete prevents the repository mutation and becomes HTTP 422;
- a major result does not block;
- a result produced after persistence cannot roll the write back merely because it is critical.

The last point is a current implementation constraint. Post-create model validation is not filtered to Major, so it
can persist and return a Critical finding after insertion even though `Saved<T>` (on the wire `SavedV1<T>`) currently
describes its contents as non-critical warnings. Critical results that block an operation reach the client in the
`critical_validation_errors` error body; see [HTTP and Data Providers](http-and-data-providers.md#error-translation).

Current operation-specific handling is deliberately not uniform:

- **Create:** Validate the create DTO before insert. If allowed, validate the inserted model again. Pre-create Major
  findings are not carried forward; post-create findings become the saved status.
- **Update:** Validate the old persisted model plus update DTO once before writing. Critical findings block; remaining
  findings replace stored entity validation state. There is no post-update model validation.
- **Delete:** Validate the persisted model before deletion. Critical findings block; Major findings do not become a
  saved or returned delete result.
- **Read:** No validation is run. A read projection may expose already persisted validation status.

## Provenance and Persistence Shape

Internally, violations are grouped by validator identity. Validator versions allow a persistence adapter to replace
obsolete results produced by earlier logic.

`ValidationResultRepository` is storage-neutral. It can delete results by entity or resource, replace results for a
resource, and list typed or type-erased results. The SeaORM implementation is described in
[SeaORM Adapter](sea-orm-adapter.md).

Client-facing aggregates flatten validator identity and group findings by their affected scope:

- `general` targets the resource as a whole;
- `create` targets creation input before an entity ID exists;
- `by_entity` targets a `SerializableId`.

Partial-result `by_entity` serializes as a sequence of `(SerializableId, Violations)` pairs rather than a JSON object.
Composite structured IDs are not valid JSON object keys.

The full-result type still uses `HashMap<SerializableId, Violations>`, which cannot serialize those keys to JSON. The
current typed-to-full conversion helper also omits resource-general findings. These defects are latent while aggregate
execution emits no full result, but must be fixed before that path becomes active.

Validation persistence is separate from entity persistence. Create stores post-insert findings after insertion.
Update deletes all stored findings for the entity and then saves the new set after updating. Delete cleanup happens
after deletion and is best effort.

## Partial and Full Results

A partial result updates only the represented scopes:

- `None` or a missing entity means no new information about that scope;
- `Some(empty)` or an explicitly present entity with no findings means that scope is now known to be clear;
- a non-empty value replaces the known findings for that represented scope.

A full result means every entity in the resource was considered, so a receiver may replace its complete known state.
Full results contain resource-general and per-entity findings; creation findings are inherently partial because they
are tied to an attempted input.

Update always broadcasts a partial result, including an empty one to clear old state. Create currently broadcasts a
partial result only when its post-insert validation found something.

## Current Aggregate-Validation Boundary

`AggregateValidator`, full-result DTOs, full-result collaboration messages, and a three-state global-run coordinator
exist. Actual aggregate execution does not.

`run_global_validation` currently coordinates `IDLE`, `RUNNING`, and `RUNNING_WITH_PENDING`, but the body is a
placeholder: it does not call `resource_validators`, persist their output, or broadcast a full result. Calls are
awaited on the CRUD request path rather than spawned in a background task, although the placeholder completes
immediately.

These types record an intended extension seam, not a guarantee that global validation occurs today.

## Collaboration Protocol

`CollaborationService::broadcast` receives each message in its wire form, `CollabMessageV1`, ready to serialize for
clients. It may deliver them through WebSockets or another application transport. The abstraction is
transport-neutral even where older source comments say “WebSocket.” Operations build the internal `CollabMessage` and
convert it at this boundary.

Messages are notifications, not entity replication:

- every message carries `wire_format_version` and an `event` tagged by `kind`;
- `entity_created` and `entity_updated` carry resource name, entity ID, and a validation-status boolean;
- `entity_deleted` carries resource name and entity ID;
- `partial_validation_result` and `full_validation_result` carry one entry per resource, sorted by resource name,
  whose entity violations are lists rather than maps keyed by ID;
- no message contains the complete entity model.

Current emission order is:

- create: optional partial validation result, then `EntityCreated`;
- update: partial validation result, then `after_update`, then `EntityUpdated`;
- delete: `EntityDeleted` after the after-delete hook and cleanup attempt;
- full validation result: not emitted by current operation code.

Broadcast calls are awaited, so service latency is part of request latency. Delivery errors are logged and swallowed;
they neither fail nor roll back the CRUD operation.

## Consistency Guarantees

Collaboration is best-effort invalidation and status notification. Messages have no revision, origin, correlation ID,
ordering token, or lock token. CrudKit provides no entity lock, optimistic version check, compare-and-swap update,
replay log, or total ordering across requests.

Receivers must tolerate missing, duplicated, delayed, or reordered notifications and should refetch authoritative
data when correctness requires it. `crudkit-leptos` currently has no built-in collaboration consumer or
validation-message deduplication implementation.

## Current SeaORM Save Behavior

The unified SeaORM repository replaces one validator's rows transactionally, but its outer `save_all` loop logs an
individual validator failure and continues, ultimately returning `Ok(())`. Code using that adapter must not currently
assume that a successful `save_all` means every validator result was stored.
