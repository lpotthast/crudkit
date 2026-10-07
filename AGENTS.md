# Repository Guidelines

CrudKit is a reusable CRUD framework.

## Design Source of Truth

Start with [`design/crudkit.md`](design/crudkit.md). It indexes the design records that own CrudKit's current
architectural and behavioral decisions:

- `design/architecture.md`, `design/data-contracts.md`, and `design/derive-macros.md` cover layering, shared contracts,
  type erasure, and code generation.
- `design/backend-operations.md`, `design/http-and-data-providers.md`, `design/sea-orm-adapter.md`, and
  `design/validation-and-collaboration.md` cover storage-neutral operations and integration boundaries.
- `design/instances-and-composition.md`, `design/views-and-navigation.md`, `design/actions.md`, and
  `design/theming-and-generated-assets.md` cover frontend composition and interaction contracts.

Read the owning document before changing a contract. Changes to serialization, operation ordering, storage,
rendering composition, reactive ownership, navigation, validation, or error behavior must update the owning design
document in the same change. Keep future work and known improvement opportunities in `TODO.md`; do not describe a
current limitation as a guaranteed long-term invariant.

The design documents explain intent and ownership. Source code remains authoritative for exact signatures and
low-level implementation details.

## Workspace Boundaries

This is a Rust 2024 Cargo workspace with 13 crates:

- `crudkit-wire-format` contains the versioned wire format (`v1` with `*V1` types) and nothing else.
- `crudkit-core` contains contracts shared across backend and frontend and the conversions to and from the wire format;
  its macro support lives in `crudkit-core-macro-util` and `crudkit-core-macros`.
- `crudkit-rs` owns storage-neutral backend CRUD behavior; backend derives live in `crudkit-rs-macros` and
  `crudkit-rs-macros-core`.
- `crudkit-sea-orm` adapts backend contracts to SeaORM; SeaORM-specific derives live in `crudkit-sea-orm-macros`.
- `crudkit-web` owns platform-neutral web models, layouts, type erasure, and data providers; its derives live in
  `crudkit-web-macros`.
- `crudkit-leptos` owns instance configuration (`config`), mounted instance behavior (`instance`), and Leptos hooks,
  atoms, and components. It builds on the hooks and atoms of Leptonic's `hooks` branch (a path dependency on
  `../leptonic/leptonic`) and never on Leptonic's themed components.
- `crudkit-leptos-theme` owns CrudKit's SCSS sources and generated-asset inputs.

Keep dependencies pointed from concrete integrations toward abstract contracts. In particular:

- Put serialized client-server contracts in a version module of `crudkit-wire-format`, and shared internal contracts
  plus their wire conversions in `crudkit-core`. Never change a released wire version in place; add a new one.
- Keep storage-neutral behavior in `crudkit-rs` and ORM-specific behavior in `crudkit-sea-orm`.
- Keep platform-neutral frontend behavior in `crudkit-web` and Leptos-specific behavior in `crudkit-leptos`. Within
  `crudkit-leptos`, put behavior in hooks and keep built-in components free of behavior that applications could not
  reach through those hooks.
- Keep runtime traits authoritative; derive macros generate implementations of those traits rather than inventing a
  parallel contract.
- Change SCSS in `crudkit-leptos-theme`, not in generated copies inside consuming applications.

## Architectural Invariants

- Prefer exhaustive typed APIs. Use enums and pattern matching instead of stringly typed branching where the set of
  cases is known.
- Optionality belongs to field metadata. `Value::Null` represents an absent optional value; `Value::Void(())`
  represents Rust's unit value.
- Keep generic code typed. Introduce `Erased*` traits and `Dyn*` wrappers only at runtime-polymorphic boundaries, and
  retain deliberate downcasting escape hatches where required.
- Frontend forms use one `ReactiveField` per field. Preserve field-level reactivity and avoid replacing it with one
  entity-wide reactive value.
- Extend behavior through traits, configuration, renderers, lifecycle hooks, and collaboration interfaces rather than
  application-specific conditionals in the framework.
- Application code owns domain workflow and fine-grained authorization policy. CrudKit owns reusable mechanics and
  explicit integration points.
- Honor `#![forbid(unsafe_code)]` and `#![deny(clippy::unwrap_used)]`. Propagate recoverable failures with semantic
  error types; reserve `expect_*` accessors or panics for documented invariant violations.

## Commands and Verification

Run commands from this directory. Use `just` to list available recipes.

```bash
just fmt
just check
just test
```

For focused work, use Cargo's package selection before broadening verification:

```bash
cargo check -p crudkit-leptos
cargo test -p crudkit-core
cargo test -p crudkit-core test_name
```

For a gating lint run, invoke Clippy directly:

```bash
cargo clippy --tests -- -Dclippy::all -Dclippy::pedantic
```

`examples/full-stack` is a separate Cargo workspace: a SQLite-backed, server-rendered CrudKit application. Run it with
`just example` and build it with `just example-check` when changing frontend behavior or the theme.

The current `just clippy` recipe allows a nonzero exit status, so its exit code alone does not prove that linting
passed. Use `just leptosfmt` when Leptos component formatting is relevant.

Start with the narrowest useful check, then run the affected workspace-level command. When fixing a reported failure,
reproduce the original top-level command first and rerun that same command after the fix.

## Code and Test Style

- Use Rustfmt defaults and idiomatic Rust naming.
- Keep comments focused on non-obvious intent or constraints and end prose comments with a period.
- Delete superseded code when refactoring. Do not add compatibility aliases or retain dead implementations unless the
  task explicitly requires backward compatibility.
- Keep every design-document line at or below 120 characters, including tables, links, and examples.
- Put focused unit tests near the code they exercise. Use integration or browser coverage when behavior depends on
  HTTP translation, DOM ownership, cleanup, or user interaction.
- Test extension points and failure paths as well as the happy path, especially across erased-type, lifecycle,
  validation, navigation, and storage-adapter boundaries.

## Git Hygiene

The working tree may contain unrelated user changes. Preserve them and keep edits scoped to the task. Do not stage,
unstage, commit, reset, or otherwise alter repository state unless the user explicitly asks.
