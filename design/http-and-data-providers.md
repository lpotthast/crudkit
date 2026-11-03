# HTTP and Data Providers

CrudKit's current transport pair is generated Axum routes in `crudkit-rs` and Reqwest-based clients in `crudkit-web`:
the REST data source, which an instance uses by default, and the typed REST data provider. Both sides exchange the
bodies of `crudkit_wire_format::v1`; see
[Wire Format Versions](#wire-format-versions). Paths, resource names, and some response shapes are still declared on
each side separately, so both sides must evolve together.

## Generated Route Set

`impl_add_crud_routes!(Resource, name)` creates one module and eight routes below a caller-supplied root:

| Path suffix                        | Request                    | Success body                    |
|------------------------------------|----------------------------|---------------------------------|
| `/{resource}/crud/v1/read-count`   | `ReadCountV1`              | `ReadCountResponseV1`           |
| `/{resource}/crud/v1/read-one`     | `ReadOneV1`                | `ReadOneResponseV1<ReadModel>`  |
| `/{resource}/crud/v1/read-many`    | `ReadManyV1`               | `ReadManyResponseV1<ReadModel>` |
| `/{resource}/crud/v1/create-one`   | `CreateOneV1<CreateModel>` | `SavedV1<Model>`                |
| `/{resource}/crud/v1/update-one`   | `UpdateOneV1<UpdateModel>` | `SavedV1<Model>`                |
| `/{resource}/crud/v1/delete-by-id` | `DeleteByIdV1`             | `DeletedV1`                     |
| `/{resource}/crud/v1/delete-one`   | `DeleteOneV1`              | `DeletedV1`                     |
| `/{resource}/crud/v1/delete-many`  | `DeleteManyV1`             | `DeletedManyV1`                 |

Every failure, on every route, answers with an `ErrorResponseV1`; see [Error Translation](#error-translation).

Every route currently uses HTTP POST, including reads and deletes. Request bodies carry query conditions, ordering,
pagination, IDs, or model DTOs. The generated module also derives an OpenAPI description for the same surface.

## Wire Format Versions

`crudkit-wire-format` defines one module per wire format version. A version's types carry its suffix (`ReadManyV1`),
and they are data transfer objects only: generated routes and web providers convert them into internal types at the
transport boundary. A breaking change adds a new version module instead of changing an existing one.

The version appears twice, deliberately:

- in the route path (`/crud/v1/...`), so routing, proxies, and logs see it without reading bodies;
- in every request and response body as `"wire_format_version": 1`, so a body stays self-describing when it is
  stored, forwarded, or received without its path. `WireFormatVersion::detect` reads only that field, which lets a
  future server select the version from the data alone.

A version's marker type accepts only its own number, so a body of another version fails to deserialize instead of
being read with the wrong shape. A request whose path and body disagree therefore fails with a client error.

Fixture tests in `crudkit-wire-format` pin the JSON of every `v1` type. `v1` covers every serialized contract between
clients and servers and is released: it is frozen, and a fixture is never edited to make a test pass. A breaking change
adds `v2` next to it.

## Request Bodies

`crudkit_wire_format::v1` defines every request body once, together with the conditions, IDs, and ordering it
carries. Generated routes deserialize the `V1` types and convert them into the internal requests of
`crudkit_core::request` before calling an operation; the web providers build internal requests and convert them into
`V1` types before sending. The conversions live in `crudkit_core::wire_v1`.

Ordering criteria are keyed by field name, the same name conditions use; see [Data Contracts](data-contracts.md#fields).
The wire format therefore does not depend on any field enum. Providers name fields through `Named::name()`, and routes
resolve them through the field enum's `FieldLookup`. An unknown ordering field fails the request with 400 Bad Request
before the operation runs. The OpenAPI description of `order_by` is a plain object.

The backend resource name comes from `CrudResource::TYPE.name()`. The frontend resource name comes from
`crudkit_web::resource::Resource::resource_name()`, or from the resource name given to the REST data source. They are
separate declarations and must match exactly.

Authentication-presence checks happen in the generated route before the operation. Fine-grained authorization and
row scoping follow [Backend Operations](backend-operations.md).

## Typed Provider

`CrudRestDataProvider<T: Resource>` preserves concrete create, read, and update types. It serializes request DTOs,
calls the stable path suffixes above, and deserializes successful JSON directly. It shares one crate-private endpoint
with the REST data source. The endpoint owns the base URL, executor, resource name, the typed provider's base
condition, and path construction.

Both clients depend on `ReqwestExecutor`, not a concrete client. Implementations currently support a plain
`reqwest::Client` and a new-client-per-request fallback. The opt-in `keycloak-auth` feature adds Leptos Keycloak's
authenticated client; it is a feature so that `crudkit-web` does not depend on Leptos by default. This injection point
owns request authentication and client construction; resource code does not add transport-specific credentials.

## REST Data Source

`CrudRestDataSource` is the [data source](data-sources.md) of a resource served by the generated routes.
`CrudInstanceConfig::new::<R>(api_base_url, executor)` configures an instance with it, so an application on
`crudkit-rs` names only the base URL and the executor. The data source holds the resource's
`crudkit_web::model_handler::ModelHandler`, which deserializes response entities into the concrete models chosen
during configuration, so reads return `DynReadModel`s and writes return `Saved<DynUpdateModel>`.

The data source maps its operations to the routes:

| Operation     | Route                        | Condition                     |
|---------------|------------------------------|-------------------------------|
| `list`        | `read-many` and `read-count` | scope                         |
| `read`        | `read-one`                   | scope AND the ID's components |
| `create`      | `create-one`                 | none                          |
| `update`      | `update-one`                 | scope AND the ID's components |
| `delete`      | `delete-by-id`               | scope, next to the ID         |
| `delete_many` | `delete-many`                | scope AND any of the IDs      |

`list` sends both requests at once and answers with `CrudPaging::Offset`, whose total is the count. It can order by
every field.

The routes are flat, so the data source turns a `CrudScope` into a condition: each parent becomes an equality clause
on the field that refers to it, the clauses are combined with AND, and the scope's condition is added with AND. A
parent whose ID cannot be compared for equality fails the request with `RequestError::InvalidRequest`, because leaving
it out would widen the operation. A creation sends the entity as it is: the instance fills the fields that refer to the
scope's parents in the create model.

Erased create and update models serialize with a typetag wrapper. Before sending them to the generated server DTO,
the data source calls the model wrapper's `to_untagged_json`, which requires an object with exactly one outer
type-tag entry and removes that entry. This keeps type information inside the client runtime while sending the
untagged model object expected by Axum. A model that cannot be serialized this way fails with
`RequestError::InvalidRequest` before anything is sent.

Malformed erased serialization becomes `RequestError::InvalidRequest`, and a response entity that does not match the
handler's model becomes `RequestError::Deserialize`. Neither is recovered by trying another model type.

The typed provider does not yet cover every generated route: it lacks delete-one and delete-many. The REST data source
uses delete-by-ID and delete-many. Neither uses delete-one.

## Write Response Compatibility

The server returns `SavedV1<Model>` for create and update. Both web clients currently expect the entity portion
to deserialize as the frontend `UpdateModel`. The JSON shapes must therefore be compatible, as described in
[Data Contracts](data-contracts.md).

## Base Conditions

The typed provider can store a base condition, which it combines with the request condition using logical AND. It
applies the base condition to count, read-one, read-many, update, and delete-by-ID, but not to create. The REST data
source has no base condition of its own: an instance's base condition reaches it as the condition of the instance's
scope.

`DeleteById` carries an optional `condition` next to the ID. The server deletes the entity only if it matches both;
otherwise the request fails as not found. Requests without the field delete by ID alone.

An instance passes its scope to every operation, so single and mass deletions stay within the instance's base
condition and parents, and a view cannot delete outside of what it shows. These conditions are convenience scoping and
UI composition, not access control:

- clients can call the API without using a data source;
- create has no selection condition;
- a malicious caller controls request bodies.

Server-side hooks or repository constraints must enforce security-sensitive scoping.

## Error Translation

Every failed request answers with an `ErrorResponseV1` body. Its `error` is tagged by `kind`, and the generated
routes pair each kind with an HTTP status:

| Kind                         | Status | Meaning                                                       |
|------------------------------|--------|---------------------------------------------------------------|
| `bad_request`                | 400    | malformed body, unknown field, or unsupported condition       |
| `unauthorized`               | 401    | required authentication missing                               |
| `forbidden`                  | 403    | lifecycle authorization rejection                             |
| `not_found`                  | 404    | selected entity not found                                     |
| `unprocessable_entity`       | 422    | lifecycle business rejection                                  |
| `critical_validation_errors` | 422    | critical validation; the body carries the violations          |
| `internal_server_error`      | 500    | repository, lifecycle-internal, or validation-storage failure |

A request body that axum cannot deserialize, including one of another wire format version, becomes `bad_request`
instead of axum's plain-text rejection. Internal failures are logged with their detailed `CrudError`; generated 500
responses expose only generic messages.

The web client classifies a failure by the body's `kind`, not by the status, so the transport's status mapping is
not part of the contract. Only a response without a version 1 error body, e.g. from a proxy, is classified by its
status. `RequestError` is also the error type of failure callbacks; `CriticalValidationErrors` carries the violations.
Requests the client cannot build (`InvalidRequest`), transport failures (`Request`), and successful-body
deserialization failures (`Deserialize`) are represented separately from server failures.
