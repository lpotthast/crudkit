# CrudKit TODO

This backlog records limitations and actionable issues found while auditing the current implementation. The design
documents describe behavior as it exists; this file describes outcomes that should change it. Remove an entry only
after the implementation, tests, and owning design document agree.

Priorities are relative:

- **P0:** correctness, data integrity, or security-sensitive behavior;
- **P1:** contract, reliability, or product-completeness work;
- **P2:** maintainability, portability, or deliberate API cleanup.

## Conditions and Shared Data

- [ ] **P1 - Report rejected conditions as client errors.** Unknown columns, unconvertible values, and
  `SeaOrmRepoError::UnsupportedCondition` all surface as `CrudError::Repository` (500), although the request is at
  fault. Map them to 400 Bad Request. See [SeaORM Adapter](design/sea-orm-adapter.md).
- [ ] **P0 - Correct JSON condition conversion.** `to_json_value` rejects `ConditionClauseValue::Json` and turns
  string input into `Value::String` rather than `Value::Json`.
- [ ] **P1 - Align the condition wire domain with supported field values.** Decide which array and temporal values are
  supported, add the missing vector forms or narrow the public contract, and make string-to-type parsing explicit.
- [ ] **P1 - Select the entity of an update by its ID.** `UpdateOneV1` selects its entity by a condition and has no
  order, so a missing or match-all condition updates whichever entity the repository returns first. CrudKit's clients
  refuse to build the condition from an empty ID, but the server cannot tell an ID condition from any other. Carry
  the ID as delete-by-ID does, in a new wire format version, and validate it the same way.
- [ ] **P1 - Return diagnostic ID reconstruction errors.** Replace the bare `Option` from
  `Id::from_serializable_id` with an error that identifies missing, extra, mistyped, or misnamed ID fields.
- [ ] **P1 - Validate type-erased configuration before use.** Detect incompatible models, fields, handlers, renderers,
  and action payloads during instance construction instead of relying on downcast panics at interaction time.

## HTTP and Client Contracts

- [ ] **P0 - Define an explicit write-result model.** The server returns `SavedV1<Model>`, while clients deserialize
  its entity as `UpdateModel`. Remove the accidental JSON-compatibility requirement by sharing a dedicated response
  type.
- [ ] **P1 - Generate resource names and paths from one contract.** Bodies are shared through
  `crudkit_wire_format::v1`, but backend and frontend still declare resource names and paths independently, allowing
  silent drift.
- [ ] **P1 - Replace dynamic typetag stripping with an explicit wire conversion.** `Dyn*Model::to_untagged_json`
  currently requires exactly one outer typetag object entry and unwraps the serialized value before sending it.
- [ ] **P1 - Complete client-provider route parity.** Add or deliberately exclude the typed provider's delete-one and
  delete-many so its API matches the generated server surface. The REST data source uses delete-by-ID and delete-many.
- [ ] **P1 - Serve a product's typed, versioned REST API.** A product with typed, versioned REST APIs, such as
  Agnite's `/v1`, should be able to build its resources on crudkit-rs and its sibling crates. CrudKit then serves the
  product's public API without the product writing the routes by hand, while the product keeps its own records in the
  same transaction. The product needs:
  - typed routes: `GET`, `POST`, `PATCH`, and `DELETE` on resource paths as an alternative to the `POST` routes of the
    wire format, with filters and sorting that a resource declares instead of the general condition language;
  - nested resources: a server-side scope of parent IDs, in which each ID resolves inside its parent or the request
    fails with 404;
  - optimistic concurrency: an opaque version in `ETag` and `If-Match`, with 428 for a missing and 412 for a stale
    version (see *Add an optimistic-concurrency contract* below);
  - pages by cursor, without a total count;
  - updates as JSON merge patches, in which `null` removes a field, with a message for each change;
  - asynchronous outcomes: 202 with the resource in its new state, such as `deleting`;
  - problem details (RFC 9457) with the paths of the fields, for validation failures and for refusals;
  - transactions: every operation, hook, and repository call runs inside a transaction that the caller provides, so
    that the product can write its own records in it, such as a change row with author, origin, and message;
  - authorization by scope before any read, and a 404 that hides a resource the caller may not see;
  - the product's own schema: no shared validation table, read views qualified by schema, and migrations that only
    go forward;
  - write-only fields, such as secret values, which are never read back, never `Debug`, and never traced;
  - actions that are not CRUD, declared next to a resource, so that its routes and their documentation come from one
    place.
- [ ] **P2 - Consider a gRPC API next to the HTTP API.** Idea for later, not yet investigated: offer the CRUD
  operations over gRPC as well, built on the same versioned wire format as the HTTP routes.
- [ ] **P2 - Decide whether the POST-only CRUD surface is permanent.** If it is, document infrastructure and caching
  implications; otherwise introduce conventional methods without losing rich request bodies or compatibility.

## Backend Atomicity and Concurrency

- [ ] **P0 - Make mutation outcomes truthful and define after-hook semantics.** A request can return an error after
  storage changed because rejecting after hooks and validation persistence run outside the mutation. Put rejection in a
  transaction, make post-commit hooks non-rejecting, or report committed-with-follow-up-failure explicitly. See
  [Backend Operations](design/backend-operations.md#atomicity-and-failure-boundaries).
- [ ] **P1 - Make validation cleanup after delete reliable.** Best-effort cleanup can leave orphaned validation rows;
  use the entity transaction, a durable retry, or database-level referential cleanup.
- [ ] **P1 - Preserve every batch-delete failure.** Do not ignore `after_delete` failures. Return enough information
  to distinguish repository, hook, validation, cleanup, and notification outcomes and to resume partial work safely.
- [ ] **P1 - Make batch-delete traversal deterministic.** Re-fetching the same unbounded condition without a cursor
  can let one entirely undeletable batch stop processing even when later matching rows could be deleted.
- [ ] **P1 - Add an optimistic-concurrency contract to `crudkit-rs`.** Its updates have no revision, compare-and-swap
  condition, or conflict response, so concurrent edits can overwrite one another. The frontend's data sources already
  carry versions and refuse outdated requests ([Data Sources](design/data-sources.md#versions)); give the generated
  routes and the REST data source versions too, in a new wire format version.
- [ ] **P2 - Replace the batch-size memory heuristic.** `size_of::<Model>() * 3` does not measure heap-owned data.
  Use a configurable row limit, storage-side batching, or a measured budget.

## Validation and Collaboration

- [ ] **P0 - Implement or remove aggregate validation.** `run_global_validation` manages state but never invokes
  validators, persists results, or broadcasts a full result. Before activation, make full `by_entity` JSON-safe and
  preserve resource-general findings during conversion. Run real work outside request latency, retain the
  one-running/one-pending guarantee, and add composite-ID round-trip tests. See
  [Validation and Collaboration](design/validation-and-collaboration.md#current-aggregate-validation-boundary).
- [ ] **P0 - Normalize validation severity and timing.** Resolve post-create Critical findings after insertion,
  discarded pre-create Major findings, ignored delete Major findings, and the absence of post-update model validation.
  Make `Saved<T>` and HTTP behavior describe exactly what can be returned.
- [ ] **P0 - Propagate unified validation save failures.** `UnifiedValidationRepository::save_all` logs individual
  failures and returns `Ok(())`; callers cannot know that validation state is incomplete.
- [ ] **P1 - Make validation replacement atomic with entity updates.** Update currently writes the entity, deletes old
  findings, and saves new findings as separate operations that can leave missing or stale status.
- [ ] **P1 - Define durable collaboration delivery.** Current broadcasts add request latency but swallow delivery
  failures. Choose an outbox/retry protocol or explicitly decouple best-effort notifications from durable events.
- [ ] **P1 - Add collaboration identity and ordering.** Messages need enough revision, origin, correlation, and
  ordering information for receivers to reject stale state and deduplicate retries.
- [ ] **P1 - Limit broadcasts to the entities a client may read.** A message reaches every client that may read its
  resource (`Audience`), also when lifecycle hooks would hide the entity from that client, e.g. by tenant. Give
  services each recipient's authentication, and let the resource decide per recipient and entity, e.g. through a
  read-visibility policy that both `before_read` and broadcasts use.
- [ ] **P1 - Add the frontend collaboration consumer.** Implement refetch/invalidation and validation-state handling,
  including duplicate, delayed, missing, and reordered messages.
- [ ] **P1 - Define clear-state broadcasts consistently.** Specify when empty partial results are emitted, especially
  after creation, so receivers can distinguish “not evaluated” from “known clear.”
- [ ] **P2 - Reconcile the legacy validation persistence surface.** Remove or integrate `CkValidationModel` and the
  per-resource traits; they are not used by `UnifiedValidationRepository`.
- [ ] **P2 - Design scalable validation-ID lookup.** The unified table stores composite IDs as JSON without a normal
  B-tree index. Add a database-appropriate indexing strategy or a queryable normalized key.

## Leptos Instances, Lists, and Navigation

- [ ] **P2 - Build the table atoms on Leptonic's `Table` atoms.** Leptonic's `TableHeader` renders every column header
  itself, without per-header content, classes, or attributes, so CrudKit keeps its own header atoms for now and
  mirrors Leptonic's rows and cells (focus-visible and hover states, cells following column moves). Once headers can
  be composed, `use_crud_table` and the table atoms could shrink to the DOM-safe keys and row behavior they add.
- [ ] **P2 - Add browser tests for the atoms.** The full-stack example's UI was verified manually (hydration,
  grid keyboard navigation, header sorting, selection, dialogs, number fields, create/edit/delete, notifications);
  automate these checks.
  Run them headless or in a visible window: Chrome pauses `requestAnimationFrame` in occluded windows, which
  suppresses Leptonic's focus restoration after overlays close.
- [ ] **P2 - Publish hook test support once an application needs it.** The native executor, the in-memory data
  source, and the in-memory server in `crudkit-leptos`'s `test_support` module are crate-private. Expose them, e.g.
  behind a feature, when applications want to unit-test their own hook-based views.
- [ ] **P2 - Let optional `bool` fields return to null.** The default checkbox can only set `true` or `false`.
- [ ] **P2 - Offer a rich-text renderer.** The Tiptap-based editor was removed with Leptonic's legacy components; JSON
  fields now use a plain text area.
- [ ] **P1 - Add list filtering.** The REST data source has a base condition, but no atom or hook lets users filter a
  list. A filter would reach the data source with the list request, which a data source may refuse.
- [ ] **P1 - Expose entity control policy.** Deleting a listed entity is always offered (`can_delete` of table rows is
  `true`), and read, edit, and create availability have no policy either. UI controls remain presentation, not
  security.
- [ ] **P2 - Make page numbers nonzero by construction.** The current page already moves into the page range when the
  page size changes or the count shrinks (`PageNr::clamp_to`); a zero page is still representable.
- [ ] **P1 - Reject duplicate instance names.** `CrudInstanceMgr` silently replaces the previous registration, which
  can redirect nested-resource lookup to the wrong instance.
- [ ] **P1 - Validate and type parent-resource mappings.** `CrudParentConfig` stores field names as strings. A bad
  mapping leaves the parent out of the instance's scope, so the REST data source lists unscoped entities, or names a
  field the create model lacks, which goes unnoticed. Resolve both fields once during instance construction and report
  one configuration error.
- [ ] **P1 - Add field atoms for arrays.** `CrudFieldControl` has no default for list values and shows a configuration
  error; applications must register a renderer.
- [ ] **P2 - Decide the mounted-configuration mutability contract.** Either persist and restore the mutable subset and
  make intended changes reactive, or tighten the API and comments around mount-time-only configuration.
- [ ] **P2 - Support server-capable read/edit loading where required.** Those views currently use `LocalResource` and
  cannot perform their data load during server rendering. Loading on the server also needs `Send` futures from data
  sources, whose trait currently allows futures that are not `Send`.
- [ ] **P2 - Show the range of a page of a list paged by cursor.** Without a total, `CrudPaginationState::range` is
  `None`, although the page's first and last positions, e.g. "21-30", are known.
- [ ] **P2 - Add localization.** `CrudUiTexts` offers German (the default) and English texts, but value formatting
  (`format_crud_value`: dates, booleans, numbers) is fixed and not locale-aware, and applications cannot provide their
  own formatter.
- [ ] **P2 - Add an application-defined entity label for destructive confirmation.** The delete dialog's default
  message does not name the entity it deletes.
- [ ] **P1 - Manage focus across view changes.** Opening a view, saving (the clean form disables its save button),
  confirming a deletion whose trigger disappears, and leaving a view all drop focus to `<body>`. `CrudViewOutlet` could
  focus the new view (e.g. its first heading or the view's root, as routers do), and confirmation dialogs could return
  focus to a stable element when their trigger is gone.
- [ ] **P2 - Let forms leave the failures they explain to `CrudFormError`.** A failed save notifies its failure, as the
  form's `CrudSaveNotifications` select, also when `CrudFormError` explains it in the form, so screen readers announce
  it twice. A choice that notifies successes only would let applications avoid that.
- [ ] **P2 - Label the problems of fields that a form does not show.** `CrudFormError` names such a field by its name
  in the model, because the form knows no label for a field it does not show. A label from the form's layout would read
  better.
- [ ] **P2 - Show save violations.** `CrudFormHandle::violations` carries the non-critical violations of the last save,
  but no atom shows them, and the example does not either.
- [ ] **P2 - Format read-only values in inputs.** Disabled or read-only text inputs show their codec's edit format
  (e.g. ISO date-times) while lists and details show `format_crud_value`'s display format.
- [ ] **P2 - Request toast exit states from Leptonic.** Leptonic's toasts expose no `data-entering`/`data-exiting`, so
  application styles can animate them in only. Raise it with Leptonic rather than working around it here.

### `crudkit-leptos` API follow-ups (review of 2026-10-05)

- [ ] **P2 - Align configuration naming.** `CrudInstanceConfig::elements` is the update layout used by edit and read
  views, `list_columns` becomes `headers` internally, registries are `read_field_renderer` in the configuration but
  `read_field_renderers()` on the context, and `read_field_renderer` only affects listed entities. One word per
  concept is missing too: the read view is `CrudView::read`, `use_crud_read`, and `CrudReadButton`, but its form atom
  is `CrudDetails`; the edit view is `CrudEditForm` and `EDIT_VIEW`, but `CrudEntityViewKind::Update`.
- [ ] **P2 - Make `CrudParentConfig` deserializable and typed.** Its `&'static str` name only deserializes from
  `'static` input, and `referencing_field` is still a string.
- [ ] **P2 - Report entity actions executed without an entity.** `execute` silently does nothing while the input
  entity is missing and leaves the action requested.

## Actions and Persistence Follow-Ups

- [ ] **P1 - Validate action IDs and lifecycle.** Reject duplicate IDs and requested/executing markers, disable entity
  actions without model state, make completion idempotent, and provide cancellation or timeout when completion is lost.
- [ ] **P1 - Give success and failure distinct aftermath semantics.** The instance currently handles `Ok` and `Err`
  identically. Define failure-specific defaults and preserve structured error information.
- [ ] **P1 - Integrate actions with dirty navigation when they replace a surface.** Provide a guarded helper or
  require an explicit navigation outcome so actions cannot accidentally discard drafts.
- [ ] **P1 - Resolve the create state of entity actions.** The entity-action enum includes create, but entity actions
  act on an update model, so their buttons stay disabled in create forms. Remove the state or pass the create draft.

## SeaORM, Build, and Maintenance

- [ ] **P1 - Resolve the `sqlx-postgres 0.7.4` future-incompatibility warning.** Inspect Cargo's report and update or
  patch the SeaORM/SQLx dependency chain before a future Rust release turns the warning into a build failure.
- [ ] **P1 - Log a diagnosis of database failures.** CrudKit logs repository errors by their `Display` text, and
  `SeaOrmRepoError::Db` displays only that the database failed, because the database's error can contain values, such
  as PostgreSQL's `Key (email)=(…) already exists`. Add what identifies a failure without values, such as the SQLSTATE
  code and the name of the violated constraint.
- [ ] **P1 - Support explicit SQL NULL updates.** Generated optional update fields currently mean “write when
  `Some`, leave unchanged when `None`” and cannot distinguish set-null from unchanged.
- [ ] **P2 - Separate portable and PostgreSQL-specific read-view support.** Make the database requirement explicit in
  APIs or provide adapter implementations for other supported SeaORM backends. `crud_read_view.rs` hard-codes
  `DbBackend::Postgres` and `jsonb_build_array`, and `impl_read_view_migration!` unwraps; the full-stack example
  writes its SQLite read views by hand.
- [ ] **P2 - Ship a no-op `CollaborationService`.** Applications without live collaboration, including the example,
  must currently write their own.
- [ ] **P2 - Document or re-export derive and route-macro dependencies.** The web `CkField` derive needs `typetag`,
  and `impl_add_crud_routes!` needs `paste`, `axum-macros`, `utoipa`, and `tracing` in the calling crate. Create
  models also need a hand-written, panicking `ErasedIdentifiable` implementation.
- [ ] **P2 - Consolidate error infrastructure and dead compatibility types.** Evaluate the `thiserror`/SNAFU split and
  remove obsolete aliases and empty components after compatibility review.
- [ ] **P2 - Reconcile stale source documentation with the design set.** Several Rust comments still describe removed
  model relationships, aggregate validation as implemented, or two active validation storage modes. Keep
  [the design index](design/crudkit.md) authoritative and link to it instead of copying architecture prose.
- [ ] **P2 - Decide whether HTTP integrations remain inside runtime crates.** Axum lives in `crudkit-rs` and Reqwest
  in `crudkit-web`; extract adapters only if the supported integration boundary requires independent versioning.
