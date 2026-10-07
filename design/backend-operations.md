# Backend Operations

`crudkit-rs` executes CRUD operations against abstract resource, repository, validation, collaboration, authentication,
and lifecycle contracts. It is storage-neutral at the repository boundary. Its current HTTP integration is the Axum
route generator described in [HTTP and Data Providers](http-and-data-providers.md).

## Resource Composition

`CrudResource` binds the model roles from [Data Contracts](data-contracts.md) to the infrastructure used for one
resource:

- a `Repository` for entity storage;
- a `ValidationResultRepository` for persisted validation status;
- entity and aggregate validators;
- a `CollaborationService` for notifications;
- resource-specific shared context;
- one `HookData` type and one `CrudLifetime` implementation;
- an authentication type and per-operation `CrudAuthPolicy`;
- a stable resource type and name.

`CrudContext<R>` holds the runtime instances of those services. Operations receive it through `Arc`, but CrudKit does
not require all resources to share one repository or one application context.

## Repository Boundary

The repository owns storage conversion. CRUD operations pass it application-level create, update, persisted, and read
models; they never construct ORM records directly.

The split between `fetch_*` and `read_*` is intentional:

- `fetch_*` returns the persisted `Model` used for hooks, validation, update, and delete;
- `read_*` returns the possibly enriched `ReadModel` used by query endpoints;
- `update` receives both the fetched persisted model and the update DTO;
- `insert` converts a create DTO and returns the stored model.

This allows a read projection and the main storage record to have different fields and even different ID types.

## Lifecycle Hooks

`CrudLifetime<R>` is the application-policy extension point around generic operations.

- Before-create and before-update hooks may modify their input DTO.
- Before-read may modify limit, offset, ordering, or conditions. It is the appropriate place to add tenant or
  row-visibility constraints on the server.
- After-read may modify returned count, entity, or entity collection without changing the operation kind.
- Delete hooks receive the persisted entity and a description of whether the request selected it by ID, as one match,
  or as part of a batch.
- One `R::HookData::default()` value is created for an operation and threaded from its before hook to its after hook.

`HookError::Forbidden` maps to a permission rejection, `UnprocessableEntity` to a domain-rule rejection, and `Internal`
to a server failure. Hooks own resource-specific policy; validators own structured data-quality findings.

## Current Operation Ordering

The ordering below is observable and matters because there is no transaction around the complete pipeline.

- **Read count, one, or many:** create hook data → mutable `before_read` → repository count or read projection →
  mutable `after_read` → return result.
- **Create:** mutable `before_create` → validate create DTO → insert persisted model → `after_create` → validate
  persisted model → persist and possibly broadcast validation result → broadcast creation → run global-validation
  coordinator → return `Saved<Model>`.
- **Update:** fetch persisted model by condition → mutable `before_update` → validate old model plus update DTO →
  repository update → replace persisted validation state → broadcast partial validation result →
  `after_update` → broadcast update → run global-validation coordinator → return `Saved<Model>`.
- **Delete by ID or one match:** fetch the persisted model matching the ID and the optional request condition →
  `before_delete` → validate model → repository delete → `after_delete` → best-effort validation cleanup →
  broadcast deletion → run global-validation coordinator → return `Deleted`.
- **Delete many:** repeatedly fetch a bounded batch → run before hook, validation, delete, after hook, cleanup, and
  notification per entity → classify per-entity outcomes → stop when empty or when a batch deletes nothing → run
  global-validation coordinator → return `DeletedMany`.

Validation result meaning and the incomplete global-validation path are defined in
[Validation and Collaboration](validation-and-collaboration.md).

## Authentication and Authorization

`CrudAuthPolicy` answers one narrow question for each CRUD operation: must an authenticated value be present?

- `OpenAuthPolicy` makes all operations public.
- `DefaultAuthPolicy` makes reads public and writes authenticated.
- `RestrictedAuthPolicy` requires authentication for every operation.

For a public operation, `RequestContext::auth` may still contain authentication when the caller supplied it. For an
authenticated operation, the generated route rejects a missing auth extension with 401 before invoking the operation.

Claims, roles, ownership, tenant membership, and entity-specific permission are deliberately outside `CrudAuthPolicy`.
Lifecycle hooks inspect `RequestContext` and the fetched entity or mutable read request to enforce those rules. A
client-side condition or hidden button is never an authorization boundary.

## Atomicity and Failure Boundaries

CrudKit does not open a transaction spanning lifecycle hooks, the resource repository, validation persistence, and
collaboration delivery. A repository or application may provide stronger guarantees internally, but the shared operation
API does not coordinate them.

Consequences of the current ordering include:

- an after-create, after-update, or after-delete hook can return an error after the entity mutation already succeeded;
- create or update can return a validation-persistence error after the entity was written;
- update validation notifications are sent before `after_update`, so a later hook error does not retract them;
- collaboration calls are awaited, but delivery failures are logged and swallowed;
- single-delete validation cleanup is best effort after the delete;
- batch delete is intentionally partial-success, and its after-delete failures are ignored;
- no shared compare-and-swap, entity revision, or optimistic-lock contract prevents concurrent lost updates.

Callers must not interpret every error response as proof that storage is unchanged.

## Batch Delete

Mass deletion avoids loading an unbounded result set. It estimates a batch size from the persisted model's stack size
using a 50 MB budget and a heap-overhead multiplier, then clamps the result to 10–1000 entities.

Every entity is reported as deleted, hook-aborted, validation-failed, or errored. If a fetched batch produces no
successful deletion, processing stops to avoid repeatedly fetching the same undeletable entities. This is a best-effort
bulk operation, not an all-or-nothing transaction.
