# Four-Model Architecture

This document describes the four-model architecture in CrudKit.

The four-model architecture ensures type safety across the full stack while maintaining clear separation between what
the client sends, what the server stores, and what gets returned for display.

## Overview

CrudKit uses four distinct model types to represent an entity throughout its lifecycle. Each model serves a specific
purpose, ensuring type safety and clarity about what data flows where.

| Model       | Purpose          | Contains                           |
|-------------|------------------|------------------------------------|
| Model       | Canonical entity | All fields.                        |
| CreateModel | Creation input   | Fields requried for creation.      |   
| UpdateModel | Update input     | ID + mutable fields.               | 
| ReadModel   | Read output      | All fields + computed/joined data. |

## The Four Models

### 1. Model

The canonical representation of en entity ("what exists").

- Contains all fields (both mutable and read-only).
- Represents the "source of truth" for an entity.
- Used internally by both frontend and backend for lifecycle hooks, validation, and business logic.
- Returned by repository operations after an insert or update.
- Broadcast via collaboration channel when entities change.

**Example:**

```rust
pub struct Article {
    pub id: i64,                 // Primary key (auto-generated, unmodifiable)
    pub title: String,           // User-provided, freely modifiable
    pub content: String,         // User-provided, freely modifiable
    pub author_id: UserId,       // Automatically set by lifecycle hook (e.g. from auth context)
    pub created_at: DateTime,    // Metadata (automatically set by system: on create)
    pub updated_at: DateTime,    // Metadata (automatically set by system: on every update)
}
```

### 2. CreateModel

The data required to create a new entity.

- Subset of `Model` fields (omitting all automatically set fields).
- In the creation UI (create view) only fields from the CreateModel can be shown to the user.
- Sent by the frontend to the backend when creating an entity.

**Example:**

```rust
pub struct CreateArticle {
    pub title: String,
    pub content: String,
}
```

### 3. UpdateModel

The data that can be modified on an existing entity.

- Contains the entities ID (all fields constructing it) to identify what to update.
- Contains all mutable fields.
- Excludes read-only and system-managed fields.

**NOTE:** Frontend crud instance configuration must ensure that the id fields are either not shown at all or only shown
unmodifiable (disabled). Through request forgery, an attacker could try to modify another entity than shown to be
editable. This is currently undetectable by the backend, as no user session state is persisted. If we would do that,
something like: "user now in update view for entity with id X" → "user sent update for entity Y" → "reject:
implausible" would be possible. But: Access rules still apply. If a lifecycle hook would determine that entity with ID Y
is not editable by the current user, the update request would be rejected.

**Example:**

```rust
pub struct UpdateArticle {
    pub id: i64,
    pub title: String,
    pub content: String,
}
```

### 4. ReadModel

The data returned to the client for display.

- May be identical to Model (simple case).
- May include computed/derived fields.
- May include joined data from related entities.
- Often backed by a database VIEW for performance.
- Optimized for read operations and UI (table/list view) display.

**Example:**

```rust
pub struct ArticleView {
    // All fields from `Model`.
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: UserId,
    pub created_at: DateTime,
    pub updated_at: DateTime,

    // Joined data (from related tables).
    pub author_name: String,
    pub author_avatar_url: String,

    // Computed fields.
    pub word_count: i32,
    pub is_recent: bool,

    // CrudKit validation status (from related table).
    pub has_violations: bool,
}
```

### Common Simplifications

In simple cases, some models may be identical. Just use the same model multiple times when defining a resource.

The framework supports these simplifications while maintaining the conceptual separation.

## Design Rationale

### Why four models?

**Type safety:** Each operation has distinct requirements. Mixing them leads to optional fields, runtime checks, and
bugs.

**Clarity:** When reading code, the type tells you exactly what context you're in.

**Flexibility:** Models can evolve independently. Adding a computed field to ReadModel doesn't affect CreateModel.

**Validation:** Each model can have operation-specific validation rules.

### Why not just one model?

A single model with optional fields creates problems:

```rust
// Antipattern: Single model with optional fields.
pub struct Article {
    pub id: Option<ArticleId>,        // None on create, Some on update/read.
    pub title: String,
    pub content: String,
    pub author_id: Option<UserId>,    // Irrelevant on create (set by system, `Some` always overwritten).
    pub created_at: Option<DateTime>, // Irrelevant on create (set by system, `Some` always overwritten).

    pub author_name: Option<String>,  // Only present in reads.
}
```

Problems:

- Needlessly requires `Option` for non-optional(!) data, just to support all processes.
- Unclear which fields must be set in which context.
- Unclear which fields can be expected to always be present (unwrappable option) in which context.
- Potential for runtime panics when wrongly accessing optional fields.
- Difficult to enforce invariants.

### Why `Model` in addition to `UpdateModel`?

The UpdateModel represents "what can change" while Model represents "what exists". These diverge when:

- Fields are immutable after creation (`author_id`).
- Fields are system-managed (`created_at`, `updated_at`).
- Fields are derived/computed and never directly set.

Lifecycle hooks and validators should receive "the full entity", not just the update delta:

```rust
async fn before_delete(
    model: &Article,  // The full entity, not UpdateModel!
    context: &MyContext,
    request: RequestContext<Auth>,
) -> Result<(), HookError<MyError>> {
    // Check ownership using author_id (not in UpdateModel).
    if model.author_id != request.auth.user_id {
        return Err(HookError::Forbidden {
            reason: "Only the author can delete".into()
        });
    }
    Ok(())
}
```

### Why `ReadModel` separate from `Model`?

ReadModel often includes:

- Joined data from related tables (avoiding N+1 queries), e.g. validation state.
- Computed/derived fields.
