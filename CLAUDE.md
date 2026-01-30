# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Crudkit is a type-safe, full-stack Rust CRUD framework featuring backend integration with SeaORM + Axum and frontend UI
components built with Leptos 0.8. The project is organized as a single workspace with 12 crates.

## Design Philosophy

The framework is designed to be **well-defined**, **extensible**, **easy to reason about**, and **safe to use**.

### Core Principles

1. **Exhaustiveness over stringly-typed APIs**: Use enums and pattern matching to force compile-time handling of all
   cases.
2. **Optionality at the metadata level**: Field presence is tracked via metadata (`is_optional()`), not via separate
   types. All optional fields use `Value::Null` for absence.
3. **Type erasure with downcasting escape hatches**: Generic code uses typed traits; runtime polymorphism uses `Erased*`
   traits with `Dyn*` wrappers that support downcasting.
4. **Fine-grained reactivity**: Frontend signals operate at the field level (`Signal<Option<T>>`), not entity level.
5. **Explicit error handling**: No implicit panics. Return `Result` with semantic error types. HTTP 200 = success;
   everything else is an error.
6. **Composition over inheritance**: Extend via trait implementations and lifecycle hooks, not subclassing.

### Safety Patterns

- `#![forbid(unsafe_code)]` in all crates.
- `#![deny(clippy::unwrap_used)]` prevents implicit panics.
- Graduated accessor methods: `as_*()` → `Option<T>` (safe), `expect_*()` → `T` (panics on mismatch, documented),
  `take_*()` → `Option<T>` (consuming).
- Validation returns `Result<Saved<T>, CrudError>` where `Saved` contains warnings; `CrudError` contains critical
  errors.

## Common Commands

This project uses `just` as a command runner. Run `just` to see all available commands.

### Building and Checking

```bash
# Check all crates for compilation errors
just check

# Run tests across all crates
just test

# Run clippy with strict linting
just clippy
```

### Formatting

```bash
# Format Rust code in all crates
just fmt

# Format Leptos components (requires leptosfmt)
just leptosfmt
```

### Single Crate Operations

To work with a single crate, use the `-p` flag:

```bash
# Check a single crate
cargo check -p crudkit-rs

# Run tests for a single crate
cargo test -p crudkit-leptos

# Run a specific test
cargo test -p crudkit-core test_name
```

### Dependency Management

```bash
# Update dependencies to latest compatible versions
just update

# Check for available upgrades (including breaking changes)
just upgrades

# Automatically upgrade all dependencies to latest versions
just upgrade

# Sort dependencies in all Cargo.toml files
just sort
```

## VCS

Automatically add newly created files to version control.

## Architecture

### Crate Organization

The project is organized as a single workspace with 12 crates following a 3-layer architecture:

```
crudkit/
├── Cargo.toml               # Root workspace
├── Cargo.lock               # Single lockfile
├── crudkit-core/            # Shared types (Value, Order, Saved, Deleted)
├── crudkit-core-macros/     # CkId derive macro (proc-macro crate)
├── crudkit-core-macro-util/ # Shared utilities for derive macros
├── crudkit-rs/              # Core backend framework
├── crudkit-rs-macros/       # Backend derive macros
├── crudkit-rs-macros-core/  # Shared macro utilities
├── crudkit-sea-orm/         # SeaORM repository implementation
├── crudkit-sea-orm-macros/  # SeaORM-specific derive macros
├── crudkit-web/             # Platform-agnostic web abstractions
├── crudkit-web-macros/      # Web layer derive macros
├── crudkit-leptos/          # Leptos UI components
└── crudkit-leptos-theme/    # CSS theme generation
```

### Layer Overview

1. **Shared Layer** - Core types used by both frontend and backend:
    - `crudkit-core` - Shared types (Value enum, Order, Saved, Deleted, etc.)
    - `crudkit-core-macros` - CkId derive macro for type-safe entity identifiers
    - `crudkit-core-macro-util` - Shared utilities for derive macros (ValueKind, string helpers)

2. **Backend Layer**:
    - `crudkit-rs` - Storage-agnostic CRUD framework
    - `crudkit-rs-macros` - Backend derive macros
    - `crudkit-rs-macros-core` - Shared macro utilities
    - `crudkit-sea-orm` - SeaORM repository implementation
    - `crudkit-sea-orm-macros` - SeaORM-specific derive macros
    - Derive macros: `CkSeaOrmCreateModel`, `CkSeaOrmUpdateModel`, `CkField`, `CkValidationModel`, `CkResourceContext`
    - Axum REST API generation via `impl_add_crud_routes!` macro
    - Lifecycle hooks (before/after create/update/delete)
    - Keycloak authentication support

3. **Frontend Layer**:
    - `crudkit-web` - Platform-agnostic web abstractions (no Leptos dependency)
        - HTTP client via `CrudRestDataProvider`
        - Derive macros: `CkResource`, `CkField`, `CkActionPayload`
    - `crudkit-web-macros` - Web layer derive macros
    - `crudkit-leptos` - Leptos 0.8 UI components
        - Views: `CrudListView`, `CrudCreateView`, `CrudEditView`, `CrudReadView`, `CrudDeleteModal`
        - Fine-grained reactivity via `ReactiveValue` (field-level signals)
    - `crudkit-leptos-theme` - CSS theme generation

### The CrudResource Pattern

The central abstraction is the `CrudResource` trait (crudkit-rs/src/resource.rs), which defines:

- Entity types (database models vs read-only views)
- Create/Update DTOs
- Repository implementation
- Validators and validation result storage
- WebSocket controller for real-time updates
- Custom context and lifecycle hooks

### Key Traits and Types

**Shared Layer** (crudkit-core):

- `Model` (crudkit-core) - Base trait for all data models with typed field access
- `Named` (crudkit-core) - Trait for types that have a `name()` method

**Backend** (crudkit-rs):

- `CrudResource` - Central trait defining all types for a CRUD resource
- `Model` - Backend model trait (extends crudkit-core::Model with `Field: Field` bound)
- `Field` - Trait for field enums providing typed access to model fields
- `Repository<R>` - Data access abstraction with fetch/insert/update/delete operations
- `CrudContext<R>` - Context providing access to:
    - `res_context: Arc<R::Context>` - Resource-specific context (custom state)
    - `repository: Arc<R::Repository>`
    - `validators: Vec<Arc<dyn EntityValidator<R>>>`
    - `aggregate_validators: Vec<Arc<dyn AggregateValidator<R>>>`
    - `validation_result_repository: Arc<R::ValidationResultRepository>`
    - `ws_controller: Arc<R::WebsocketService>`
    - `global_validation_state: Arc<GlobalValidationState>`
- `CrudLifetime<R>` - Lifecycle hooks (before_create, after_create, etc.)

**Web Layer** (crudkit-web):

- `Resource` - Central frontend trait defining CreateModel/ReadModel/UpdateModel
- `Model` - Frontend model trait with field enumeration and serialization
- `FieldAccess<T>` - Trait for typed field value access with `value()`/`set_value()`
- `CrudRestDataProvider<T>` - HTTP client for CRUD operations

**Type-Erased Traits** (crudkit-web/model.rs) - Use `Erased*` for traits, `Dyn*` for wrapper types:

- `ErasedModel`, `ErasedCreateModel`, `ErasedReadModel`, `ErasedUpdateModel` - Type-erased model traits
- `ErasedField`, `ErasedCreateField`, `ErasedReadField`, `ErasedUpdateField` - Type-erased field traits
- `ErasedIdentifiable` - Type-erased trait for models with `id() -> SerializableId`
- `DynModel`, `DynCreateModel`, `DynReadModel`, `DynUpdateModel` - Boxed type-erased model wrappers
- `DynCreateField`, `DynReadField`, `DynUpdateField` - Arc-wrapped type-erased field wrappers
- `TypeErasedField` - Marker trait for `Dyn*Field` wrapper types

**Frontend** (crudkit-leptos):

- `ReactiveValue` - Fine-grained reactive wrapper around Value (each field has RwSignal)
- `SignalsTrait` - Convert between models and reactive field maps
- `CrudInstanceContext<T>` - Manages view state (List/Create/Edit/Read/Delete)

### The Value System

The `Value` enum is the foundation for type-safe field access:

```rust
pub enum Value {
    Null,                    // Explicit absence (optional fields)
    Void(()),                // Field doesn't participate in Value system
    Bool(bool),
    I8(i8), ...,             // Primitives
    String(String),          // Common types
    Uuid(Uuid),              // Ecosystem integration
    Array(Vec<Value>),       // Collections
    Other(Box<dyn FieldValue>), // Extension point
}
```

**Key design decisions:**

1. **No separate optional variants** (e.g., no `OptionalString`). Optionality is tracked via field metadata.
2. **Uniform `Value::Null`** for all absent optional values enables consistent handling.
3. **`Null` vs `Void`**: `Null` means "no value present" for an optional field. `Void` is the Value representation of
   Rust's unit type `()`.
4. **`Other` variant** allows custom types via the `FieldValue` trait.

### Three-Tier Type Erasure

The framework uses a consistent pattern for runtime polymorphism:

| Tier | Pattern          | Purpose                         | Example                      |
|------|------------------|---------------------------------|------------------------------|
| 1    | Typed traits     | Compile-time generic code       | `Model`, `FieldAccess<T>`    |
| 2    | `Erased*` traits | Object-safe trait objects       | `ErasedModel`, `ErasedField` |
| 3    | `Dyn*` wrappers  | Boxed/Arc wrappers with helpers | `DynModel`, `DynField`       |

**Conversion flow:** `T: Model` → auto-impl `ErasedModel` → wrap in `DynModel` → downcast back when needed.

Wrappers provide `downcast_ref<T>()` and `downcast_mut<T>()` for escaping back to concrete types.

### Field-Level Reactivity (Frontend)

Rather than entity-level signals, the framework uses field-level granularity:

```rust
pub enum ReactiveValue {
    Bool(RwSignal<Option<bool>>),
    String(RwSignal<Option<String>>),
    // ... one variant per Value variant
}
```

**Key insight:** All field signals use `Signal<Option<T>>`:

- Required fields: `Some(value)`
- Optional fields: `None` when absent
- This enables uniform components that work for both required and optional fields.

Field renderers accept `Signal<Option<T>>` and derive display from `FieldMode` (Display/Readable/Editable).

### Error Handling Architecture

The framework follows a unified error handling approach: **HTTP 200 always means success with entity. Everything else is
an error.**

**Backend Error Types** (crudkit-rs):

`HookError<E>` - Returned by lifecycle hooks to reject operations:

- `Forbidden { reason }` - Permission denied (maps to HTTP 403)
- `UnprocessableEntity { reason }` - Business logic rejection (maps to HTTP 422)
- `Internal(E)` - Technical/infrastructure error (maps to HTTP 500)

`CrudError` - Unified error type for all CRUD operations:

- `Forbidden` / `UnprocessableEntity` - From lifecycle hooks
- `ValidationFailed` - Critical validation errors (HTTP 422)
- `NotFound` - Entity not found (HTTP 404)
- `Repository` / `LifecycleError` - Infrastructure errors (HTTP 500)

**Frontend Error Type** (crudkit-web):

`CrudOperationError` - Type-safe error for frontend handlers:

- `Forbidden { reason }` - Permission denied
- `UnprocessableEntity { reason }` - Business logic rejection
- `NotFound { message }` - Entity not found
- `ServerError { message }` - Server error
- `Unauthorized { message }` - Authentication required
- `NetworkError { message }` - Client-side error

**Return Types**:

- Create/Update operations return `Result<Saved<T>, CrudError>` where `Saved<T>` contains the entity and full validation
  violations (non-critical warnings)
- Delete operations return `Result<Deleted, CrudError>` where `Deleted` contains the count of affected entities

### Naming Conventions

The framework follows consistent naming patterns:

**Trait Naming:**

- **No `*Trait` suffix** for main abstractions: Use `Field`, `Model`, `HasId` - not `FieldTrait`, `ModelTrait`
- **No `get_*` prefix** for simple accessors: Use `name()`, `id()`, `value()` - not `get_name()`, `get_id()`
- **`Erased*` prefix** for type-erased traits: `ErasedModel`, `ErasedField`, `ErasedIdentifiable`
- **`Dyn*` prefix** for type-erased wrapper types: `DynReadModel = Box<dyn ErasedReadModel>`,
  `DynCreateField = Arc<dyn ErasedCreateField>`

**Value Accessor Patterns:**

| Pattern      | Returns      | Behavior                              | Use case                |
|--------------|--------------|---------------------------------------|-------------------------|
| `as_*()`     | `Option<&T>` | Safe, returns `None` on type mismatch | When type might vary    |
| `expect_*()` | `T`          | Panics on type mismatch               | When type is guaranteed |
| `take_*()`   | `Option<T>`  | Consumes, returns `None` on mismatch  | Ownership transfer      |

**Derive Macro Prefix:**

- All derive macros use the `Ck` prefix (short for CrudKit): `CkId`, `CkField`, `CkResource`, etc.
- SeaORM-specific macros use `CkSeaOrm` prefix: `CkSeaOrmCreateModel`, `CkSeaOrmUpdateModel`

### Derive Macro Ecosystem

The framework provides derive macros to reduce boilerplate:

**Shared** (`crudkit-core-macros`): `CkId`

**Backend** (`crudkit-rs`): `CkField`, `CkValidationModel`, `CkResourceContext`

**SeaORM** (`crudkit-sea-orm`): `CkSeaOrmCreateModel`, `CkSeaOrmUpdateModel`

**Web Layer** (`crudkit-web`): `CkResource`, `CkField`, `CkActionPayload`

### Composite Primary Keys

Unlike most CRUD frameworks, Crudkit fully supports composite primary keys via the `Id` trait. An entity's
ID can be a tuple of multiple fields (e.g., `(user_id, org_id)`).

### Query DSL (Conditions)

The `Condition` type provides composable, type-safe query building:

```rust
pub enum Condition {
    All(Vec<ConditionElement>),  // AND logic
    Any(Vec<ConditionElement>),  // OR logic
}

pub struct ConditionClause {
    pub column_name: String,
    pub operator: Operator,       // Equal, NotEqual, Less, LessOrEqual, Greater, GreaterOrEqual, IsIn
    pub value: ConditionClauseValue,
}
```

**Key features:**

- Nested conditions for complex queries.
- `ConditionClauseValue` is a separate enum from `Value` (excludes non-comparable types).
- `Condition::none()` creates a matchless condition for bulk operations.
- `merge_conditions()` safely combines filters with AND logic.

### Framework Integrations

- **SeaORM** 0.12.15 - Primary ORM (repository pattern allows alternatives)
- **Leptos** 0.8.15 - Frontend reactive framework with fine-grained reactivity
- **Leptonic** 0.5.0 - Leptos component library
- **Axum** 0.8.8 - HTTP server with middleware support
- **Keycloak** - Authentication (`axum-keycloak-auth` backend, `leptos-keycloak-auth` frontend)
- **utoipa** 5.4.0 - OpenAPI documentation generation

## Important File Locations

### Core Trait Definitions

- `crudkit-core/src/lib.rs` - Base `Model` trait, `Named` trait, `Value` enum, `Order`, shared types
- `crudkit-rs/src/resource.rs` - `CrudResource` trait
- `crudkit-rs/src/data.rs` - Backend `Model` trait, `Field` trait
- `crudkit-rs/src/repository.rs` - `Repository` trait
- `crudkit-web/src/lib.rs` - `Resource`, `Model`, `FieldAccess` traits
- `crudkit-web/src/model.rs` - Type-erased `Dyn*` traits and `Any*` wrappers

### CRUD Operations

- `crudkit-rs/src/create.rs` - Create operations with validation
- `crudkit-rs/src/read.rs` - Read operations with filtering
- `crudkit-rs/src/update.rs` - Update operations with lifecycle hooks
- `crudkit-rs/src/delete.rs` - Delete operations with lifecycle hooks

### Derive Macro Implementations

- `crudkit-core-macros/src/lib.rs` - CkId derive macro
- `crudkit-core-macro-util/src/lib.rs` - Shared derive macro utilities
- `crudkit-rs-macros/src/lib.rs` - Backend derive macros
- `crudkit-sea-orm-macros/src/lib.rs` - SeaORM derive macros
- `crudkit-web-macros/src/lib.rs` - Web layer derive macros

## Code Style

### Comments

- Don't state the obvious.
- Use comments to make implicit behavior explicit.
- Always end comments with a period.

### Refactoring

- Delete unused code. Don't keep old types, functions, or aliases for backwards compatibility unless explicitly asked.
- When consolidating duplicate code, remove the old implementations entirely.

## Working with the Codebase

### When Adding Shared Types

- Put types used by both frontend and backend in `crudkit-core/`

### When Modifying Backend Logic

- Core framework changes go in `crudkit-rs/`
- SeaORM-specific code goes in `crudkit-sea-orm/`
- Backend derive macros are in `crudkit-rs-macros/`

### When Modifying Web Layer

- Platform-agnostic abstractions go in `crudkit-web/`
- Web derive macros are in `crudkit-web-macros/`

### When Modifying Frontend

- Leptos components go in `crudkit-leptos/`
- CSS/theming goes in `crudkit-leptos-theme/`

### Understanding the Data Flow

1. **Create Flow**:
    - Frontend: User fills form -> CreateModel
    - HTTP POST to backend REST endpoint
    - Backend: `before_create` hook -> Repository.insert() -> Validation -> `after_create` hook -> WebSocket broadcast
    - Success: HTTP 200 with `Saved<T>` | Failure: HTTP 4xx/5xx with error
    - Frontend receives `Result<Saved<T>, CrudOperationError>`, calls `on_entity_created` or `on_entity_creation_failed`

2. **Read Flow**:
    - Frontend: CrudInstanceContext.load() -> CrudRestDataProvider.read_many()
    - HTTP POST with filters/pagination (uses POST for complex query bodies)
    - Backend: Repository.read_many() from ReadViewEntity (may be SQL view)
    - Returns paginated results or error

3. **Update Flow**:
    - Frontend: User edits -> UpdateModel with field-level reactive signals
    - HTTP POST to backend
    - Backend: `before_update` hook -> Repository.update() -> Validation -> `after_update` hook -> WebSocket broadcast
    - Success: HTTP 200 with `Saved<T>` | Failure: HTTP 4xx/5xx with error
    - Frontend receives `Result<Saved<T>, CrudOperationError>`, calls `on_entity_updated` or `on_entity_update_failed`

4. **Delete Flow**:
    - Frontend: Delete button -> Confirmation modal
    - HTTP POST to backend (delete-by-id endpoint)
    - Backend: `before_delete` hook -> Repository.delete() -> `after_delete` hook -> WebSocket broadcast
    - Success: HTTP 200 with `Deleted` | Failure: HTTP 4xx/5xx with error
    - Frontend shows toast based on `Result<Deleted, CrudOperationError>`

### Lifecycle Hooks

The `CrudLifetime<R>` trait provides hooks that run before/after each CRUD operation. Hooks can:

- Modify data before persistence (e.g., set timestamps, normalize values)
- Reject operations by returning `HookError::Forbidden` (permission check) or `HookError::UnprocessableEntity` (business
  rule)
- Perform side effects after successful operations (e.g., send notifications)

Example lifecycle hook:

```rust
async fn before_delete(
    model: &Article,
    context: &MyContext,
    request: RequestContext<Auth>,
    data: HookData,
) -> Result<HookData, HookError<MyError>> {
    // Permission check -> HTTP 403
    if model.creator_id != request.auth.user_id && !request.auth.is_admin {
        return Err(HookError::Forbidden {
            reason: "Only the creator or admin can delete".into()
        });
    }
    // Business rule check -> HTTP 422
    if model.has_active_orders() {
        return Err(HookError::UnprocessableEntity {
            reason: "Cannot delete article with active orders".into()
        });
    }
    Ok(data)
}
```

### Validation System

The validation system supports:

- Multiple validators per entity via `Vec<Arc<dyn EntityValidator<R>>>` in `CrudContext`
- Aggregate validators via `Vec<Arc<dyn AggregateValidator<R>>>` for system-wide consistency checks
- Two severity levels: Major (warnings, allow save) vs Critical (blocks save, returns HTTP 422)
- Delta validation via `validate_updated(old, new, trigger)` to compare entity states during updates
- Per-field violations with custom messages
- Validation result storage in database
- Real-time validation updates via WebSocket
- Async global validation with debounce-like behavior (if already running, schedule a re-run; if already scheduled, do
  nothing)

**Key Types**:

- `EntityValidator<R>` - Validates individual entities with `name()`, `version()`, `validate_single()`,
  `validate_updated()`
- `AggregateValidator<R>` - Validates aggregate-level constraints with `validate_all()`
- `GlobalValidationState` - Atomic state for debouncing global validation runs
- `Saved<T>` - Contains entity and `SerializableAggregateViolations` (full violation details, not just a boolean)

### Parent-Child Resources

The framework supports hierarchical relationships where child resources are filtered by parent ID. The
`CrudInstanceContext` tracks parent links and automatically includes parent filters in queries.

### Extension Points

The framework provides well-defined extension points:

| Extension              | Mechanism                                         | Location              |
|------------------------|---------------------------------------------------|-----------------------|
| Custom field types     | Implement `FieldValue` trait, use `Value::Other`  | `crudkit-core`        |
| Custom storage         | Implement `Repository<R>` trait                   | `crudkit-rs`          |
| Field validators       | Add to `CrudContext::validators`                  | `crudkit-rs`          |
| Aggregate validators   | Add to `CrudContext::aggregate_validators`        | `crudkit-rs`          |
| Lifecycle hooks        | Implement `CrudLifetime<R>`                       | `crudkit-rs`          |
| Custom field renderers | Create `FieldRenderer::new()`                     | `crudkit-leptos`      |
| Custom ID types        | Derive `CkId` on any `Eq + Hash + Serialize` type | `crudkit-core-macros` |

**Adding a new field type:**

1. Define the type and implement `FieldValue` (serde, debug, clone, eq, hash).
2. Store instances as `Value::Other(Box::new(my_value))`.
3. For frontend, create a component accepting `Signal<Option<MyType>>`.
4. Register a `FieldRenderer` for the new type.

**Adding a validator:**

```rust
struct MyValidator;

impl EntityValidator<MyResource> for MyValidator {
    fn name(&self) -> &'static str { "my-validator" }
    fn version(&self) -> u32 { 1 }

    async fn validate_single(&self, entity: &Entity) -> Vec<Violation> {
        // Return Major violations (warnings) or Critical violations (block save)
    }
}
```

## Additional Documentation

The `documentation/` folder contains detailed architectural documentation:

- `four-model-architecture.md` - Explains the four-model pattern (Model, CreateModel, UpdateModel, ReadModel) and why
  each serves a distinct purpose
- `validation-architecture.md` - Details the validation system including EntityValidators, AggregateValidators,
  validation modes, and severity levels
