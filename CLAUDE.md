# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Crudkit is a type-safe, full-stack Rust CRUD framework featuring backend integration with SeaORM + Axum and frontend UI
components built with Leptos 0.8. The project is organized as a multi-workspace repository with 8 independent top-level
crates.

## Common Commands

This project uses `just` as a command runner. Run `just` to see all available commands.

Note: The Justfile currently covers most crates but excludes `crudkit-web` and `crudkit-leptos-theme`. For these, use
cargo commands directly with `--manifest-path`.

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

To work with a single crate, use the `--manifest-path` flag:

```bash
# Check a single crate
cargo check --manifest-path ./crudkit-rs/Cargo.toml

# Run tests for a single crate
cargo test --manifest-path ./crudkit-leptos/Cargo.toml

# Run a specific test
cargo test --manifest-path ./crudkit-id/Cargo.toml test_name
```

### Dependency Management

```bash
# Update dependencies to latest compatible versions
just update

# Check for available upgrades (including breaking changes)
just upgrades

# Automatically upgrade all dependencies to latest versions
just upgrade
```

## VCS

Automatically add newly created files to version control.

## Architecture

### Crate Organization

The project follows a 3-layer architecture:

1. **Shared Layer** - Core types used by both frontend and backend:
    - `crudkit-id` - Type-safe entity identifiers with composite primary key support
    - `crudkit-condition` - Query filtering and condition building
    - `crudkit-core` - Shared types (Value enum, Order, Saved, Deleted, etc.)
    - `crudkit-validation` - Entity validation framework with severity levels
    - `crudkit-collaboration` - Types used for sharing data between different users

2. **Backend Layer** (`crudkit-rs` workspace):
    - `crudkit-rs/crates/crudkit-rs/` - Core backend framework
    - `crudkit-rs/crates/crudkit-sea-orm/` - SeaORM repository implementation
    - Derive macros: `CkCreateModel`, `CkUpdateModel`, `CkColumns`, `CkValidationModel`, `CkResourceContext`
    - Axum REST API generation via `impl_add_crud_routes!` macro
    - Lifecycle hooks (before/after create/update/delete)
    - Keycloak authentication support

3. **Frontend Layer**:
    - `crudkit-web` workspace - Platform-agnostic web abstractions (no Leptos dependency)
        - HTTP client via `CrudRestDataProvider`
        - Derive macros: `CkResource`, `CkField`, `CkActionPayload`
    - `crudkit-leptos` workspace - Leptos 0.8 UI components
        - Views: `CrudListView`, `CrudCreateView`, `CrudEditView`, `CrudReadView`, `CrudDeleteModal`
        - Fine-grained reactivity via `ReactiveValue` (field-level signals)
    - `crudkit-leptos-theme` - CSS theme generation

### The CrudResource Pattern

The central abstraction is the `CrudResource` trait (crudkit-rs/crates/crudkit-rs/src/resource.rs), which defines:

- Entity types (database models vs read-only views)
- Create/Update DTOs
- Repository implementation
- Validators and validation result storage
- WebSocket controller for real-time updates
- Custom context and lifecycle hooks

### Key Traits and Types

**Shared Layer** (crudkit-core, crudkit-id):

- `Model` (crudkit-core) - Base trait for all data models with typed field access
- `Named` (crudkit-core) - Trait for types that have a `name()` method
- `HasId` (crudkit-id) - Trait for models with typed ID access via `id()` method
- `Id` (crudkit-id) - Trait for composite primary key types

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

- `Resource` - Central frontend trait defining CreateModel/ReadModel/UpdateModel (alias: `CrudMainTrait`)
- `Model` - Frontend model trait with field enumeration and serialization (alias: `CrudModel`)
- `FieldAccess<T>` - Trait for typed field value access with `value()`/`set_value()` (alias: `CrudFieldValueTrait`)
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
- **`Dyn*` prefix** for type-erased wrapper types: `DynReadModel = Box<dyn ErasedReadModel>`, `DynCreateField = Arc<dyn ErasedCreateField>`

**Derive Macro Prefix:**

- All derive macros use the `Ck` prefix (short for CrudKit): `CkId`, `CkField`, `CkResource`, etc.
- SeaORM-specific macros may use longer prefixes internally but are aliased for convenience

**Backward Compatibility:**

- Old trait names are available as type aliases (e.g., `CrudModel` = `Model`, `CrudMainTrait` = `Resource`)
- These aliases are deprecated and will be removed in future versions

### Derive Macro Ecosystem

The framework provides derive macros to reduce boilerplate:

**Shared**: `CkId` (crudkit-id)

**Backend** (crudkit-rs): `CkCreateModel`, `CkUpdateModel`, `CkColumns`, `CkValidationModel`, `CkResourceContext`

**Web Layer** (crudkit-web): `CkResource`, `CkField`, `CkActionPayload`

### Composite Primary Keys

Unlike most CRUD frameworks, Crudkit fully supports composite primary keys via the `Id` trait in crudkit-id. An entity's
ID can be a tuple of multiple fields (e.g., `(user_id, org_id)`).

### Framework Integrations

- **SeaORM** 1.2.0 - Primary ORM (repository pattern allows alternatives)
- **Leptos** 0.8.15 - Frontend reactive framework with fine-grained reactivity
- **Leptonic** 0.5.0 - Leptos component library
- **Axum** 0.8.8 - HTTP server with middleware support
- **Keycloak** - Authentication (`axum-keycloak-auth` backend, `leptos-keycloak-auth` frontend)
- **utoipa** 5.4.0 - OpenAPI documentation generation

## Important File Locations

### Core Trait Definitions

- `crudkit-core/crates/crudkit-core/src/lib.rs` - Base `Model` trait, `Named` trait, `Value` enum, `Order`, shared types
- `crudkit-id/crates/crudkit-id/src/lib.rs` - `Id` trait, `HasId` trait
- `crudkit-rs/crates/crudkit-rs/src/resource.rs` - `CrudResource` trait
- `crudkit-rs/crates/crudkit-rs/src/data.rs` - Backend `Model` trait, `Field` trait
- `crudkit-rs/crates/crudkit-rs/src/repository.rs` - `Repository` trait
- `crudkit-web/crates/crudkit-web/src/lib.rs` - `Resource`, `Model`, `FieldAccess` traits
- `crudkit-web/crates/crudkit-web/src/model.rs` - Type-erased `Dyn*` traits and `Any*` wrappers

### CRUD Operations

- `crudkit-rs/crates/crudkit-rs/src/create.rs` - Create operations with validation
- `crudkit-rs/crates/crudkit-rs/src/read.rs` - Read operations with filtering
- `crudkit-rs/crates/crudkit-rs/src/update.rs` - Update operations with lifecycle hooks
- `crudkit-rs/crates/crudkit-rs/src/delete.rs` - Delete operations with lifecycle hooks

### Derive Macro Implementations

- `crudkit-id/crates/derive-crudkit-id/src/lib.rs` - CkId macro
- `crudkit-rs/crates/derive-*/src/lib.rs` - Backend derive macros
- `crudkit-web/crates/derive-*/src/lib.rs` - Web layer derive macros
- `crudkit-core/crates/crudkit-derive-core/src/lib.rs` - Shared derive macro utilities

## Code Style

### Comments

- Don't state the obvious.
- Use comments to make implicit behavior explicit.
- Always end comments with a period.

## Working with the Codebase

### When Adding Shared Types

- Put types used by both frontend and backend in `crudkit-core`
- ID-related types go in `crudkit-id`
- Filter/condition types go in `crudkit-condition`
- Validation types go in `crudkit-validation`

### When Modifying Backend Logic

- Core framework changes go in `crudkit-rs/crates/crudkit-rs/`
- SeaORM-specific code goes in `crudkit-rs/crates/crudkit-sea-orm/`
- Backend derive macros are in `crudkit-rs/crates/derive-*/`

### When Modifying Web Layer

- Platform-agnostic abstractions go in `crudkit-web/crates/crudkit-web/`
- Web derive macros are in `crudkit-web/crates/derive-*/`

### When Modifying Frontend

- Leptos components go in `crudkit-leptos/crates/crudkit-leptos/`
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
