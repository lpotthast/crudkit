# Validation Architecture

This document describes the validation architecture in crudkit-rs.

## Overview

The validation system supports two types of validators and two validation modes, enabling both immediate feedback during
CRUD operations and comprehensive system-wide validation.

## Validator Types

### 1. EntityValidator

Validate a single entity or an entity change.

- **Trait:** `EntityValidator<R: CrudResource>`
- **Scope:** Single entity at a time
- **Identity:** Each validator has `get_name()` and `get_version()` for result tracking
- **Methods:**
    - `get_name()` - Returns the unique name of this validator
    - `get_version()` - Returns the version of this validator (increment when logic changes)
    - `validate_single(&entity, trigger)` - Validate an entity
    - `validate_updated(&old, &new, trigger)` - Validate an entity change
- **Usage:** Adhoc validation during CRUD operations

### 2. AggregateValidator

Validate all entities of a resource type.

- **Trait:** `AggregateValidator<R: CrudResource>`
- **Scope:** All entities of a type in the system
- **Identity:** Each validator has `get_name()` and `get_version()` for result tracking
- **Methods:**
    - `get_name()` - Returns the unique name of this validator
    - `get_version()` - Returns the version of this validator
    - `validate_all()` - Validate all entities
- **Usage:** Global validation that runs asynchronously after CRUD operations

## Validation Modes

### 1. Adhoc Validation (Synchronous)

Runs as part of CRUD operations. Results are returned in the HTTP response.

- **When:** Before and after create/update/delete operations
- **Validators:** EntityValidator
- **Response:**
    - Critical violations block the operation (HTTP 422)
    - Major violations allow the operation but are included in the response
    - All violations are returned synchronously to the requestor

### 2. Global Validation (Asynchronous)

Runs after CRUD operations complete. Results are broadcast via WebSocket.

- **When:** After successful create/update/delete operations
- **Validators:** AggregateValidator
- **Delivery:** WebSocket broadcast to all connected users
- **Use case:** Cross-entity validation, system-wide consistency checks

## Violation Severity

### Critical

- **Purpose:** Block invalid operations
- **Behavior:** Prevents create/update/delete from completing
- **HTTP Status:** 422 Unprocessable Entity
- **Persistence:** NOT persisted (blocking means no entity state to associate with)

### Major

- **Purpose:** Warn about data quality issues
- **Behavior:** Allows operation to complete, includes violations in response
- **HTTP Status:** 200 OK with `violations` field containing all non-critical violations
- **Persistence:** Stored in validation result repository

## Per-Operation Behavior

### Create

```
Request → before_create hook → BEFORE validation
                                  ↓
                         Critical violations? ─YES→ HTTP 422 + violations
                                  ↓ NO
                            Insert entity
                                  ↓
                         after_create hook → AFTER validation
                                  ↓
                         WebSocket: entity violations broadcast
                                  ↓
                         Global validation → WebSocket: aggregate violations
                                  ↓
                         HTTP 200 + Saved<T> + violations
```

### Update

```
Request → before_update hook → BEFORE validation
                                  ↓
                         Critical violations? ─YES→ HTTP 422 + violations
                                  ↓ NO
                            Update entity
                                  ↓
                         after_update hook
                                  ↓
                         WebSocket: entity violations broadcast
                                  ↓
                         Global validation → WebSocket: aggregate violations
                                  ↓
                         HTTP 200 + Saved<T> + violations
```

### Delete

```
Request → before_delete hook → BEFORE validation
                                  ↓
                         Critical violations? ─YES→ HTTP 422 + violations
                                  ↓ NO
                            Delete entity
                                  ↓
                         after_delete hook
                                  ↓
                         WebSocket: entity deleted notification
                                  ↓
                         Global validation → WebSocket: aggregate violations
                                  ↓
                         HTTP 200 + Deleted
```

**Note:** Delete validation is edge-case since the data doesn't change. Critical violations on delete are primarily for
time-sensitive business rules.

## Example Flow

1. User tries to update entity A
2. Critical validation violation detected
3. Update blocked, HTTP 422 returned with all violations
4. User fixes inputs and tries again
5. Entity A updated successfully
6. User receives `Saved<A>` with major violations synchronously
7. All users receive A's violations via WebSocket
8. Global validation triggered
9. AggregateValidator checks all entities of A's type
10. Violation found in entity B (triggered by A's content change)
11. Global validation result broadcast to all users via WebSocket

## Response Formats

### Success with Violations

```json
{
  "entity": {
    /* entity data */
  },
  "violations": {
    "general": null,
    "create": null,
    "by_entity": {
      "[entity-id]": [
        { "Major": "Warning message" }
      ]
    }
  }
}
```

### Validation Failed (HTTP 422)

```json
{
  "error": "Validation failed with critical errors.",
  "violations": {
    "general": null,
    "create": [
      {
        "Critical": "Field 'name' is required"
      }
    ],
    "by_entity": {
      "[entity-id]": [
        {
          "Major": "Name should be longer"
        }
      ]
    }
  }
}
```

### WebSocket Validation Broadcast

```json
{
  "PartialValidationResult": {
    "ResourceType": {
      "general": null,
      "create": null,
      "by_entity": {
        "[entity-id]": [
          {
            "Major": "Warning message"
          }
        ]
      }
    }
  }
}
```

## Frontend Message Deduplication

The user who initiates a CRUD operation receives validation results through two channels:

1. **Synchronously** - In the HTTP response (`Saved<T>` with `violations` field)
2. **Asynchronously** - Via WebSocket broadcast (sent to all connected users)

This is intentional to keep the backend simple. The frontend (crudkit-leptos) is responsible for deduplicating
these messages. When the same violation is received both synchronously and via WebSocket, the frontend should
only display it once.

## Configuration

### CrudContext

Validators are provided when constructing `CrudContext`. Each resource can have multiple validators:

```rust
pub struct CrudContext<R: CrudResource> {
    pub res_context: Arc<R::Context>,
    pub repository: Arc<R::Repository>,
    pub validators: Vec<Arc<dyn EntityValidator<R>>>,
    pub aggregate_validators: Vec<Arc<dyn AggregateValidator<R>>>,
    pub validation_result_repository: Arc<R::ValidationResultRepository>,
    pub ws_controller: Arc<R::WebsocketService>,
    pub global_validation_state: Arc<GlobalValidationState>,
}
```

### Default Implementations

- `AlwaysValidValidator` - No-op entity validator (no violations)
- `NoAggregateValidator` - No-op aggregate validator (no global validation)

## Adding Multiple Validators

Validators are provided as a `Vec` when constructing `CrudContext`, allowing unlimited validators per resource:

```rust
use crudkit_rs::prelude::*;
use std::sync::Arc;

// Define your validators
struct NameValidator {
    min_length: usize
}
struct EmailValidator {
    allowed_domains: Vec<String>
}
struct UniquenessValidator<R> {
    repository: Arc<R::Repository>
}

impl<R: CrudResource> EntityValidator<R> for NameValidator {
    fn get_name(&self) -> &'static str { "NameValidator" }
    fn get_version(&self) -> u32 { 1 }

    fn validate_single(&self, entity: &R::ActiveModel, trigger: ValidationTrigger) -> EntityViolations<R::Id> {
        // validation logic
    }

    fn validate_updated(&self, old: &R::ActiveModel, new: &R::ActiveModel, trigger: ValidationTrigger) -> EntityViolations<R::Id> {
        // validation logic
    }
}

// When creating the CrudContext:
let validators: Vec<Arc<dyn EntityValidator<MyResource> > > = vec![
    Arc::new(NameValidator { min_length: 3 }),
    Arc::new(EmailValidator { allowed_domains: vec!["example.com".to_string()] }),
    // ... unlimited validators
];

let aggregate_validators: Vec<Arc<dyn AggregateValidator<MyResource> > > = vec![
    Arc::new(UniquenessValidator { repository: repository.clone() }),
    // ... unlimited validators
];

let context = CrudContext {
validators,
aggregate_validators,
// ...
};
```

### Validator Identity

Each validator provides `get_name()` and `get_version()` methods. This allows the system to:

1. **Track validation results**: Associate violations with the validator that produced them
2. **Detect obsolete results**: When a validator version increases, previously stored violations from older versions can
   be identified as potentially outdated
3. **Debug and audit**: Know which validator produced which violation

When implementing a validator, increment the version number whenever the validation logic changes. This signals that old
stored violations may no longer be accurate.
