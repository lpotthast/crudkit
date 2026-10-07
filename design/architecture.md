# Architecture

CrudKit is a workspace of layered libraries and their code-generation companions. Applications can use the shared and
backend crates without the Leptos frontend. They can also use the web and Leptos crates with a non-SeaORM backend that
implements the same HTTP contract.

## Dependency Direction

The important runtime dependency direction is:

```text
                     crudkit-wire-format
                              |
                         crudkit-core
                         /          \
                crudkit-rs       crudkit-web
                    ^                 ^
                    |                 |
            crudkit-sea-orm     crudkit-leptos
                                      |
                           crudkit-leptos-theme
                           (build dependency)
```

Arrows point from an integration toward the contract it implements or consumes. Backend and frontend do not depend on
each other; they meet through the versioned wire format and the generated HTTP shape. `crudkit-rs` and `crudkit-web`
also depend on `crudkit-wire-format` directly, because they convert at their transport boundaries.

## Crate Families

- **Wire format — `crudkit-wire-format`:** The versioned serialized contract between clients and servers. Each version
  is a module (`v1`) of data transfer objects with a matching suffix (`ReadManyV1`). It has no CrudKit dependencies;
  a breaking change adds a new version instead of changing an existing one. `v1` covers every request, response,
  and error body, and the collaboration messages.
- **Shared — `crudkit-core`:** Framework-neutral models, values, identifiers, conditions, validation results,
  collaboration messages, internal CRUD requests, and save/delete result types, plus the conversions to and from each
  wire-format version (`wire_v1`). Its conditions, IDs, requests, results, validation, and collaboration types are
  not serialized themselves; whatever leaves the process uses a wire-format type instead.
- **Backend — `crudkit-rs`:** Storage-neutral resources, repositories, CRUD operations, lifecycle hooks, validation
  orchestration, authentication policy, collaboration service abstraction, and the current Axum route generator.
- **Web client — `crudkit-web`:** Frontend resource and field contracts, typed and type-erased models, serializable
  layouts and views, HTTP request execution, typed or dynamic REST data providers, and UI-independent list,
  deletion, navigation, and value-formatting logic. It has no Leptos component dependency.
- **UI — `crudkit-leptos`:** Instance configuration (`config`), the mounted-instance runtime with navigation scopes
  and dirty guards (`instance`), state and interaction hooks (`hooks`), headless atoms (`atoms`), and the built-in UI
  (`components`), using Leptos and the hooks and atoms of Leptonic's hooks branch.
  See [Leptos Hooks, Atoms, and Components](leptos-hooks-atoms-components.md).
- **Styling — `crudkit-leptos-theme`:** Embedded CrudKit SCSS sources and destructive generation into a disposable
  output directory.
- **Storage adapter — `crudkit-sea-orm`:** `Repository` implementation, field-to-column mappings, query translation,
  read-view helpers, validation persistence, and CrudKit-owned migrations for SeaORM.
- **Derives — `crudkit-core-macros`, `crudkit-rs-macros`, `crudkit-web-macros`, and
  `crudkit-sea-orm-macros`:** Generate implementations for the contracts owned by their corresponding runtime layer.
- **Macro support — `crudkit-core-macro-util` and `crudkit-rs-macros-core`:** Share parsing and token-generation logic
  without adding it to runtime crates.

Axum currently lives in `crudkit-rs`, and Reqwest-based execution lives in `crudkit-web`. They are integration surfaces
inside those crates, not separate adapter crates.

## Typed Core, Erased UI Host

Backend operations remain generic over `R: CrudResource`. Storage adapters likewise use associated types and trait
bounds, so model mismatches are compile-time errors.

The Leptos instance is intentionally non-generic after configuration. Concrete frontend model and field types are
converted to the `Erased*` traits and `Dyn*` wrappers described in [Data Contracts](data-contracts.md).
`crudkit-web`'s `ModelHandler` captures the resource-specific conversions and deserializers before `CrudInstance`
mounts; `crudkit-leptos` holds no resource-specific code of its own.

This boundary lets one component host render arbitrary resources. It does not make configuration dynamically safe: a
downcast mismatch means the application assembled incompatible resource, model, field, or handler values and may panic.

## Code Generation Boundary

Derive crates reduce repetitive implementations; they do not define a second architecture. The generated implementation
must satisfy the same public trait that a handwritten implementation would satisfy.

Adapter-specific generation stays adapter-specific. For example, `CkId` belongs to the shared layer, frontend `CkField`
implements frontend value access, backend `CkField` implements query metadata, and SeaORM derives implement ActiveModel
or column bridges. This keeps ORM types out of `crudkit-rs` and UI requirements out of `crudkit-core`.

## Application Policy Boundary

CrudKit owns generic mechanics: selecting entities, invoking hooks, rendering fields, coordinating views, and reporting
validation or transport outcomes. An application owns:

- resource names and domain labels;
- role, claim, tenant, and ownership checks;
- workflow transitions beyond generic CRUD;
- application routes and URL semantics;
- which custom views, actions, and field renderers exist;
- database-specific constraints and transactions beyond an adapter's documented behavior.

Reusable primitives may support those choices, but CrudKit must not special-case an application's resource or workflow
meaning.

## Runtime Ownership

Leptos signals, stored values, contexts, and navigation objects are shared handles. The detailed ownership and cleanup
contracts belong to [Instances and Composition](instances-and-composition.md) and
[Views, Navigation Scopes, and Dirty Guards](views-and-navigation.md).
