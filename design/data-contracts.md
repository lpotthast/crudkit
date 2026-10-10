# Data Contracts

CrudKit separates an entity's storage, input, and presentation shapes. It then provides shared representations for
fields, values, identifiers, conditions, and layouts. Generic backend and frontend code can therefore operate without
knowing an application's concrete types.

## Model Roles

The backend `CrudResource` has four model roles:

- **`Model`:** The persisted entity: what currently exists in storage. It has an ID, is serializable, and is used by
  repositories, lifecycle hooks, entity validation, update, and delete.
- **`CreateModel`:** Input accepted before an entity exists. It is deserializable, may omit generated or system-owned
  fields, and need not have an ID.
- **`UpdateModel`:** Input that may be applied to an existing entity. It is deserializable and contains the fields the
  repository may update. Backend selection is carried separately as a `Condition`.
- **`ReadModel`:** Output optimized for queries and presentation. It has an ID, is serializable, and may include joins,
  computed values, or validation status not stored on the main table.

The frontend `Resource` exposes only `CreateModel`, `ReadModel`, and `UpdateModel`. It does not operate on the backend's
storage model directly. Its `ReadModel` must convert into `UpdateModel`, because read and edit views load a presentation
model and then build editable state from it. Frontend update models also have an ID so UI actions and follow-up
navigation can identify the entity.

The roles may use the same Rust type when their shapes genuinely coincide. Keeping separate associated types still
records which contract an operation consumes.

### Why separate roles

A single struct with optional fields for every phase makes “not present in this operation” indistinguishable from
“present and null.” Separate types let required creation input, mutable update input, persisted state, and enriched
read output evolve independently.

### Current write-response constraint

Generated backend create and update routes serialize `SavedV1<R::Model>`. The typed and dynamic web clients
currently deserialize the entity of those bodies as `UpdateModel`. Consequently, the backend `Model` JSON and frontend
`UpdateModel` JSON must currently be wire-compatible for successful write responses.

This is an implementation constraint, not a collapse of the four conceptual roles. A future transport contract that
returns a distinct write-result model must change both route and data-provider sides together.

## Fields

Every model has its own field enum. Field types are deliberately role-specific because a read projection, create input,
update input, and stored model may expose different fields.

- `crudkit-rs::Field` supplies static field names. `FieldLookup` and `ConditionValueConverter` add safe lookup and
  query-value conversion where filtering is supported.
- `crudkit_web::model::FieldAccess<T>` reads and writes values on a concrete model and reports `ValueKind` and
  optionality. Frontend forms need value access; the storage-neutral backend does not.
- The corresponding derives generate enums and implementations, but the traits remain the contract.

A field's wire name is its Rust field name, the name `Field::name()` and `Named::name()` return. Conditions and
ordering keys use that one spelling, and derived field enums serialize as it rather than as their PascalCase variant.
A field name is therefore part of the serialized query and ordering contract. Renaming a field affects clients and
storage mappings even when the Rust compiler can update local references.

## Values and Optionality

`Value` is CrudKit's runtime field-value representation. It supports primitive values, common ecosystem types,
homogeneous arrays, and an extensible `Other` case.

Optionality is metadata on the field, not a second family of `Value` variants:

- `Value::Null` means an optional field currently has no value.
- `Value::Void(())` represents the Rust unit value and is not absence.
- `ValueKind` describes the expected non-optional runtime kind used by derives and default field renderers.
- `Value::Array` is intended to be homogeneous; `verify_array_homogeneity` checks that invariant.
- `Value::Other` stores application-defined `FieldValue` trait objects. `FieldValue` is typetag-enabled so those erased
  custom values can participate in serialization where a containing contract explicitly supports it.
- `Value` itself is not CrudKit's general wire format. Requests serialize concrete models or the dedicated types of
  `crudkit_wire_format::v1`, such as `ConditionClauseValueV1`, `IdValueV1`, and `SerializableIdV1`.
- `TimeDuration` serializes as a signed 64-bit count of microseconds. Values outside that range are currently cast
  without validation.
- A frontend custom renderer is required to present an application-defined `Value::Other` usefully.

Safe `as_*` and consuming `take_*` accessors return `Option`. The `expect_*` accessors deliberately panic on a
mismatched kind and are appropriate only after a typed configuration invariant has established the variant.

## Identifiers

`Id` supports simple and composite identifiers. An ID is ordered, hashable, and composed of named `IdField` values.
Floating-point and optional ID components are excluded because identity requires total equality.

`SerializableId` is the type-erased ID: an ordered vector of `(field name, IdValue)` entries. Its wire form is
`SerializableIdV1`, which writes date-time components in explicit string formats (`YYYY-MM-DDTHH:MM:SS[.fraction]`
and RFC 3339) instead of relying on the `time` crate's Cargo features. It does not encode
a resource name, so callers must retain the resource context separately. Entry names and values are sufficient to
reconstruct a typed ID only when `Id::from_serializable_id` accepts the complete shape.

The backend delete-by-ID operation currently does not perform that reconstruction. It converts the supplied entries
directly into equality clauses. An empty ID therefore produces a match-all condition, while a valid non-ID model field
can become a selector. Operation boundaries must not currently treat arbitrary `SerializableId` input as validated.

Read and update models implement `HasId`; create models normally do not. UI view subjects, entity change notifications,
delete-by-ID requests, parent resolution, and validation storage all use the same `SerializableId` representation.

## Conditions

`Condition` is a serializable query tree:

- `All` means logical AND;
- `Any` means logical OR;
- an empty `All` matches everything;
- an empty `Any`, also exposed as `Condition::none()`, matches nothing.

Leaves carry a field name, an `Operator`, and a `ConditionClauseValue`. Conditions are not raw SQL. A storage adapter
must resolve the field name through the resource's field enum, convert the value for that field, and then build a
backend-specific predicate.

`merge_conditions(a, b)` always combines two present conditions under a new `All`. Data-provider base conditions,
nested-resource conditions, and entity-ID conditions therefore narrow one another. Converting an ID to a condition
produces an `All` of equality clauses, one per ID component.

Not every `Value` or `IdValue` can be used in a condition. Field converters and storage adapters reject unsupported
combinations with errors; see the [SeaORM adapter](sea-orm-adapter.md#unsupported-condition-cases).

The built-in wire enum has vector variants only for `u8`, `i32`, and `i64`. Durations are filtered by their wire
format, `i64` microseconds, given as a number or a string. The JSON converter rejects `ConditionClauseValue::Json`
and converts a string to `Value::String` rather than `Value::Json`. Applications must not rely on JSON filtering
through this converter yet.

## Layout Descriptions

`crudkit-web::layout` describes form structure independently of Leptos components. `Elem` can contain a field with
`FieldOptions`, a separator, or an enclosing group. Enclosures support plain groups, tabs, and cards. Create and update
layouts use their own erased field types, preventing a field from the wrong model role from being inserted accidentally.

`Layout` records a one- through four-column choice. `CrudFormLayout` exposes it as `data-columns` on each
`CrudFormGroup`; application styles decide whether and how to honor it. Tabs, cards, separators, and child order are
implemented.

## Frontend Type Erasure

The frontend uses three tiers:

1. typed `Model` and `FieldAccess<T>` implementations for compile-time construction;
2. object-safe `Erased*` traits for runtime polymorphism and serialization;
3. `Dyn*` wrappers that provide cloning, equality, hashing where needed, and explicit downcasting.

Models are boxed because a view owns and mutates one model value. Fields are `Arc`-backed because registries, layouts,
maps, and renderers clone field handles frequently. Typetag supplies serialization for erased values.

`crudkit-web`'s `ModelHandler` closes over the concrete create, read, and update types. It owns response
deserialization, read-to-update conversion, default create-model construction, field-value enumeration, and create-field
lookup. After that handler is built, `CrudInstance` can remain non-generic.

Downcast helpers panic on a wrong concrete type. This is intentional fail-fast behavior for incompatible application
configuration, not validation for untrusted wire data; wire JSON is deserialized into a concrete type before it is
wrapped.

## Derive Boundary

The traits described here remain the contracts; applications may implement them manually. Generated implementations,
naming conventions, and ownership by derive family are described in [Derive Macros](derive-macros.md).
