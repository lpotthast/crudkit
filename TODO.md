# CrudKit TODO

This backlog records limitations and actionable issues found while auditing the current implementation. The design
documents describe behavior as it exists; this file describes outcomes that should change it. Remove an entry only
after the implementation, tests, and owning design document agree.

Priorities are relative:

- **P0:** correctness, data integrity, or security-sensitive behavior;
- **P1:** contract, reliability, or product-completeness work;
- **P2:** maintainability, portability, or deliberate API cleanup.

## Conditions and Shared Data

- [ ] **P0 — Make condition conversion and query translation total and fallible.** Invalid value/operator pairs must
  return structured errors, never panic, call `unimplemented!`, or silently weaken a predicate. Cover scalar `IsIn`,
  non-`IsIn` arrays, heterogeneous and nested arrays, unsupported `Null` operators, `Void`, `Other`, and `i128`/`u128`.
  Add an operator-by-value test matrix. See [Data Contracts](design/data-contracts.md) and
  [SeaORM Adapter](design/sea-orm-adapter.md).
- [ ] **P0 — Implement duration condition conversion.** `ConditionClauseValue::to_time_duration` currently calls
  `unimplemented!`; duration filtering can therefore panic during field-value conversion before a predicate is built.
- [ ] **P0 — Correct JSON condition conversion.** `to_json_value` rejects `ConditionClauseValue::Json` and turns
  string input into `Value::String` rather than `Value::Json`.
- [ ] **P1 — Align the condition wire domain with supported field values.** Decide which array and temporal values are
  supported, add the missing vector forms or narrow the public contract, and make string-to-type parsing explicit.
- [ ] **P1 — Range-check duration conversions.** Both `crudkit-core::TimeDuration` serialization and the SeaORM
  duration newtype cast `whole_microseconds()` to `i64` and can corrupt out-of-range values.
- [ ] **P0 — Validate serialized IDs at server operation boundaries.** Reject empty, duplicate, missing, extra,
  misnamed, and mistyped components before building an ID predicate. Delete-by-ID currently accepts arbitrary entries;
  an empty `SerializableId` becomes a match-all condition. Add route tests for simple and composite IDs.
- [ ] **P1 — Return diagnostic ID reconstruction errors.** Replace the bare `Option` from
  `Id::from_serializable_id` with an error that identifies missing, extra, mistyped, or misnamed ID fields.
- [ ] **P1 — Validate type-erased configuration before use.** Detect incompatible models, fields, handlers, renderers,
  and action payloads during instance construction instead of relying on downcast panics at interaction time.

## HTTP and Client Contracts

- [ ] **P0 — Share one structured 422 response contract.** Generated routes currently emit `error` and optional
  `violations`, while the web client expects `ErrorInfo { errors }`; real validation failures can be reported as network
  errors. Add server/client contract tests. See
  [HTTP and Data Providers](design/http-and-data-providers.md#current-422-incompatibility).
- [ ] **P0 — Define an explicit write-result model.** The server returns `Saved<Model>`, while clients deserialize
  `Saved<UpdateModel>`. Remove the accidental JSON-compatibility requirement by sharing a dedicated response type.
- [ ] **P1 — Generate resource names, paths, and DTO shapes from one contract.** Backend and frontend currently
  declare resource names independently and share the route surface by convention, allowing silent drift.
- [ ] **P1 — Replace dynamic typetag stripping with an explicit wire conversion.** The dynamic provider currently
  requires exactly one outer typetag object entry and mutates the serialized value before sending it.
- [ ] **P1 — Complete client-provider route parity.** Add or deliberately exclude typed delete-one/delete-many and
  dynamic delete-one so the provider API matches the generated server surface.
- [ ] **P1 — Make provider scoping semantics uniform.** Decide whether a base condition is a general client scope and,
  if so, apply it consistently or expose explicitly unscoped operations. Delete-by-ID currently bypasses it. Keep
  authorization server-side regardless.
- [ ] **P2 — Decide whether the POST-only CRUD surface is permanent.** If it is, document infrastructure and caching
  implications; otherwise introduce conventional methods without losing rich request bodies or compatibility.

## Backend Atomicity and Concurrency

- [ ] **P0 — Make mutation outcomes truthful and define after-hook semantics.** A request can return an error after
  storage changed because rejecting after hooks and validation persistence run outside the mutation. Put rejection in a
  transaction, make post-commit hooks non-rejecting, or report committed-with-follow-up-failure explicitly. See
  [Backend Operations](design/backend-operations.md#atomicity-and-failure-boundaries).
- [ ] **P1 — Make validation cleanup after delete reliable.** Best-effort cleanup can leave orphaned validation rows;
  use the entity transaction, a durable retry, or database-level referential cleanup.
- [ ] **P1 — Preserve every batch-delete failure.** Do not ignore `after_delete` failures. Return enough information
  to distinguish repository, hook, validation, cleanup, and notification outcomes and to resume partial work safely.
- [ ] **P1 — Make batch-delete traversal deterministic.** Re-fetching the same unbounded condition without a cursor
  can let one entirely undeletable batch stop processing even when later matching rows could be deleted.
- [ ] **P1 — Add an optimistic-concurrency contract.** Updates currently have no revision, compare-and-swap condition,
  or conflict response, so concurrent edits can overwrite one another.
- [ ] **P2 — Replace the batch-size memory heuristic.** `size_of::<Model>() * 3` does not measure heap-owned data.
  Use a configurable row limit, storage-side batching, or a measured budget.

## Validation and Collaboration

- [ ] **P0 — Implement or remove aggregate validation.** `run_global_validation` manages state but never invokes
  validators, persists results, or broadcasts a full result. Before activation, make full `by_entity` JSON-safe and
  preserve resource-general findings during conversion. Run real work outside request latency, retain the
  one-running/one-pending guarantee, and add composite-ID round-trip tests. See
  [Validation and Collaboration](design/validation-and-collaboration.md#current-aggregate-validation-boundary).
- [ ] **P0 — Normalize validation severity and timing.** Resolve post-create Critical findings after insertion,
  discarded pre-create Major findings, ignored delete Major findings, and the absence of post-update model validation.
  Make `Saved<T>` and HTTP behavior describe exactly what can be returned.
- [ ] **P0 — Propagate unified validation save failures.** `UnifiedValidationRepository::save_all` logs individual
  failures and returns `Ok(())`; callers cannot know that validation state is incomplete.
- [ ] **P1 — Make validation replacement atomic with entity updates.** Update currently writes the entity, deletes old
  findings, and saves new findings as separate operations that can leave missing or stale status.
- [ ] **P1 — Define durable collaboration delivery.** Current broadcasts add request latency but swallow delivery
  failures. Choose an outbox/retry protocol or explicitly decouple best-effort notifications from durable events.
- [ ] **P1 — Add collaboration identity and ordering.** Messages need enough revision, origin, correlation, and
  ordering information for receivers to reject stale state and deduplicate retries.
- [ ] **P1 — Add the frontend collaboration consumer.** Implement refetch/invalidation and validation-state handling,
  including duplicate, delayed, missing, and reordered messages.
- [ ] **P1 — Define clear-state broadcasts consistently.** Specify when empty partial results are emitted, especially
  after creation, so receivers can distinguish “not evaluated” from “known clear.”
- [ ] **P2 — Reconcile the legacy validation persistence surface.** Remove or integrate `CkValidationModel` and the
  per-resource traits; they are not used by `UnifiedValidationRepository`.
- [ ] **P2 — Design scalable validation-ID lookup.** The unified table stores composite IDs as JSON without a normal
  B-tree index. Add a database-appropriate indexing strategy or a queryable normalized key.

## Leptos Instances, Lists, and Navigation

- [ ] **P0 — Invalidate list selection whenever the displayed dataset changes.** Selection currently clears only on
  the reload token, not on page, ordering, base-condition, or data changes. `all_selected` compares lengths rather than
  membership, so another page can appear selected and mass deletion can target stale rows.
- [ ] **P1 — Finish or remove additional table-row actions.** `CrudTable` drops the supplied action list before
  passing it to the body, and the remaining trigger path calls `todo!()`. Unify this path with `CrudEntityAction` or
  delete the unusable legacy abstraction.
- [ ] **P1 — Implement or remove list filtering.** The filter button is disabled and its state never affects a
  request.
- [ ] **P1 — Expose table control policy.** Read, edit, and delete availability are hard-coded to `true`, and
  `CrudBuiltinViewControls` does not govern list create or row actions. UI controls remain presentation, not security.
- [ ] **P1 — Enforce valid pagination state.** Make page numbers nonzero, clamp the current page when count or page
  size shrinks, and test empty and last-page deletion cases.
- [ ] **P1 — Reject duplicate instance names.** `CrudInstanceMgr` silently replaces the previous registration, which
  can redirect nested-resource lookup to the wrong instance.
- [ ] **P1 — Validate and type parent-resource mappings.** `CrudParentConfig` stores field names as strings. A bad
  mapping can merely remove list scoping but panic later while prefilling the create model. Resolve both fields once
  during instance construction and report one configuration error.
- [ ] **P1 — Implement default `OffsetDateTime` and array renderers.** Built-in rendering currently shows placeholder
  text for both kinds.
- [ ] **P2 — Apply layout column metadata.** `Layout` records one to four columns, but `CrudFields` ignores the
  choice.
- [ ] **P2 — Remove or implement the documented per-position field renderer tier.** Current source comments describe a
  resolution level that the renderer does not perform.
- [ ] **P2 — Decide the mounted-configuration mutability contract.** Either persist and restore the mutable subset and
  make intended changes reactive, or tighten the API and comments around mount-time-only configuration.
- [ ] **P2 — Support server-capable read/edit loading where required.** Those views currently use `LocalResource` and
  cannot perform their data load during server rendering.
- [ ] **P2 — Add localization.** Several list, pagination, deletion, and date-time strings or formats are hard-coded,
  including German labels and non-locale-aware date formatting.
- [ ] **P2 — Add an application-defined entity label for destructive confirmation.** The delete modal currently shows
  a debug-formatted ID instead of a useful resource-specific description.

## Actions and Persistence Follow-Ups

- [ ] **P1 — Validate action IDs and lifecycle.** Reject duplicate IDs and requested/executing markers, disable entity
  actions without model state, make completion idempotent, and provide cancellation or timeout when completion is lost.
- [ ] **P1 — Give success and failure distinct aftermath semantics.** The instance currently handles `Ok` and `Err`
  identically. Define failure-specific defaults and preserve structured error information.
- [ ] **P1 — Integrate actions with dirty navigation when they replace a surface.** Provide a guarded helper or
  require an explicit navigation outcome so actions cannot accidentally discard drafts.
- [ ] **P1 — Resolve incomplete action states and outlets.** The entity-action enum includes create, but the built-in
  create view does not render entity actions. Generalized action-placement names still publish only create controls.
- [ ] **P1 — Define save-and-stay draft semantics.** After a successful create with `CrudCreateSaveTarget::Stay`, the
  form and dirty baseline are unchanged, so later navigation can warn about data that was already saved.

## SeaORM, Build, and Maintenance

- [ ] **P1 — Resolve the `sqlx-postgres 0.7.4` future-incompatibility warning.** Inspect Cargo's report and update or
  patch the SeaORM/SQLx dependency chain before a future Rust release turns the warning into a build failure.
- [ ] **P1 — Support explicit SQL NULL updates.** Generated optional update fields currently mean “write when
  `Some`, leave unchanged when `None`” and cannot distinguish set-null from unchanged.
- [ ] **P2 — Separate portable and PostgreSQL-specific read-view support.** Make the database requirement explicit in
  APIs or provide adapter implementations for other supported SeaORM backends.
- [ ] **P2 — Stop requiring unused Leptonic metadata.** The build script requires `js-dir` even though CrudKit uses
  only `style-dir`.
- [ ] **P2 — Consolidate error infrastructure and dead compatibility types.** Evaluate the `thiserror`/SNAFU split and
  remove obsolete `NoData` variants, aliases, and empty components after compatibility review.
- [ ] **P2 — Reconcile stale source documentation with the design set.** Several Rust comments still describe removed
  model relationships, aggregate validation as implemented, nonexistent renderer tiers, or two active validation
  storage modes. Keep [the design index](design/crudkit.md) authoritative and link to it instead of copying architecture
  prose.
- [ ] **P2 — Decide whether HTTP integrations remain inside runtime crates.** Axum lives in `crudkit-rs` and Reqwest
  in `crudkit-web`; extract adapters only if the supported integration boundary requires independent versioning.
