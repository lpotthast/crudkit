# Data Sources

An instance reads and changes its resource through a data source. The data source is the frontend's only integration
point for data: CrudKit's REST routes are one data source, and any other API, e.g. a typed client of an application's
own HTTP API, is another. The trait and its types live in `crudkit_web::data_source`. How the instance uses them
belongs to [Instances and Composition](instances-and-composition.md).

## Contract

`CrudDataSource` is an object-safe async trait with these operations:

| Operation     | Input                         | Result                              |
|---------------|-------------------------------|-------------------------------------|
| `list`        | scope, `CrudListRequest`      | `CrudPage`                          |
| `read`        | scope, ID                     | `Option<DynReadModel>`              |
| `create`      | scope, `DynCreateModel`       | `Saved<DynUpdateModel>`             |
| `update`      | scope, ID, `DynUpdateModel`   | `Saved<DynUpdateModel>`             |
| `delete`      | scope, ID                     | `Deleted`                           |
| `delete_many` | scope, IDs                    | `DeletedMany`                       |

It also names its resource (`resource_name`, the origin of an instance's notifications) and declares the fields it
can order lists by (`can_order_by`).

A data source works on erased models, because the instance is non-generic after configuration. An adapter of another
API converts between that API's types and the resource's concrete models and wraps them, e.g. with
`DynReadModel::from`. `read` returns `None` when no entity has the ID within the scope, so an API answering
404 Not Found maps to `None`.

`delete_many` has a default that deletes one entity after the other and reports each outcome in `DeletedMany`:
`Forbidden` and `UnprocessableEntity` count as aborted, `CriticalValidationErrors` as failed validation, and every
other failure as an error. A data source that can delete many entities at once overrides it.

Every failure is a `RequestError`. A data source that cannot honor a part of a request, such as a condition or an
ordering it does not support, refuses the request with `RequestError::InvalidRequest` instead of ignoring that part,
so that a misconfigured instance fails visibly instead of showing or changing the wrong entities.

The futures of the operations need not be `Send` (`#[async_trait(?Send)]`, re-exported as
`crudkit_web::data_source::async_trait`). Instances load and save in the browser, through `LocalResource` and local
actions, which run on one thread. The trait object itself is `Send + Sync`, so the instance's configuration can hold it
in an `Arc`. Loading on the server during rendering would need `Send` futures; see the [backlog](../TODO.md).

## Scope

Every operation receives the instance's `CrudScope`. A scope holds:

- the parents of the entities, outermost first. Each parent is a `SerializableIdEntry`: the name of the field through
  which the resource's entities refer to the parent, e.g. `space_id`, and the parent's ID;
- a condition that narrows every operation except creation, the instance's base condition.

A data source decides how it applies a scope. One serving nested paths, such as
`/projects/{project_id}/spaces/{space_id}/services`, takes the parents from the scope (`CrudScope::parent`) and puts
them into the path, so a nested resource lives below its parent's path instead of in a condition. The REST data source
turns them into conditions; see [HTTP and Data Providers](http-and-data-providers.md#rest-data-source).

The instance builds its scope from its configured scope, e.g. parents from the application's route, and adds the
parent of a `CrudParentConfig`; see
[Instances and Composition](instances-and-composition.md#parent-and-nested-resources). A scope is convenience and
composition, not access control. The server authorizes every request.

## Pages

A `CrudListRequest` names the page both ways: by its `offset` and, once the list knows it, by the `cursor` that opens
it. It also carries the page's `limit` and the ordering. A data source uses whichever way it pages by.

A `CrudPage` holds the entities and its `CrudPaging`, which says how the page relates to the rest of the list:

- `Offset { total }`: the data source opens every page by its offset, and the list holds `total` entities. The list
  offers every page, as described in [Leptos Hooks and Atoms](leptos-hooks-and-atoms.md#atoms).
- `Cursor { next }`: the data source opens a page by the cursor that the page before it ended with. `next` opens the
  next page and is `None` on the last one. The list knows no total, so it offers only the previous and the next page.

`CrudCursor` is opaque: CrudKit stores it and hands it back, and never reads it. The instance keeps the cursors of the
pages it reached in a `crudkit_web::list::PageCursors`. Cursors depend on the ordering, the page size, and the scope,
so the list starts over at its first page when its ordering or page size changes.

## Ordering

`can_order_by(field)` declares the read-model fields by which `list` can order. A table offers sorting only for columns
whose header options allow it and whose field the data source can order by. The instance leaves configured orderings
by other fields out, and ignores requests to order by them, so a `CrudListRequest` only names fields the data source
declared.
