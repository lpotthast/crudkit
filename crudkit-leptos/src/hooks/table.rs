//! Interaction hook connecting a CrudKit list to Leptonic's table hooks.
//!
//! The table renders the rows of a [`CrudListState`] as an ARIA grid: keyboard navigation, focus
//! management, column header sorting, and row selection come from Leptonic. Ordering and selection
//! stay owned by CrudKit, because sorting happens on the server and the instance keeps its ordering
//! across views. Changes made through the table are written into CrudKit's state, and changes of
//! CrudKit's state are mirrored into the table.

// TODO: Could be simplified using Leptonic's `Table` atoms, which now accept bound sort and
// selection state. CrudKit's atoms still add DOM-safe keys and instance-specific row behavior.

use crate::config::Header;
use crate::hooks::field::{CrudRowFields, use_crud_row_fields};
use crate::hooks::list::CrudListState;
use crudkit_core::Order;
use crudkit_web::prelude::*;
use leptonic::hooks::Key;
use leptonic::hooks::collections::{CollectionOptions, Selection, SelectionOptions};
use leptonic::hooks::{
    DisabledBehavior, GridFocusMode, KeyboardNavigationBehavior, SelectionMode, SortDescriptor,
    SortDirection, TableCollection, TableData, TableOptions, UseFocusVisibleInput, UseGridProps,
    UseTableInput, UseTableReturn, UseTableStateInput, use_focus_visible, use_table,
    use_table_state,
};
use leptonic::utils::{CapturedElement, ValueBinding};
use leptos::prelude::*;
use std::fmt::Write as _;
use std::sync::Arc;

/// Key of the column holding row actions.
pub(crate) const ACTIONS_COLUMN: &str = "crudkit-actions";

/// Returns the table rendered by the surrounding [`CrudTable`](crate::atoms::CrudTable).
///
/// # Panics
///
/// Panics when called outside of a `CrudTable`.
#[must_use]
pub fn use_crud_table_data() -> CrudTableData {
    expect_crud_table_data("use_crud_table_data")
}

/// Returns the table surrounding `atom`.
///
/// # Panics
///
/// Panics when `atom` is rendered outside of a `CrudTable`.
pub(crate) fn expect_crud_table_data(atom: &'static str) -> CrudTableData {
    use_context::<CrudTableData>()
        .unwrap_or_else(|| panic!("`{atom}` must be rendered inside a `CrudTable`"))
}

/// The row of the surrounding [`CrudTableRows`](crate::atoms::CrudTableRows) iteration. Provided
/// as an `Option`, so that an instance can hide it from its descendants.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RowEntity {
    pub(crate) key: StoredValue<Key>,
    /// The row's entity, following reloads.
    pub(crate) entity: Memo<DynReadModel>,
    /// The entity's fields, shared by the row's cells.
    pub(crate) fields: CrudRowFields,
}

impl RowEntity {
    /// Creates the row of `key` in `table`, which shows `entity` until the table's rows change.
    pub(crate) fn new(table: &CrudTableData, key: Key, entity: DynReadModel) -> Self {
        let rows = table.rows;
        let row_key = key.clone();
        // A removed row keeps its last entity until it is disposed.
        let entity = Memo::new(move |previous: Option<&DynReadModel>| {
            rows.with(|rows| {
                rows.iter()
                    .find(|(key, _)| *key == row_key)
                    .map(|(_, entity)| entity.clone())
            })
            .or_else(|| previous.cloned())
            .unwrap_or_else(|| entity.clone())
        });
        Self {
            key: StoredValue::new(key),
            entity,
            fields: use_crud_row_fields(entity),
        }
    }
}

/// Returns the row of the surrounding [`CrudTableRows`](crate::atoms::CrudTableRows) iteration,
/// for `atom`.
///
/// # Panics
///
/// Panics outside of a row.
pub(crate) fn use_row_entity(atom: &'static str) -> RowEntity {
    use_context::<Option<RowEntity>>()
        .flatten()
        .unwrap_or_else(|| panic!("`{atom}` must be rendered inside a row of `CrudTableRows`"))
}

/// Returns the entity of the surrounding [`CrudTableRow`](crate::atoms::CrudTableRow): the
/// instance's read model. It changes when a reload brings a changed version of the entity.
///
/// # Panics
///
/// Panics when called outside of a row of [`CrudTableRows`](crate::atoms::CrudTableRows).
#[must_use]
pub fn use_crud_table_row() -> Signal<DynReadModel> {
    use_row_entity("use_crud_table_row").entity.into()
}

/// Input of [`use_crud_table`].
#[derive(Debug, Clone)]
pub struct UseCrudTableInput {
    /// The list whose rows are shown.
    pub list: CrudListState,
    /// The data columns.
    pub columns: Signal<Vec<Header>>,
    /// Whether rows can be selected through a leading checkbox column.
    pub selectable: Signal<bool>,
    /// Whether a trailing column holds row actions.
    pub with_actions: Signal<bool>,
    /// Accessible name of the table.
    pub aria_label: MaybeProp<String>,
    /// Ids of elements naming the table, e.g. a heading.
    pub aria_labelledby: Option<String>,
    /// Called when a row is activated, e.g. by pressing it while nothing is selected.
    pub on_row_action: Option<Callback<DynReadModel>>,
}

/// Output of [`use_crud_table`].
pub struct UseCrudTableReturn {
    /// Attributes for the `<table>` element. Spread them with `{..props.into_attrs()}`.
    pub props: UseGridProps,
    /// State shared by the table's rows, cells, and headers.
    pub table: CrudTableData,
}

/// State of one CrudKit table, shared by its parts.
#[derive(Debug, Clone)]
pub struct CrudTableData {
    /// Leptonic's table data, required by the table part hooks.
    pub data: TableData,
    /// The table's columns and rows.
    pub collection: Memo<Arc<TableCollection>>,
    /// The displayed entities with their row keys, in display order.
    pub rows: Memo<Vec<(Key, DynReadModel)>>,
    /// The data columns.
    pub columns: Signal<Vec<Header>>,
    /// Whether focus rings should be visible, i.e. the user navigates with the keyboard.
    pub focus_visible: Signal<bool>,
}

/// Returns the table key of a column showing `field`.
#[must_use]
pub(crate) fn column_key(field: &DynReadField) -> Key {
    Key::from(dom_safe(&field.name()))
}

/// Returns the table key of the row showing `entity`: the values of its ID, each encoded and joined
/// by `-`. The ID fields are the same for all entities of a resource, so their values alone tell the
/// entities apart, and encoding escapes `-` within a value.
#[must_use]
pub(crate) fn row_key(entity: &DynReadModel) -> Key {
    let values = entity
        .id()
        .entries()
        .map(|entry| dom_safe(&entry.value.to_string()))
        .collect::<Vec<_>>();
    Key::from(values.join("-"))
}

/// Encodes `value` injectively into `[A-Za-z0-9_]`, leaving `-` free as a separator.
///
/// Leptonic derives element IDs and selectors from collection keys, so keys must not contain quotes,
/// brackets, or similar characters.
pub(crate) fn dom_safe(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            encoded.push(character);
        } else {
            // Writing to a `String` cannot fail.
            let _ = write!(encoded, "_{:x}_", u32::from(character));
        }
    }
    encoded
}

/// Connects `input.list` to Leptonic's table hooks.
#[must_use]
pub fn use_crud_table(input: UseCrudTableInput) -> UseCrudTableReturn {
    let UseCrudTableInput {
        list,
        columns,
        selectable,
        with_actions,
        aria_label,
        aria_labelledby,
        on_row_action,
    } = input;

    let rows = table_rows(&list);
    let collection = table_collection(columns, rows, selectable, with_actions);

    let entities_of = move |selection: &Selection| {
        rows.with_untracked(|rows| match selection {
            Selection::All => rows
                .iter()
                .map(|(_, entity)| entity.clone())
                .collect::<Vec<_>>(),
            Selection::Keys(keys) => rows
                .iter()
                .filter(|(key, _)| keys.contains(key))
                .map(|(_, entity)| entity.clone())
                .collect(),
        })
    };

    // The table shows and edits CrudKit's own ordering and selection through bindings, so the
    // instance stays their single source of truth.
    let sort_descriptor = sort_binding(&list, columns);
    let selection = ValueBinding::new(
        Signal::derive(move || Selection::keys(list.selection.selected().iter().map(row_key))),
        Callback::new(move |selection: Selection| list.selection.replace(entities_of(&selection))),
    );

    let state = use_table_state(UseTableStateInput {
        table: collection,
        selection: SelectionOptions {
            selection_mode: Signal::derive(move || {
                if selectable.get() {
                    SelectionMode::Multiple
                } else {
                    SelectionMode::None
                }
            }),
            selection: Some(selection),
            disabled_behavior: DisabledBehavior::Selection,
            ..SelectionOptions::default()
        },
        focus_mode: GridFocusMode::Row,
        default_sort_descriptor: None,
        sort_descriptor: Some(sort_descriptor),
        on_sort_change: None,
    });

    let entity_of_key = move |key: &Key| {
        rows.with_untracked(|rows| {
            rows.iter()
                .find(|(row, _)| row == key)
                .map(|(_, entity)| entity.clone())
        })
    };

    let UseTableReturn { props, data } = use_table(UseTableInput {
        state,
        element: CapturedElement::new(),
        id: None,
        aria_label,
        aria_labelledby,
        keyboard_delegate: None,
        options: CollectionOptions::default(),
        keyboard_navigation_behavior: KeyboardNavigationBehavior::default(),
        should_select_on_press_up: false,
        on_row_action: on_row_action.map(|on_row_action| {
            Callback::new(move |key: Key| {
                if let Some(entity) = entity_of_key(&key) {
                    on_row_action.run(entity);
                }
            })
        }),
        on_cell_action: None,
    });

    UseCrudTableReturn {
        props,
        table: CrudTableData {
            data,
            collection,
            rows,
            columns,
            focus_visible: use_focus_visible(UseFocusVisibleInput::default())
                .focus_should_be_visible,
        },
    }
}

/// Pairs the loaded entities of `list` with their row keys.
fn table_rows(list: &CrudListState) -> Memo<Vec<(Key, DynReadModel)>> {
    let list = *list;
    Memo::new(move |_| {
        list.rows
            .read()
            .loaded()
            .map(|rows| {
                rows.iter()
                    .map(|row| (row_key(row), row.clone()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    })
}

/// Binds the table's sort descriptor to the primary ordering of `list`.
fn sort_binding(
    list: &CrudListState,
    columns: Signal<Vec<Header>>,
) -> ValueBinding<Option<SortDescriptor>> {
    let list = *list;
    let field_of_column = move |column: &Key| {
        columns
            .read_untracked()
            .iter()
            .find(|header| column_key(&header.field) == *column)
            .map(|header| header.field.clone())
    };

    ValueBinding::new(
        Signal::derive(move || primary_sort(&list)),
        Callback::new(move |descriptor: Option<SortDescriptor>| match descriptor {
            Some(descriptor) => {
                if let Some(field) = field_of_column(&descriptor.column) {
                    let order = match descriptor.direction {
                        SortDirection::Ascending => Order::Asc,
                        SortDirection::Descending => Order::Desc,
                    };
                    list.ordering.set(field, order);
                }
            }
            None => list.ordering.clear(),
        }),
    )
}

/// Builds the table's columns and rows.
fn table_collection(
    columns: Signal<Vec<Header>>,
    rows: Memo<Vec<(Key, DynReadModel)>>,
    selectable: Signal<bool>,
    with_actions: Signal<bool>,
) -> Memo<Arc<TableCollection>> {
    Memo::new(move |_| {
        let columns = columns.get();
        let rows = rows.get();
        let with_actions = with_actions.get();
        Arc::new(TableCollection::build_with(
            TableOptions {
                show_selection_checkboxes: selectable.get(),
            },
            |t| {
                for Header { field, options } in &columns {
                    let column = t.column(column_key(field), options.display_name.to_string());
                    if options.ordering_allowed {
                        column.allows_sorting();
                    }
                }
                if with_actions {
                    t.column(ACTIONS_COLUMN, "");
                }
                for (key, _) in &rows {
                    t.row(key.clone(), "", |r| {
                        for _ in &columns {
                            r.cell("");
                        }
                        if with_actions {
                            r.cell("");
                        }
                    });
                }
            },
        ))
    })
}

/// Returns the table's sort descriptor for the primary ordering of `list`.
fn primary_sort(list: &CrudListState) -> Option<SortDescriptor> {
    list.ordering.order_by().with(|order_by| {
        order_by.first().map(|(field, order)| SortDescriptor {
            column: column_key(field),
            direction: match order {
                Order::Asc => SortDirection::Ascending,
                Order::Desc => SortDirection::Descending,
            },
        })
    })
}

#[cfg(test)]
mod tests {
    use super::dom_safe;
    use assertr::prelude::*;

    #[test]
    fn dom_safe_keys_contain_only_safe_characters_and_stay_distinct() {
        let key = dom_safe(r#"[["id",{"I64":-1}]]"#);
        assert_that!(key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')).is_true();
        assert_that!(dom_safe("a b")).is_not_equal_to(dom_safe("a_b"));
        assert_that!(dom_safe("created_at")).is_equal_to("created_5f_at".to_owned());
    }
}
