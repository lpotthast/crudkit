# Leptos Hooks and Atoms

`crudkit-leptos` separates CrudKit's frontend behavior from markup, following the layering of Leptonic's hooks branch.
CrudKit ships no prebuilt screens and no theme: applications build every CRUD screen from atoms, in their own markup
and styles. The crate documentation (`crudkit-leptos/docs/guide.md`, included by `lib.rs`) is the guide for
application authors and contains everything needed to build a UI; its examples compile as doctests. This document
records the design behind it.

## Layers

| Layer             | Module                     | Contents                                                            |
|-------------------|----------------------------|---------------------------------------------------------------------|
| Pure logic        | `crudkit-web`              | Navigation state machine, list arithmetic, selection, formatting    |
| Configuration     | `crudkit_leptos::config`   | What applications declare: configuration, actions, renderers, texts |
| Instance runtime  | `crudkit_leptos::instance` | Context, manager, navigation, `CrudInstance`, `CrudViewOutlet`      |
| State hooks       | `crudkit_leptos::hooks`    | Reactive domain state of an instance or view, without markup        |
| Interaction hooks | `crudkit_leptos::hooks`    | State hooks bound to Leptonic's input and table hooks               |
| Atoms             | `crudkit_leptos::atoms`    | Components binding that state to one element each, or to none       |

Configuration is plain data and contains no markup apart from the renderers an application registers itself
(`FieldRenderer`, action views, `CrudViewRegistry` entries). Behavior lives in hooks: everything an atom does is
reachable through a public hook, so applications can write markup the atoms do not cover.

`crudkit-leptos` depends on Leptonic's `hooks` branch with its `hooks` and `atoms` features. It never uses Leptonic's
themed components or theme. Leptonic supplies accessible behavior: press handling, keyboard navigation, focus
management, ARIA semantics, form field association, overlays, and toasts. Builds need `--cfg=web_sys_unstable_apis`,
which Leptonic requires.

## Atom Rules

CrudKit's atoms follow Leptonic's atom guardrails:

- An atom renders exactly one element. Providers (`CrudInstance`, `CrudList`, `CrudField`), outlets (`CrudViewOutlet`),
  and iterators (`CrudTableRows`, `CrudTableCells`, `CrudFormLayout`, ...) render none. The exceptions are Leptonic's
  hidden inputs, and action buttons, which render their action's view (an overlay) after their button, because the
  view belongs to the button that requested it.
- An atom's element carries a default class, `crudkit-` plus the atom's name without its `Crud` prefix
  (`CrudSaveButton` renders `crudkit-SaveButton`). An atom built on a Leptonic atom keeps that atom's default class too.
  The caller's `classes` add to the default class (merged with leptos-classes), and `styles` follow the element's own
  inline styles. State is exposed as `data-*` attributes, absent when false. Every atom documents its default class
  and data attributes.
- Props follow Leptonic's conventions: `is_*` flags as `Signal<bool>`, user-visible text as `MaybeProp<String>`,
  callbacks as `Option<Callback<..>>`. Contexts are provided with `<Provider>`.
- CrudKit binds its state to Leptonic's atoms instead of duplicating them: field atoms wrap `TextField`, `NumberField`,
  `Switch`, and `Checkbox`; buttons wrap `Button`; confirmation dialogs wrap `ModalBackdrop`; the page size control
  wraps `Select`; notifications use Leptonic's toast queue and atoms.
- Every piece of markup is a component, also the crate-private building blocks atoms share (`ColumnHeader`,
  `GridCell`, the layout elements).
- An atom rendered outside the container it needs (a list, table, row, form, field, or dialog), or in one that does not
  fit (a selection cell in a table that is not selectable, a field of another model than its form's), panics with a
  message naming the atom, like Leptonic's atoms: that is a mistake in the application's markup. Configuration
  problems that depend on data, such as a field the model does not have, are logged and shown instead.
- Defaults keep applications short: atoms showing text show `CrudUiTexts` unless given children, and iterators render
  their one obvious atom (a `CrudTableCell` per column, a `CrudPageButton` per page). Defaults are compositions of the
  same public atoms, so an application can take over at any level.

## Contexts

Atoms find what they act on in context rather than in props:

- The instance (`CrudInstanceContext`), provided by `CrudInstance`. Inside a rendered view, its navigation is the
  view's, see [Views, Navigation Scopes, and Dirty Guards](views-and-navigation.md).
- The list (`CrudListState`), provided by `CrudList` and read with `use_crud_list`.
- The table, provided by `CrudTable`, and the row, provided per row by `CrudTableRows`: the row's entity (a memo that
  follows reloads, read with `use_crud_table_row`) and its fields, shared by the row's cells. Rows are keyed by entity
  id, so a reload that changes an entity updates its row in place.
- The form: `CrudCreateForm`, `CrudEditForm`, and `CrudDetails` provide a typed form context for their fields and a
  `CrudFormHandle` (kind, status, dirty, error, and saving state, save, default follow-up, violations, draft) read with
  `use_crud_form`.
- The entity: table rows and entity forms provide the `CrudEntityHandle` entity-related buttons act on (its id,
  deleting it), read with `use_crud_entity`. A `CrudDeleteButton` therefore deletes the entity of the row or form it
  is in.
- The field: `CrudField` provides the typed `CrudFieldState` and the type-erased `CrudFieldBinding` that field atoms and
  input hooks bind to.

The row, form, and entity contexts are provided as `Option`s. `CrudInstance` (and `provide_crud_instance`) resets them
to `None`, so the atoms of an instance nested in a row or form act on the nested instance only.

`CrudField<T>` is generic over the bound field enum. `IntoDynField`, implemented by the `CkField` derive, names the
field's type-erased type (`DynCreateField`, `DynReadField`, or `DynUpdateField`), so `<CrudField field=Club::Name>`
finds the typed form context of its model by type. The form handle records its field type, so a field of another model
is reported instead of bound to the wrong form.

## State Hooks

| Hook                          | Returns                                              | State owner               |
|-------------------------------|------------------------------------------------------|---------------------------|
| `use_crud_instance`           | `CrudInstanceContext` with the view's navigation     | instance                  |
| `use_crud_navigation`         | `CrudNavigation` of the current view                 | instance                  |
| `use_crud_list_state`         | `CrudListState`: rows, count, paging, ordering       | instance and calling list |
| `use_crud_list`               | The `CrudListState` of the surrounding `CrudList`    | `CrudList`                |
| `use_crud_create_form`        | `UseCrudCreateFormReturn`: form, save, saving state  | calling view              |
| `use_crud_edit_form`          | `UseCrudEditFormReturn`: entity, form, save, delete  | calling view              |
| `use_crud_read`               | `UseCrudReadReturn`: entity, status, values, delete  | calling view              |
| `use_crud_form`               | The `CrudFormHandle` of the surrounding form atom    | form atom                 |
| `use_crud_entity`             | The `CrudEntityHandle` of the row or form, if any    | row or form atom          |
| `use_crud_table_row`          | The entity of the surrounding row, following reloads | `CrudTableRows`           |
| `use_crud_field`              | `CrudFieldState` of the field being rendered         | `CrudField`, renderer     |
| `use_crud_field_binding`      | `CrudFieldBinding` of the field, any model           | `CrudField`, renderer     |
| `use_crud_delete`             | `CrudDeleteState`: pending deletions of the view     | instance                  |
| `use_crud_leave_confirmation` | `CrudLeaveConfirmation` hosted by the instance       | instance                  |
| `use_crud_actions_state`      | `CrudActionsState`: requested and executing actions  | calling view              |
| `use_crud_notifier`           | `CrudNotifier` of the application, else logging      | application               |

Conventions:

- A hook taking configuration accepts one `UseCrudXInput` value and returns one `UseCrudXReturn` value. Inputs have no
  constructors: callers write struct literals, using struct update syntax only for inputs implementing `Default`. A
  hook with a single required argument and no options, such as `use_crud_read(id)`, takes the argument directly.
- Returned state handles that only refer to arena-owned signals are `Copy`, e.g. `CrudListState` and
  `CrudFormHandle`, so copies share state, as described for `CrudInstanceContext` in
  [Instances and Composition](instances-and-composition.md). Handles expose state as read-only `Signal` fields and
  changes as methods. Returns holding owned data, such as field state or input bindings, are `Clone`.
- Hooks creating state (`use_crud_list_state`, the form hooks, `use_crud_actions_state`) create it in the calling owner.
  Hooks named after a context (`use_crud_list`, `use_crud_form`, `use_crud_field`) return the surrounding one.
- Hooks that require an instance or a context panic outside of it. Calling them there is a programming error.
- State hooks render nothing. Labels and messages they emit, such as notification texts, are data.

## Interaction Hooks

- `use_crud_text_input`, `use_crud_number_input`, and `use_crud_toggle_input` bind the field being rendered (its
  `CrudFieldBinding`) to Leptonic's `TextField`, `NumberField<T>`, and `Checkbox`/`Switch` state. They are not generic
  over the field type. A `CrudTextCodec` converts between values and text. Optional fields map empty input to
  `Value::Null`; input the codec rejects becomes an input error of the form and leaves the value unchanged. Text that
  does not parse yet is kept while the user types.
- `use_crud_table` renders a `CrudListState` through Leptonic's table hooks as an ARIA grid. Ordering and selection
  stay owned by CrudKit and are bound into Leptonic's table state. A header press replaces the ordering with that
  column; secondary ordering is available through `CrudOrderingState::toggle` with `append`.
- `use_crud_tab_selection` binds a layout's tab group to Leptonic's `Tabs`. The instance remembers the selected tab of
  every tab group, so the selection survives re-rendering a layout and carries over between views whose layouts use
  the same tab IDs.

Leptonic derives element IDs and selectors from collection keys, so CrudKit encodes field names, tab IDs, and each
value of a row's ID injectively into `[A-Za-z0-9_]`; a row's key joins its ID's values with `-`.

`format_crud_value` formats a value as display text; `CrudValue`, table cells, and fields in display mode use it.

## Atoms

| Area          | Atoms                                                                                         |
|---------------|-----------------------------------------------------------------------------------------------|
| Instance      | `CrudInstance`, `CrudViewOutlet` (in `instance`), `CrudInstanceMgr`, `CrudResetButton`        |
| List          | `CrudList`, `CrudListStatus`, `CrudCreateButton`, `CrudDeleteSelectedButton`,                 |
|               | `CrudClearSelectionButton`, `CrudSelectionCount`                                              |
| Table         | `CrudTable`, `CrudTableHeader`, `CrudTableHeaderRow`, `CrudTableColumnHeaders`,               |
|               | `CrudTableColumnHeader`, `CrudTableSelectAllHeader`, `CrudSelectAllCheckbox`,                 |
|               | `CrudTableActionsHeader`, `CrudTableBody`, `CrudTableRows`, `CrudTableRow`,                   |
|               | `CrudTableSelectionCell`, `CrudRowCheckbox`, `CrudTableCells`, `CrudTableCell`,               |
|               | `CrudTableActionsCell`                                                                        |
| Pagination    | `CrudPagination`, `CrudPageButtons`, `CrudPageButton`, `CrudPageGap`,                         |
|               | `CrudPreviousPageButton`, `CrudNextPageButton`, `CrudItemsPerPage`, `CrudItemsPerPageOptions` |
| Entities      | `CrudReadButton`, `CrudEditButton`, `CrudDeleteButton`                                        |
| Forms         | `CrudCreateForm`, `CrudEditForm`, `CrudDetails`, `CrudEntityStatus`, `CrudSaveButton`,        |
|               | `CrudReturnButton`                                                                            |
| Layout        | `CrudFormLayout`, `CrudFormGroup`, `CrudFormCard`, `CrudFormTabs`, `CrudFormTab`,             |
|               | `CrudFormTabPanel`, `CrudFormSeparator`                                                       |
| Fields        | `CrudField`, `CrudFieldControl`, `CrudTextField`, `CrudNumberField`, `CrudSwitch`,            |
|               | `CrudCheckbox`, `CrudDisplayField`, `CrudFieldLabel`, `CrudFieldError`, `CrudFieldValue`      |
| Confirmations | `CrudDeleteDialog`, `CrudDeleteManyDialog`, `CrudLeaveDialog`, `CrudConfirmationTitle`,       |
|               | `CrudConfirmationMessage`, `CrudConfirmButton`, `CrudCancelButton`                            |
| Actions       | `CrudResourceActions`, `CrudEntityActions`, `CrudActionButton`, `CrudEntityActionButton`      |
| Values        | `CrudValue`, `CrudIcon`                                                                       |

`CrudTable` takes `selectable` and `with_actions` explicitly, because they add columns to the grid whose header and
cells the application renders; `row_action` (`CrudRowAction`) decides what activating a row does. Column headers and
cells expose the column's kind of value as `data-kind`, like `CrudValue`, so number columns can be aligned. Like
Leptonic's table atoms, rows, column headers, and cells expose `data-focus-visible` and `data-hovered`, and a cell
renders again when its column moves, e.g. when the table's columns change.

`CrudListStatus` and `CrudEntityStatus` are polite live regions that stay rendered, empty while there is nothing to
explain: screen readers announce changes of a live region's content, but rarely the content of a new one.

`CrudFieldControl` renders the `FieldRenderer` registered for the field in context, else a default composition for the
field's input kind, each replaceable through a prop. `CrudFormLayout` renders the configured layout of the surrounding
form from the layout atoms and a `CrudField` per field. Number fields are generic over the field's primitive type, so
64-bit and 128-bit integers stay exact. They do not group digits by default, because IDs and years are the most common
numbers in CRUD forms.

Confirmation dialogs open while their confirmation is pending; closing them (Escape, a press outside) cancels, except
while the confirmed deletion runs, when the confirm button is pending. Their content mounts once per opening. The
instance counts its mounted dialogs and reports a confirmation becoming pending without a dialog to ask it, which
would otherwise wait forever. An action button renders its action's view (an overlay) itself and owns its requested
and executing state; the completion it hands to the action stays valid after the view is gone.

User-facing texts come from `CrudUiTexts`, provided with `provide_crud_texts` as a signal, so that a change of language
updates rendered atoms. Atoms read the texts where they render them; hooks read them when they notify. The defaults
are German; `CrudUiTexts::english` provides English texts.

## Views

A `CrudViewRegistry` maps view names to renderers. It is empty by default. `table`, `create`, `read`, and `edit`
register the four standard views and check that a view is opened with the subject it needs (an entity id for `read` and
`edit`, none otherwise) and without a payload; `register` and `replace` add application-defined views, which receive
the opened `CrudView`. Renderers take no navigation argument: views read it from context. `CrudViewOutlet` renders
the current view of its instance, or of another `CrudNavigation` it is given, each view in its own reactive owner and
navigation scope. A view without a renderer, or opened with the wrong subject, renders a visible error.

## Forms

`CrudFormState` holds the draft model, the baseline it is compared against, input errors, and one `ReactiveField` per
field. `set_field` updates the draft and the field's reactive value together. Create forms start from the create
model's default, with a nested instance's parent reference filled in. Edit and read forms reload whenever the entity
loads; the edit and read hooks combine loading and field values into a `CrudEntityLoadStatus` (loading, ready, not
found, or failed). After a successful update, the saved entity becomes the baseline, so the draft is clean again. A
list's `CrudListLoadStatus` (loading, ready, empty, or failed) likewise drives `CrudListStatus`.

A form does not save while a field holds input its codec rejects, and its fields are read-only while it saves. The
save request reports its outcome through the instance's notifier, also when the form is gone by then: a success or an
error notification, as the form's `CrudSaveNotifications` select (all by default, failures only, or none).
`on_saved` and `on_save_failed` run while the form is mounted.

`CrudSaveFollowUp` selects what happens after a successful save: `Stay` (a create form starts a fresh draft), `Return`,
`Edit` (the saved entity, which stays after editing), `CreateAnother` (a fresh draft in a create form), or a view
resolved from the saved entity's id. The form atoms default to `Edit` after creating and `Stay` after editing;
submitting a form saves it with that default. A `CrudSaveButton` without a follow-up of its own is the form's submit
button, so Enter in a field saves the form. Follow-up navigation is committed, as described in
[Views, Navigation Scopes, and Dirty Guards](views-and-navigation.md).

HTML forms cannot nest. A form atom inside another one, e.g. in a field showing a nested instance, renders a
`<div role="group">` that provides Leptonic's form contexts itself and saves on Enter in one of its inputs, and its
save button is a plain button. An edit form keeps its unsaved changes when its entity reloads.

Create and edit forms guard the current navigation with their dirty state. Read forms do not register a guard. The form
atoms wrap Leptonic's `Form` with `ValidationBehavior::Aria`, because CrudKit validates as the user edits.

## Notifications

CrudKit describes user feedback as `CrudNotification { kind, title, message, origin }` and hands it to the nearest
`CrudNotifier`. Notifications go to one application-wide sink: the application provides a notifier once at its root
with `provide_crud_notifier`, typically `CrudNotifier::toasts` of a Leptonic `ToastQueue<CrudNotification>` rendered
with Leptonic's toast atoms. Successes and information close after five seconds; warnings and errors stay until closed.
Without a provided notifier, notifications are logged as warnings.

An instance captures the nearest notifier when it mounts and names itself as the `origin` (`CrudNotificationOrigin`:
instance id, instance name, and resource name) of everything emitted through it: deletions, saves, and action
outcomes. Notifications emitted outside of an instance have no origin, and an origin set by the emitter is kept. A
notifier provided further down overrides the root one for its subtree. Requests completing after their instance
unmounted still notify, through the application's notifier, without an origin.

## Testing

State hooks are unit-tested on a native single-threaded executor against an in-memory server. The crate-private
`test_support` module provides this setup: `with_crud_manager` runs a test below a manager, `CrudTestServer` answers
CrudKit's requests from a handler and records them, and `settle` runs pending tasks. The test dev-dependencies enable
`reactive_graph`'s `effects` feature, because Leptos only runs effects in client builds.

The atoms' markup (elements, default classes, data attributes, misplacement panics) is tested by rendering atoms to
HTML, which needs the `ssr` feature: `cargo test -p crudkit-leptos --features ssr --lib atoms::tests`, run by
`just test`.

The crate documentation's examples are compiled as doctests against the `clubs` and `players` resources defined by
the hidden `__doc_example!` macro, so the guide cannot drift from the API.

The full-stack example in `examples/full-stack` is the end-to-end target. Its UI is a design system of its own
(`src/ui/`, styled by `style/ui.scss`), built from atoms, Leptonic's atoms, and hooks. It styles atoms both through
classes it passes and through their default classes and data attributes.
