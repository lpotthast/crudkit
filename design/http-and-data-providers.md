# HTTP and Data Providers

CrudKit's current transport pair is generated Axum routes in `crudkit-rs` and Reqwest-based data providers in
`crudkit-web`. They share JSON shapes by convention rather than through a common Rust resource type, so both sides
must evolve together.

## Generated Route Set

`impl_add_crud_routes!(Resource, name)` creates one module and eight routes below a caller-supplied root:

| Path suffix | Request | Success body |
| --- | --- | --- |
| `/{resource}/crud/read-count` | `ReadCount` | `u64` |
| `/{resource}/crud/read-one` | `ReadOne<R>` | `ReadModel` |
| `/{resource}/crud/read-many` | `ReadMany<R>` | `Vec<ReadModel>` |
| `/{resource}/crud/create-one` | `CreateOne<CreateModel>` | `Saved<Model>` |
| `/{resource}/crud/update-one` | `UpdateOne<UpdateModel>` | `Saved<Model>` |
| `/{resource}/crud/delete-by-id` | `DeleteById` | `Deleted` |
| `/{resource}/crud/delete-one` | `DeleteOne<R>` | `Deleted` |
| `/{resource}/crud/delete-many` | `DeleteMany` | `DeletedMany` |

Every route currently uses HTTP POST, including reads and deletes. Request bodies carry query conditions, ordering,
pagination, IDs, or model DTOs. The generated module also derives an OpenAPI description for the same surface.

The backend resource name comes from `CrudResource::TYPE.name()`. The frontend resource name comes from
`crudkit_web::Resource::resource_name()` or dynamic instance configuration. They are separate declarations and must
match exactly.

Authentication-presence checks happen in the generated route before the operation. Fine-grained authorization and
row scoping follow [Backend Operations](backend-operations.md).

## Typed Provider

`CrudRestDataProvider<T: Resource>` preserves concrete create, read, and update types. It serializes request DTOs,
calls the stable path suffixes above, and deserializes successful JSON directly.

The provider depends on `ReqwestExecutor`, not a concrete client. Implementations currently support a plain
`reqwest::Client`, Leptos Keycloak's authenticated client, and a new-client-per-request fallback. This injection point
owns request authentication and client construction; resource code does not add transport-specific credentials.

## Dynamic Provider

`DynCrudRestDataProvider` is used after a `CrudInstance` has erased its concrete resource types. Reads return raw
`serde_json::Value`; `ModelHandler` then deserializes that JSON into the concrete model chosen during configuration.

Erased create and update models serialize with a typetag wrapper. Before sending them to the generated server DTO,
the dynamic provider requires an object with exactly one outer type-tag entry and removes that entry. This keeps type
information inside the client runtime while sending the untagged model object expected by Axum.

Malformed erased serialization or a mismatched `ModelHandler` becomes a request or deserialization error. It is not
recovered by trying another model type.

The client surfaces do not yet cover every generated route. The typed provider lacks delete-one and delete-many. The
dynamic provider implements delete-many but not delete-one. Both implement delete-by-ID.

## Write Response Compatibility

The server returns `Saved<Model>` for create and update. Both web providers currently expect the entity portion to
deserialize as the frontend `UpdateModel`. The JSON shapes must therefore be compatible, as described in
[Data Contracts](data-contracts.md).

## Base Conditions

Both providers can store a base condition. When supported by an operation, the provider combines it with the request
condition using logical AND.

The typed provider applies its base condition to count, read-one, read-many, and update. The dynamic provider also
applies it to delete-many. Neither provider applies it to create or delete-by-ID.

`CrudInstance` also builds conditions explicitly for its list, read, edit, update, parent, and mass-delete behavior.
These conditions are convenience scoping and UI composition, not access control:

- clients can call the API without using a data provider;
- delete-by-ID does not carry a base condition;
- create has no selection condition;
- a malicious caller controls request bodies.

Server-side hooks or repository constraints must enforce security-sensitive scoping.

## Error Translation

Backend operation errors are mapped to HTTP status families:

| Status | Meaning |
| --- | --- |
| 400 | malformed condition or request |
| 401 | required authentication missing |
| 403 | lifecycle authorization rejection |
| 404 | selected entity not found |
| 422 | lifecycle business rejection or critical validation |
| 500 | repository, lifecycle-internal, or validation-storage failure |

Internal failures are logged with their detailed `CrudError`; generated 500 responses expose only generic messages.

The web client maps statuses to `RequestError` and then to the callback-facing `CrudOperationError`. Network failures
and successful-body deserialization failures are represented separately from server statuses.

### Current 422 incompatibility

The server and client do not currently share one 422 body type. Generated routes return either `{ "error": ... }` or
`{ "error": ..., "violations": ... }`, while `error_response_to_request_error` attempts to deserialize every 422
response as `ErrorInfo { errors: ... }`.

With the generated server and current web client, a 422 body can therefore surface as `RequestError::Deserialize` and
then `CrudOperationError::NetworkError` instead of `UnprocessableEntity`. Until both sides use one tagged error
contract, the higher-level variant mapping is not an end-to-end guarantee.
