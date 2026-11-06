# Review of `data-sources`

20 commits, about 10k lines over `main`, reviewed on 2026-11-05.

**Verdict:** The direction is sound, the work is careful, and the docs, tests and TODO.md all move with the code. I
didn't find a correctness bug I'd call confirmed. There is one plausible race and a few design weak spots worth deciding
on before this merges.

Clippy on the workspace (tests included) reports no warnings, and `cargo test --workspace` plus the crudkit-leptos
`ssr` markup tests all pass.

## What it does, conceptually

### Things I think are right

- **One data-source trait replaces the dynamic REST provider.** Every operation takes a `CrudScope` plus a request
  type, the request and outcome types are `#[non_exhaustive]`, and the REST routes become one implementation among
  others. This is the right place to put the line. Product APIs with typed routes, nested paths, ETags and 202
  responses now fit without changing CrudKit's HTTP shape.
- **The scope holds parent IDs, not conditions.** Conditions now belong to the REST data source only. That cleanly
  fixes the old leak where "base condition" meant something different on each code path.
- **Versions and merge patches are carried, never interpreted.** Treating `CrudVersion` and `CrudCursor` as opaque
  values CrudKit stores and hands back is the right level of abstraction.
- **Rebasing a draft by JSON merge patch.** On reload, the user's change is turned into a merge patch, applied to the
  new version, and overlapping changes are reported as conflicts. This is a real feature, and it's well thought
  through: unreadable input, a rebase that can't apply, and stale outdated-refusals are all handled.
- **Backend hardening.** An empty or mismatched ID no longer becomes a match-all condition, broadcasts carry an
  `Audience`, and error and log messages no longer include values. All of these are clear improvements.
- **Translator refactor.** Stable IDs, Fluent-compatible names, an English fallback, and an exhaustive enum so a Rust
  translator gets a compile error when a text is added. The macro-generated catalog is a nice touch.

### Things I'd question

1. **The rebase assumes serde names equal field names.** It works on JSON keys and maps them back to fields by name.
   The design doc states the precondition (the update model must survive a serde round trip, with no renames or
   skips), but nothing enforces it. A `#[serde(rename_all = "camelCase")]` would silently break conflict attribution
   and the "field changed" checks. Consider having the `CkField` derive reject serde rename/skip attributes, or adding
   a debug-time round-trip check.
2. **The version only survives because `From<ReadModel> for UpdateModel` copies it.** The doc admits that dropping it
   can turn into "update any version". That's a sharp edge sitting in user code. The derive could generate or verify
   that the version is carried over.
3. **The REST data source ignores what it can't honor.** The trait says a source that can't honor part of a request
   must refuse it. Yet `CrudRestDataSource::update` (`crudkit-web/src/data_source/rest.rs:208`) silently drops
   `version()` if someone puts `#[ck_field(version)]` on a model used with REST. It's documented and the gap is in
   TODO.md, but refusing a versioned request (or at least warning) would follow your own rule.
4. **Conflict messages show raw serde values.** `conflict_message` (`crudkit-leptos/src/hooks/form/state.rs:596`) uses
   `value.to_string()` for anything that isn't a string or a top-level boolean. So dates, enums and numbers appear in
   their wire form, e.g. `"InProgress"` or `2026-11-05T09:30:00Z`, not the way the field displays them. Going through
   the field's `Value` and `format_crud_value` would match what the user typed against.
5. **Messages are still built by joining fragments.** `FormProblem` is `{location}: {message}`, field errors are joined
   with `" "`, and mass-deletion sentences are concatenated. It's documented and acceptable for now, but translators
   can't reorder across those fragments.
6. **Framework-written English is shown as if it were a data-source message.** `"the entity was not deleted"`
   (`crudkit-web/src/data_source/deletion.rs:136`) and `"Critical validation errors prevent the deletion."`
   (`crudkit-web/src/data_source/rest.rs:321`) end up in errors that `CrudTexts::request_error` treats as user-facing
   data-source messages. They are never translated. The single-delete path reaches the user. (The fallback in
   `deletion.rs` is only reached from the one-by-one `delete_many`, where notifications show counts only, so it's
   latent today.) The TODO entry covers `InvalidRequest`, `Request` and `Deserialize`, but not these two.
7. **A completed delete that affected 0 entities counts as an error.** In the one-by-one `delete_many`, an entity
   someone else already deleted shows up as "not deleted". It's arguably a success from the user's point of view.

## Code quality

### Strengths

- Doc comments are thorough and consistent, and every design doc was updated.
- Typed enums are used throughout, and `concerns_only_the_entity` matches exhaustively.
- There are no `unwrap`s, and access to state after disposal goes through `try_*` everywhere.
- Test coverage is unusually good for this kind of logic: merge patch round-trips and conflicts, rebase edge cases,
  stale outdated refusals, cursor generations, ID validation, and that tracing records no data.

### Weaknesses

- **`rebase` is long and dense.** `CrudFormState::rebase` (about 100 lines) mixes serializing, building the patch,
  computing conflicts, failure bookkeeping, and pushing values into reactive fields. Splitting out the conflict merge
  (the `failure.update` closure) into a pure function would make it testable without Leptos and easier to follow.
- **`CrudFormState` keeps growing.** It now holds 14 public signals plus private bookkeeping (targets, error-summary
  counter, owners). Consider grouping problems, conflicts, rebase failure and alerts behind one public "failure" view.
- **Minor duplication.** `path_in_field` is implemented twice, on `CrudFieldProblem` and `CrudFieldConflict`.
  `CrudDeleteRequest` stores both `id` and `entity` even though `id` comes from `entity`.
- **The error summary is inconsistent for invalid input.** It shows the raw `InvalidFields` message without the
  "Invalid input" kind text (`crudkit-leptos/src/atoms/form.rs:585`), while every other error goes through
  `request_error`. If that message is empty, only the list appears, with no lead sentence.
- **Text IDs are written by hand.** The kebab-case ID is a literal next to each variant, guarded only by a test. That's
  fine, though a `paste`-style generator would remove the duplication.

## Plausible bug: a scope change can send a stale cursor

On a list paged by cursor, the cursors are restarted by an `Effect` that watches the scope
(`crudkit-leptos/src/instance/context.rs:392`). Meanwhile the page resource reads `ctx.scope` and the cursor of the
current page directly (`crudkit-leptos/src/hooks/list.rs:91`). When the scope changes (e.g. the route's parent ID
changes while the instance stays mounted), nothing guarantees which runs first. If the resource runs first, it sends
the old scope's cursor for page N with the new scope. The generation check stops it from recording a cursor, but the
request itself is wrong. A strict API will reject the foreign cursor, and the list will flash an error or wrong rows
before reloading page 1.

Ordering and page-size changes don't have this problem because `restart_paging()` runs synchronously before the signal
is set. The scope is the only trigger that goes through an effect. There is no test for a scope change on a list paged
by cursor. One fix is to store the scope a generation belongs to inside `PageCursors` and restart from within the
resource when it differs.

## Recommendation

Address the scope/cursor race and the serde-rename precondition (points 1 and 2) before merging. The rest can go into
TODO.md.
