# Derive Macros

CrudKit's derive crates generate implementations of contracts owned by the runtime crates. This document owns
generation rules and cross-macro consistency; model meaning, query semantics, and storage behavior remain in their
focused documents.

## Ownership by Layer

- **`crudkit-core-macros`:** Typed IDs, ID-field enums, `HasId`, and conversion to or from `SerializableId`.
- **`crudkit-rs-macros`:** Backend field metadata and lookup, condition conversion, model-role DTOs, and
  resource-context markers.
- **`crudkit-web-macros`:** Frontend resource association, value-accessing field enums, erased model or field
  implementations, and action payload support.
- **`crudkit-sea-orm-macros`:** SeaORM column bridges, generated DTO-to-ActiveModel behavior, read projections, and
  legacy adapter-specific validation models.

`crudkit-core-macro-util` centralizes Rust-type classification into `ValueKind` so backend and frontend generation agree
on primitive categories. `crudkit-rs-macros-core` generates the common create and update DTO shape used by both the
storage-neutral and SeaORM derives.

## Identifier Generation

`CkId` selects fields annotated with `#[ck_id(id)]`; by convention, an unannotated field named `id` is also an ID field.
It generates a typed `{Model}Id`, a `{Model}IdField` enum, `HasId` for the model, and the serialization bridge.

Composite IDs retain every selected field. Optional and floating-point components are rejected because `Id` requires
stable equality, ordering, and hashing. Generated reconstruction checks that a `SerializableId` has the expected named
and typed entries.

## Backend Field Generation

Backend `CkField` generates one enum variant per field and implementations for:

- the backend `Model` association;
- static field naming;
- lookup from a wire field name;
- conversion from `ConditionClauseValue` to the field's expected `Value` kind.

Both the backend and the frontend `CkField` rename every variant's serde representation to the field name, so a field
enum serializes as the same name its `name()` returns.

`#[ck_field(convert_ccv = "...")]` is the explicit escape hatch when the built-in conversion is insufficient. It changes
query-value parsing and must remain consistent with the storage adapter's handling of the resulting `Value`.

ID generation remains the responsibility of `CkId`; backend field generation must not be treated as an alternative
source of identity.

## Frontend Field Generation

Frontend `CkField` is role-aware: create, read, and update models receive distinct erased field traits. It generates
model field enumeration, typed get/set behavior, `ValueKind`, optionality, and erased wrappers used by `CrudInstance`.
It also implements `IntoDynField` for the field enum, naming its erased field type (`DynCreateField`, `DynReadField`,
or `DynUpdateField`), so that `crudkit-leptos`' `CrudField` atom binds a typed field to the matching form.

Optional Rust fields map absence to `Value::Null`; they do not generate optional `Value` variants. Unsupported custom
types classify as `Other` and require application rendering or conversion behavior.

The generated setter uses fail-fast value access for non-optional built-in fields. Supplying a `Value` of the wrong kind
is a configuration/programming error and may panic.

## Create and Update DTO Generation

Storage-neutral `CkCreateModel` and `CkUpdateModel` use shared token generation. The SeaORM variants invoke the same
generator and add the adapter behavior described in [SeaORM Adapter](sea-orm-adapter.md). This prevents the generic DTO
and ORM conversion macro from drifting into different field sets.

Attributes such as `exclude`, `optional`, and `use_default` affect externally visible request or persistence semantics.
They are not cosmetic code-generation options and should be reviewed like handwritten DTO changes.

## Resource and Adapter Bridges

Frontend `CkResource` derives the `Resource` association from naming conventions or explicit model overrides. Its
`resource_name` must match the backend resource name on the wire.

`CkSeaOrmBridge` maps generated CrudKit field enums to SeaORM column enums. A rename must update both sides; silently
falling back to an arbitrary column is not allowed.

`ReadView` generates a read projection with `has_validation_errors`. It is paired with the SQL read-view helpers, not a
general projection generator.

## Legacy Validation Derive

`CkValidationModel` and its per-resource validation traits remain public, but the active
`UnifiedValidationRepository` uses one shared table and does not depend on those generated models. New design work
should not assume both approaches are interchangeable without first reconciling or removing the legacy surface.

## Traits Remain Authoritative

Applications may implement all generated contracts by hand. Runtime code must depend on the traits and serialized
shapes, not on generated type names or a macro's internal token structure.

The `Model` traits in `crudkit-core`, `crudkit-rs`, and `crudkit-web` are parallel current contracts with similar
associated-field shapes; they do not extend one another in Rust. Derives implement the layer-specific trait they target.
