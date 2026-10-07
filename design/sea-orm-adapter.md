# SeaORM Adapter

`crudkit-sea-orm` implements the storage-neutral backend contracts with SeaORM. ORM types stay in the adapter;
`crudkit-rs` operations continue to use the application-level models from
[Data Contracts](data-contracts.md).

## Resource Bridge

`SeaOrmResource` extends `CrudResource` with two query surfaces:

- The main entity handles inserts, hook and validation fetches, updates, and deletes. Its associated types describe
  the entity, model, active model, columns, and primary key.
- The read view handles read-one and read-many presentation queries. Its associated types describe the read entity,
  read model, read active model, read columns, and read primary key.

The two surfaces may describe the same database object, but they are separate associated types so a resource can use
a SQL view with joined or computed fields for reads.

Explicit `model_field_to_column` and `read_model_field_to_column` functions restore storage typing after a request
condition or ordering crossed the wire using field descriptions.

## Create, Update, and Delete

`SeaOrmRepo` performs the following conversions:

- create DTO → SeaORM ActiveModel → inserted SeaORM model → application persisted model;
- existing application model → ActiveModel, then apply update DTO → updated SeaORM model → application persisted
  model;
- application model → ActiveModel → delete;
- read-view SeaORM model → application read model.

The repository returns adapter errors to the operation engine and does not invoke hooks, validation, or collaboration
itself.

## Generated DTO Semantics

SeaORM create and update derives reuse the storage-neutral DTO generator, then add ActiveModel behavior.

For generated create models:

- a required included field becomes `Set(value)`;
- an optional included field becomes `Set(value)` for `Some` and `NotSet` for `None`;
- an excluded field becomes `NotSet` unless `use_default` explicitly requests `Set(Default::default())`.

For generated update models:

- a required included field is written explicitly;
- an optional included field writes `Some(value)` and leaves storage unchanged for `None`;
- an excluded field is unchanged unless `use_default` explicitly writes a default.

Optional update fields therefore mean “update when present,” not “write SQL NULL.” A nullable field that must
distinguish unchanged from set-null needs a richer application DTO than the generated single `Option<T>` convention.

## Query Translation

The adapter recursively translates `Condition::All` and `Condition::Any` into SeaQuery conditions. Each clause
follows this path:

1. resolve the string field name through `FieldLookup`;
2. convert `ConditionClauseValue` through that field's `ConditionValueConverter`;
3. map the typed field to a SeaORM column;
4. translate the operator and `Value` into a SeaQuery predicate.

Unknown fields and failed field conversions return `SeaOrmRepoError`. Ordering uses the same explicit field-to-column
bridge.

### Unsupported condition cases

The condition AST is broader than what the adapter can translate. These clauses return
`SeaOrmRepoError::UnsupportedCondition` with the column and the reason:

- scalar `IsIn`, list values with another operator, heterogeneous lists, and nested lists;
- `Null` with an operator other than equality and inequality;
- `i128` and `u128` values, which SeaORM cannot represent, as well as `Void` and `Other`.

Translation never drops a clause, so a rejected clause fails the whole query rather than widening its result. A
serialized `Condition` is not proof that every adapter can execute it. Like the other repository errors, these
currently surface as server errors.

## Unified Validation Storage

The built-in CrudKit migrator creates one `CrudkitValidation` table for all resources. Rows contain:

- resource name;
- entity ID as JSON in the explicit `SerializableIdV1` format, independent of how internal types would serialize;
- validator name and version;
- severity and message;
- creation timestamp.

JSON IDs preserve composite keys without creating one validation table schema per resource. Queries first scope by
resource name; the migration intentionally does not create a normal B-tree index for the JSON entity ID.

Read views compare this column with JSON built in SQL, so the stored format is part of their contract. Date-time ID
components are stored as `YYYY-MM-DDTHH:MM:SS[.fraction]` or RFC 3339 strings. Rows written before this format was made
explicit stored them in whatever form the `time` crate's Cargo features selected, and do not match after the change.

Replacing results for one entity and validator uses a database transaction: delete rows for that validator with
versions less than or equal to the current version, then insert the new findings. The higher-level `save_all` error
behavior is documented in [Validation and Collaboration](validation-and-collaboration.md).

The public `CkValidationModel` and per-resource validation-model traits remain in the crate, but
`UnifiedValidationRepository` does not implement a per-resource-table mode. They are legacy adapter surface, not a
second active persistence strategy.

## Read-View Migrations

Application migrations may use `create_read_view`, `drop_read_view`, or `impl_read_view_migration!` to maintain a
`{Table}ReadView`. The generated view selects all main-table columns and appends `has_validation_errors`, computed
with an `EXISTS` lookup into `CrudkitValidation` using the resource name and serialized ID.

These helpers generate PostgreSQL-specific SQL and quote table names according to their current convention. The
generic `Repository` contract is storage-neutral; the bundled read-view implementation is not.

CrudKit's own `Migrator` installs only the unified validation table. Each application's resource read views remain
application migration responsibility.
