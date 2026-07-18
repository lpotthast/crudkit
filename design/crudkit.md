# CrudKit Design

These documents explain the decisions and invariants behind CrudKit's current implementation. They are design records,
not an API reference or a roadmap. Exact signatures remain in the source; behavior that other crates or applications
rely on belongs here. Work intended to change those decisions belongs in [the backlog](../TODO.md).

CrudKit supplies reusable CRUD mechanics across Rust models, storage, HTTP, and Leptos. It deliberately does not own an
application's workflow, authorization rules, route structure outside the generated CRUD API, or domain language.

## Document Map

- [Architecture](architecture.md) defines crate families, dependency direction, compile-time and runtime boundaries, and
  ownership of application policy.
- [Data Contracts](data-contracts.md) defines model roles, fields, values, identifiers, conditions, layout descriptions,
  and frontend type erasure.
- [Derive Macros](derive-macros.md) defines generation ownership, shared code-generation rules, and the boundary between
  generated implementations and runtime contracts.
- [Backend Operations](backend-operations.md) defines `CrudResource`, repositories, authentication policy, lifecycle
  hooks, operation ordering, and atomicity boundaries.
- [HTTP and Data Providers](http-and-data-providers.md) defines generated Axum routes, request and response shapes,
  typed and dynamic clients, condition scoping, and error translation.
- [Validation and Collaboration](validation-and-collaboration.md) defines validators, severities, partial and full
  results, persistence semantics, change notifications, and the current aggregate-validation boundary.
- [SeaORM Adapter](sea-orm-adapter.md) defines how the storage-neutral backend maps to SeaORM entities, columns,
  queries, read views, and validation storage.
- [Instances and Composition](instances-and-composition.md) defines mounted `CrudInstance` state, reactive handles,
  nested resources, field rendering, context isolation, and reset behavior.
- [Views, Navigation Scopes, and Dirty Guards](views-and-navigation.md) defines open view descriptions, renderer
  registration, accepted-view ownership, navigation attempts, return actions, dirty guards, and leave confirmation.
- [Actions](actions.md) defines resource and entity actions, payload UI, execution state, completion, and
  application-placed create-action outlets.
- [Theming and Generated Assets](theming-and-generated-assets.md) defines the SCSS source, build-time generation,
  disposable output boundary, and application-owned overrides.

Each decision has one owning document. Other documents link to it instead of restating it.

## Cross-Cutting Principles

- Shared contracts are serializable where they cross a process or crate boundary.
- Backend operations depend on storage abstractions; storage adapters translate them to a concrete ORM.
- Built-in frontend behavior is a default that applications can compose or replace through explicit extension points.
- Runtime type erasure is introduced only where a non-generic host must render differently typed resources.
- Recoverable runtime configuration failures should be visible in the UI and diagnostics. Violations of internal
  typed-configuration invariants may still fail fast.
- Application code owns fine-grained authorization and domain workflow policy.

## Maintaining the Design

A change to serialization, operation ordering, storage boundaries, rendering composition, reactive ownership,
navigation, or error behavior must update the owning document in the same change. A current limitation is labeled as
such; it must not silently become a promised invariant.

- Keep every line at or below 120 characters, including tables, links, and code examples.

Tests should cover a decision at the lowest useful layer. Use integration or browser coverage when the contract depends
on HTTP mapping, DOM ownership, cleanup, or user interaction.
