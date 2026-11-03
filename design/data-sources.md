# Data Sources

An instance reads and changes its resource through a data source. The data source is the frontend's only integration
point for data: CrudKit's REST routes are one data source, and any other API, e.g. a typed client of an application's
own HTTP API, is another. The trait and its types live in `crudkit_web::data_source`. How the instance uses them
belongs to [Instances and Composition](instances-and-composition.md).

## Contract

`CrudDataSource` is an object-safe async trait. Every operation takes the instance's `CrudScope` and a request, and
answers with an outcome of its own:

| Operation     | Request                 | Outcome                | Default                      |
|---------------|-------------------------|------------------------|------------------------------|
| `list`        | `CrudListRequest`       | `CrudPage`             |                              |
| `count`       | `CrudCountRequest`      | `Option<u64>`          | `None`, the count is unknown |
| `read`        | `CrudReadRequest`       | `Option<DynReadModel>` |                              |
| `create`      | `CrudCreateRequest`     | `CrudSaved`            |                              |
| `update`      | `CrudUpdateRequest`     | `CrudSaved`            |                              |
| `delete`      | `CrudDeleteRequest`     | `CrudDeleted`          |                              |
| `delete_many` | `CrudDeleteManyRequest` | `CrudDeletedMany`      | one deletion after the other |

The data source also names its resource (`resource_name`, the origin of an instance's notifications) and declares the
fields it can order lists by (`can_order_by`).

The requests and outcomes are `#[non_exhaustive]` and built with their constructors, e.g. `CrudSaved::new(entity)`.
They are the frontend's own types, not the backend's `Saved` or `Deleted`, so that they can gain what other APIs need
without breaking data sources:

- `CrudUpdateRequest` holds the ID, the changed entity, and the `baseline` that the user's change started from: the
  version that was loaded or saved last. A data source that sends only what changed, e.g. as a JSON merge patch,
  compares the entity with it.
- `CrudDeleteRequest` and `CrudDeleteManyRequest` hold the entities as the user last saw them, not only their IDs.
- A version that names what a change or a deletion is based on, such as an `ETag` for `If-Match`, belongs to the
  models, so that it reaches the data source with the baseline and with the deleted entities.
- `CrudSaved` and `CrudDeleted` describe the outcome, so that an outcome such as an accepted change, whose resource
  is still changing, becomes another field instead of another trait.

A data source works on erased models, because the instance is non-generic after configuration. An adapter of another
API converts between that API's types and the resource's concrete models and wraps them, e.g. with
`DynReadModel::from`. `read` returns `None` when no entity has the ID within the scope, so an API answering
404 Not Found maps to `None`.

Every failure is a `RequestError`. A data source that cannot honor a part of a request, such as an ordering it does
not support, refuses the request with `RequestError::InvalidRequest` instead of ignoring that part, so that a
misconfigured instance fails visibly instead of showing or changing the wrong entities.

The futures of the operations need not be `Send` (`#[async_trait(?Send)]`, re-exported as
`crudkit_web::data_source::async_trait`). Instances load and save in the browser, through `LocalResource` and local
actions, which run on one thread. The trait object itself is `Send + Sync`, so the instance's configuration can hold it
in an `Arc`. Loading on the server during rendering would need `Send` futures; see the [backlog](../TODO.md).

## Mass Deletion

The default `delete_many` deletes one entity after the other with `delete`, and reports each entity in
`CrudDeletedMany`:

- `deleted` holds the IDs of the entities whose deletion affected an entity;
- `failed` holds the other entities with their typed `RequestError`;
- `skipped` holds the entities that were not tried.

A failure that concerns only its entity, such as `Forbidden`, `NotFound`, `UnprocessableEntity`,
`CriticalValidationErrors`, `BadRequest`, or `InvalidRequest`, lets the deletion go on. Any other failure, such as a
lost connection, `Unauthorized`, or a server error, would fail the remaining deletions too, so it stops the deletion,
and the remaining entities are skipped. A data source that can delete many entities at once overrides `delete_many`.
Notifications count `Forbidden` and `UnprocessableEntity` as aborted, `CriticalValidationErrors` as failed validation,
other failures as errors, and the skipped entities separately (`crudkit_web::delete::DeleteManyOutcome`).

## Scope

Every operation receives the instance's `CrudScope`, the parents of its entities, outermost first. Each parent is a
`SerializableIdEntry`: the name of the field through which the resource's entities refer to the parent, e.g.
`space_id`, and the parent's ID. The scope holds no condition. A condition is a part of the REST data source, so a data
source of another API never meets one.

A data source decides how it applies a scope. One serving nested paths, such as
`/projects/{project_id}/spaces/{space_id}/services`, takes the parents from the scope (`CrudScope::parent`) and puts
them into the path, so a nested resource lives below its parent's path instead of in a condition. The REST data source
turns them into conditions; see [HTTP and Data Providers](http-and-data-providers.md#rest-data-source).

The instance builds its scope from its configured scope, a signal, e.g. of parents from the application's route, and
adds the parent of a `CrudParentConfig`; see
[Instances and Composition](instances-and-composition.md#parent-and-nested-resources). A scope is convenience and
composition, not access control. The server authorizes every request.

## Pages

A `CrudListRequest` names the page both ways: by its `offset` and, once the list knows it, by the `cursor` that opens
it. It also carries the page's `limit` and the ordering. A data source uses whichever way it pages by.

A `CrudPage` holds the entities and its `CrudPaging`, which says how the page relates to the rest of the list:

- `Offset`: the data source opens every page by its offset. The list's total comes from `count`, which the instance
  requests again only when the scope changes or the list reloads, not for every page. The list offers every page, as
  described in [Leptos Hooks and Atoms](leptos-hooks-and-atoms.md#atoms). While the count is unknown, e.g. because it
  failed, the list still shows its page, but offers no page after it.
- `Cursor { next }`: the data source opens a page by the cursor that the page before it ended with. `next` opens the
  next page and is `None` on the last one. The list knows no total, so it offers only the previous and the next page.

`CrudCursor` is opaque: CrudKit stores it and hands it back, and never reads it. The instance keeps the cursors of the
pages it reached in a `crudkit_web::list::PageCursors`. Cursors depend on the ordering, the page size, and the scope,
so the list starts over at its first page when one of them changes. Every restart begins a new generation of the
cursors, and a page records the cursor of its next page only in the generation it was requested in, so a late answer
for an old ordering never overwrites a cursor of the new one.

## Ordering

`can_order_by(field)` declares the read-model fields by which `list` can order. A table offers sorting only for columns
whose header options allow it and whose field the data source can order by. The instance leaves configured orderings
by other fields out, and ignores requests to order by them, so a `CrudListRequest` only names fields the data source
declared.
