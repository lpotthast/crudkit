# Leptos Hooks, Atoms, and Components

`crudkit-leptos` separates CrudKit's frontend behavior from its markup, following the layering of Leptonic's hooks
branch. Applications choose how much of CrudKit's UI they use, per screen or per field:

1. mount `CrudInstance` and use the built-in views;
2. recompose a view from CrudKit's components and atoms and register it in the instance's `CrudViewRegistry`;
3. write their own markup on top of CrudKit's hooks.

All three depths share one implementation of loading, paging, ordering, selection, drafts, dirty guards, saving,
deletion, and action bookkeeping. Built-in views use the same hooks that applications use.

## Layers

| Layer | Module | Contents |
|---|---|---|
| Pure logic | `crudkit-web` | Navigation state machine, list arithmetic, selection, deletion, formatting. |
| Configuration | `crudkit_leptos::config` | What applications declare: configuration, actions, renderers, texts. |
| Instance runtime | `crudkit_leptos::instance` | Context, manager, navigation, and markup-free providers and outlets. |
| State hooks | `crudkit_leptos::hooks` | Reactive domain state of an instance or view. No markup. |
| Interaction hooks | `crudkit_leptos::hooks` | State hooks bound to Leptonic's input and table hooks. |
| Atoms | `crudkit_leptos::atoms` | Headless single-element components where CrudKit adds behavior. |
| Components | `crudkit_leptos::components` | The built-in, composed UI with `crudkit-*` classes. |

Configuration is plain data and contains no markup. Its extension points are values that the components layer fills
in: `CrudViewRegistry::default()` registers the built-in views, and `FieldRenderer::default_for` and the other
built-in field renderers render CrudKit's markup. The styled `CrudInstance` is a component; `CrudInstanceProvider`
and `CrudViewOutlet` are its markup-free runtime counterparts.

`crudkit-leptos` depends on Leptonic's `hooks` branch with only its `atoms` feature. It never uses Leptonic's themed
components, so CrudKit's markup and styling stay independent of Leptonic's theme. Leptonic supplies accessible
behavior: press handling, keyboard navigation, focus management, ARIA semantics, and form field association.
Applications do not need Leptonic's `Root`. Builds need `--cfg=web_sys_unstable_apis`, which Leptonic requires.

## State Hooks

| Hook                          | Returns                                                                      | State owner               |
|-------------------------------|------------------------------------------------------------------------------|---------------------------|
| `use_crud_instance`           | `CrudInstanceContext` with the current view's navigation                     | instance                  |
| `use_crud_list`               | `CrudListState`: rows, count, pagination, ordering, selection                | instance and calling list |
| `use_crud_create_form`        | `UseCrudCreateFormReturn`: form, save, saving state                          | calling view              |
| `use_crud_edit_form`          | `UseCrudEditFormReturn`: loaded entity, status, form, save, delete           | calling view              |
| `use_crud_read`               | `UseCrudReadReturn`: loaded entity, status, field values, delete             | calling view              |
| `use_crud_field`              | `CrudFieldState` of the field being rendered                                 | field renderer            |
| `use_crud_delete`             | `CrudDeleteState`: pending deletions, bound to the calling view's navigation | instance                  |
| `use_crud_leave_confirmation` | `CrudLeaveConfirmation` hosted by the instance                               | instance                  |
| `use_crud_actions`            | `CrudActionsState`: available, requested, and executing actions              | calling view              |
| `use_crud_notifier`           | `CrudNotifier` provided by the application or the manager                    | application               |

Conventions:

- A hook taking configuration accepts one `UseCrudXInput` value and returns one `UseCrudXReturn` value. Inputs with a
  required value provide `UseCrudXInput::new(..)` and are completed with struct update syntax. A hook with a single
  required argument and no options, such as `use_crud_read(id)`, takes the argument directly.
- Returned handles are `Copy`. They refer to arena-owned signals, so copies share state, as described for
  `CrudInstanceContext` in [Instances and Composition](instances-and-composition.md). Handles expose state as
  read-only `Signal` fields and changes as methods; accessors that need an argument, such as
  `CrudFormState::field`, are methods too.
- Hooks creating state (`use_crud_list`, the form hooks, `use_crud_actions`) create it in the calling owner. A view
  calls them once and shares the handle, e.g. through context. Hooks returning instance state return the existing
  handle.
- Hooks that require an instance panic outside of one. Calling them there is a programming error.
- State hooks render nothing and never assume a particular markup. Labels and messages they emit, such as notification
  texts, are data.

## Interaction Hooks

Interaction hooks combine a state hook with Leptonic's hooks or atom state types:

- `use_crud_text_input`, `use_crud_number_input`, and `use_crud_toggle_input` bind the field being rendered to
  Leptonic's `TextField`, `NumberField<T>`, and `Checkbox`/`Switch` state. A `CrudTextCodec` converts between
  values and text. Optional fields map empty input to `Value::Null`; input the codec rejects becomes an input error
  of the form and leaves the value unchanged. Text that does not parse yet is kept while the user types.
- `use_crud_table` renders a `CrudListState` through Leptonic's table hooks as an ARIA grid. Ordering and selection
  stay owned by CrudKit and are bound into Leptonic's table state: the table shows CrudKit's primary ordering and
  current selection, and a header press or selection change writes CrudKit's state. A header press replaces the
  ordering with that column; secondary ordering is available through `CrudOrderingState::toggle` with `append`.
- `use_crud_tab_selection` binds a layout's tab group to Leptonic's `Tabs` (`selected_key`). The instance remembers
  the selected tab of every tab group, so the selection survives re-rendering a layout and carries over between
  views whose layouts use the same tab IDs. Without a remembered selection, the first tab is shown.

Leptonic derives element IDs and selectors from collection keys, so CrudKit encodes row IDs and field names
injectively into `[A-Za-z0-9_-]`. Applications name columns with the typed `CrudTableColumn` and rows by their entity.

The table is built from hooks rather than Leptonic's `Table` atom. Since the atom gained bindings for sort and
selection state, it could replace part of this code (see the backlog). Number fields are generic over the field's
primitive type, so 64-bit and 128-bit integers stay exact. They do not group digits, because IDs and years are the most
common numbers in CRUD forms.

## Atoms and Components

Atoms (`CrudTable`, `CrudTableHeader`, `CrudTableBody`, `CrudTableRow`, `CrudTableCell`, `CrudIcon`) render one
element each, take `classes`, use no CSS classes of their own, and expose state as `data-*` attributes;
`CrudTableRows` repeats `CrudTableRow` for the loaded entities. Elsewhere, components use Leptonic's atoms directly;
icon-only and page buttons, for example, are Leptonic's `Button` with `aria_label` and `aria_current`.

Components render CrudKit's DOM contract: `crudkit-*` classes and `data-*` attributes, presented by the optional
theme described in [Theming and Generated Assets](theming-and-generated-assets.md). Components with one root element
take `classes`, which are added to their `crudkit-*` class. Built-in views are compositions of public components
such as `CrudTableToolbar`, `CrudSelectionBar`, `CrudPagination`, `CrudFormLayout`, `CrudResourceActionButtons`,
`CrudEntityActionButtons`, and `CrudConfirmDialog`. The view components (`CrudTableView`, `CrudCreateView`,
`CrudEditView`, `CrudReadView`) take their elements, renderer registries, controls, and navigation from the
surrounding instance unless given explicitly, so recomposed views use them without extra wiring. Navigation and
controls given to a view apply to its whole subtree: the hooks it calls (dirty guards, save follow-ups, deletion),
the components it renders, and instances nested in it. `atoms::prelude` and `components::prelude` export the atoms
and components; the crate prelude includes both.

Confirmation dialogs are modal alert dialogs on Leptonic's `ModalBackdrop`, `ModalContent`, and `Dialog` atoms. They
contain focus, and Escape cancels. Their content mounts once per opening and only updates its texts, so focus is not
disturbed while a dialog closes.

User-facing texts come from `CrudUiTexts`, provided with `provide_crud_texts`. The defaults are German.

## Forms

`CrudFormState` holds the draft model, the baseline it is compared against, input errors, and one `ReactiveField` per
field. `set_field` updates the draft and the field's reactive value together. Create forms start from the create
model's default, with a nested instance's parent reference filled in. Edit and read forms reload whenever the entity
loads; `is_ready` reports when field values exist, and the edit and read hooks combine both into a `CrudEntityStatus`
(loading, ready, not found, or failed). After a successful update, the saved entity becomes the baseline,
so the draft is clean again.

A form does not save while a field holds input its codec rejects (`has_errors`). After creating with `Stay`, the
create form starts over from a fresh default model. A failed save sends an error notification through the instance's
notifier unless the form input sets `quiet`; `on_save_failed` runs either way.

`CrudSaveFollowUp` selects what happens after a successful save. `Default` applies the instance's
`create_save_target` after creating and stays after editing. Follow-up navigation is committed, as described in
[Views, Navigation Scopes, and Dirty Guards](views-and-navigation.md).

Create and edit forms guard the current navigation with their dirty state. A read form does not register a guard.

## Custom UIs

An application can render an instance without any of CrudKit's markup. `CrudInstanceProvider` (or
`provide_crud_instance` inside an application's own provider component) creates the instance
context and `CrudViewOutlet` renders the current view through the registry, so the application supplies the
surrounding element, its own views, and its own dialogs for `use_crud_delete` and `use_crud_leave_confirmation`.
Custom views read the configuration through `CrudInstanceContext` accessors such as `list_columns`,
`create_elements`, and the field renderer registries. State changes go through the hooks: the context exposes list
state only as read-only signals, and deletion and leave confirmation through `use_crud_delete` and
`use_crud_leave_confirmation`.

`CrudFormField` makes one field of a `CrudFormState` the current field without rendering markup, so application inputs
built on the input hooks, or a registered `FieldRenderer` called through `render`, bind to it. `use_crud_row_fields`
is its read-only counterpart for listed entities: it provides values and display-mode field states, so custom tables
can use registered renderers too.

Custom tables combine `CrudTable`, `CrudTableHeader`, `CrudTableBody`, `CrudTableRows` (one `CrudTableRow` per
loaded entity, re-created when the entity changes), and `CrudTableCell`. Cells read their row from context and are
therefore created inside the row's children; `use_crud_table_row` returns the row's entity. Leptonic's `Tab` and
`TabPanel` likewise belong inside `TabList` and `Tabs`.

## Notifications

CrudKit describes user feedback as `CrudNotification { kind, title, message }` and hands it to the nearest
`CrudNotifier`. Applications route notifications into their own notification system with `provide_crud_notifier`
above their `CrudInstanceMgr`. `CrudNotificationQueue` keeps the shown notifications, removes each after a configurable
time, and feeds itself through its `notifier`; applications render its `entries` in their own markup. Otherwise the
manager provides a queue and renders it as `CrudNotificationRegion`, a polite live region. Outside of a manager,
notifications are logged.

## Testing

State hooks are unit-tested on a native single-threaded executor against an in-memory server. The crate-private
`test_support` module provides this setup: `with_crud_manager` runs a test below a manager, `CrudTestServer` answers
CrudKit's requests from a handler and records them, and `settle` runs pending tasks. The test dev-dependencies enable
`reactive_graph`'s `effects` feature, because Leptos only runs effects in client builds.

The full-stack example in `examples/full-stack` is the end-to-end target. It renders each resource twice: with the
built-in UI and with a custom design system built on CrudKit's hooks and atoms.