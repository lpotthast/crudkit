//! Table atoms rendering a CrudKit list as an accessible grid.
//!
//! [`CrudTable`] wires the list of the surrounding view into Leptonic's table hooks (see
//! [`crate::hooks::table::use_crud_table`]). [`CrudTableHeader`], [`CrudTableRow`], and
//! [`CrudTableCell`] each render one element and expose their state as `data-*` attributes.
//! [`CrudTableRows`] renders one row per loaded entity.

use crate::config::Header;
use crate::hooks::list::CrudListState;
use crate::hooks::table::{
    CrudTableColumn, UseCrudTableInput, UseCrudTableReturn, row_key, use_crud_table,
    use_crud_table_data,
};
use crate::hooks::texts::use_crud_texts;
use crudkit_web::prelude::*;
use leptonic::hooks::{
    CellFocusMode, ColumnKind, IntoAttrs, Key, NodeKind, SelectionMode, SortDirection,
    UseTableCellInput, UseTableCellReturn, UseTableColumnHeaderInput, UseTableColumnHeaderReturn,
    UseTableRowInput, UseTableRowReturn, use_checkbox, use_grid_row_group, use_table_cell,
    use_table_column_header, use_table_header_placeholder, use_table_header_row, use_table_row,
    use_table_select_all_checkbox, use_table_selection_checkbox,
};
use leptonic::utils::classes::Classes;
use leptos::context::Provider;
use leptos::prelude::*;
use std::sync::Arc;

fn flag(signal: Signal<bool>) -> impl Fn() -> Option<&'static str> + Send + Sync + 'static {
    move || signal.get().then_some("true")
}

/// A `<table>` showing the rows of `list`.
#[component]
pub fn CrudTable(
    /// The shown list. Defaults to the [`CrudListState`] in context.
    #[prop(optional)]
    list: Option<CrudListState>,
    /// The data columns.
    #[prop(into)]
    columns: Signal<Vec<Header>>,
    /// Whether rows can be selected through a leading checkbox column.
    #[prop(into)]
    selectable: Signal<bool>,
    /// Whether a trailing column holds row actions.
    #[prop(into)]
    with_actions: Signal<bool>,
    /// Accessible name of the table.
    #[prop(into, optional)]
    aria_label: Option<String>,
    /// Called when a row is activated.
    #[prop(into, optional)]
    on_row_action: Option<Callback<DynReadModel>>,
    /// Classes of the `<table>` element.
    #[prop(into, optional)]
    classes: Classes,
    children: Children,
) -> impl IntoView {
    let list = list.unwrap_or_else(|| {
        use_context::<CrudListState>()
            .expect("`CrudTable` needs a `list` prop or a `CrudListState` in context")
    });
    let UseCrudTableReturn { props, table } = use_crud_table(UseCrudTableInput {
        list,
        columns,
        selectable,
        with_actions,
        aria_label,
        on_row_action,
    });
    view! {
        <Provider value=table>
            <table {..props.into_attrs()} class=classes>
                {children()}
            </table>
        </Provider>
    }
}

/// The `<thead>` of a [`CrudTable`] with all column headers.
///
/// Column headers expose `data-allows-sorting`, `data-sort-direction` (`ascending` /
/// `descending`), `data-focused`, and `data-pressed`. The selection column header holds a
/// "select all" checkbox.
#[component]
pub fn CrudTableHeader(
    /// Accessible name of the select-all checkbox. Defaults to [`crate::config::CrudUiTexts::select_all`].
    #[prop(into, optional)]
    select_all_label: Option<String>,
    /// Classes of the `<thead>` element.
    #[prop(into, optional)]
    classes: Classes,
) -> impl IntoView {
    let select_all_label =
        select_all_label.unwrap_or_else(|| use_crud_texts().select_all.to_string());
    let table = use_crud_table_data();
    let collection = table.collection;
    let row_group = use_grid_row_group();
    view! {
        <thead {..row_group.row_group_props.into_attrs()} class=classes>
            <For
                each=move || collection.with(|t| t.header_rows().to_vec())
                key=Clone::clone
                children=move |row: Key| {
                    let cells = move || {
                        collection
                            .with(|t| {
                                t.collection()
                                    .children(&row)
                                    .map(|n| (n.key.clone(), n.kind))
                                    .collect::<Vec<_>>()
                            })
                    };
                    let select_all_label = select_all_label.clone();
                    view! {
                        <tr {..use_table_header_row().into_attrs()}>
                            <For
                                each=cells
                                key=|(key, _)| key.clone()
                                children=move |(key, kind): (Key, NodeKind)| {
                                    if kind == NodeKind::Placeholder {
                                        let data = use_crud_table_data().data;
                                        view! {
                                            <th {..use_table_header_placeholder(&data, &key)
                                                .into_attrs()}></th>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <ColumnHeader
                                                key
                                                select_all_label=select_all_label.clone()
                                            />
                                        }
                                            .into_any()
                                    }
                                }
                            />
                        </tr>
                    }
                }
            />
        </thead>
    }
}

#[component]
fn ColumnHeader(key: Key, select_all_label: String) -> impl IntoView {
    let table = use_crud_table_data();
    let data = table.data.clone();
    let state = data.state;
    let (text, kind, allows_sorting) = table.collection.with_untracked(|t| {
        t.column(&key)
            .map_or((Arc::from(""), ColumnKind::Data, false), |c| {
                (c.text_value.clone(), c.kind, c.allows_sorting)
            })
    });
    let sort_key = key.clone();
    let sort_direction = move || {
        state.sort_descriptor.with(|sort| match sort {
            Some(sort) if sort.column == sort_key => Some(match sort.direction {
                SortDirection::Ascending => "ascending",
                SortDirection::Descending => "descending",
            }),
            _ => None,
        })
    };
    let selection = state.grid.list.selection;
    let focus_key = key.clone();
    let is_focused =
        move || (selection.is_focused() && selection.is_focused_key(&focus_key)).then_some("true");
    let content = if kind == ColumnKind::SelectionCheckbox {
        (selection.selection_mode() == SelectionMode::Multiple)
            .then(|| {
                let mut input = use_table_select_all_checkbox(&data);
                // Leptonic labels the checkbox in English until its localized strings land. Overriding
                // the hook input is the supported way to use CrudKit's texts meanwhile.
                input.options.aria_label = select_all_label.into();
                let checkbox = use_checkbox(input);
                let (attrs, styles) = checkbox.input_props.into_parts();
                view! { <input {..attrs} style=styles /> }
            })
            .into_any()
    } else {
        text.to_string().into_any()
    };
    let UseTableColumnHeaderReturn {
        column_header_props,
        is_pressed,
    } = use_table_column_header(UseTableColumnHeaderInput {
        table: data,
        key,
        allows_arrow_navigation: false,
    });
    let (attrs, styles) = column_header_props.into_parts();
    view! {
        <th
            {..attrs}
            style=styles
            data-allows-sorting=allows_sorting.then_some("true")
            data-sort-direction=sort_direction
            data-focused=is_focused
            data-pressed=flag(is_pressed)
        >
            {content}
        </th>
    }
}

/// The `<tbody>` of a [`CrudTable`].
#[component]
pub fn CrudTableBody(
    /// Classes of the `<tbody>` element.
    #[prop(into, optional)]
    classes: Classes,
    children: Children,
) -> impl IntoView {
    let row_group = use_grid_row_group();
    view! {
        <tbody {..row_group.row_group_props.into_attrs()} class=classes>
            {children()}
        </tbody>
    }
}

#[derive(Debug, Clone)]
struct RowContext {
    key: Key,
    entity: DynReadModel,
}

/// Returns the entity of the surrounding [`CrudTableRow`].
///
/// # Panics
///
/// Panics when called outside of a `CrudTableRow`.
#[must_use]
pub fn use_crud_table_row() -> DynReadModel {
    use_context::<RowContext>()
        .expect("`use_crud_table_row` must be called inside a `CrudTableRow`")
        .entity
}

/// One [`CrudTableRow`] per loaded entity of the table, with `children` rendering its cells.
///
/// A row is re-created when its entity changes, e.g. after a reload.
#[component]
pub fn CrudTableRows<C, IV>(
    /// Renders the cells of a row, e.g. with one [`CrudTableCell`] per column.
    children: C,
    /// Accessible name of each row's selection checkbox. Defaults to [`crate::config::CrudUiTexts::select_row`].
    #[prop(into, optional)]
    select_label: Option<String>,
    /// Classes of each row.
    #[prop(into, optional)]
    row_classes: Classes,
) -> impl IntoView
where
    C: Fn(DynReadModel) -> IV + Send + Sync + 'static,
    IV: IntoView + 'static,
{
    let table = use_crud_table_data();
    let select_label =
        StoredValue::new(select_label.unwrap_or_else(|| use_crud_texts().select_row.to_string()));
    let children = Arc::new(children);
    view! {
        <For
            each=move || table.rows.get()
            // Rows also key on their content, so that a reload re-renders exactly the changed rows.
            key=|(key, entity)| (key.clone(), format!("{entity:?}"))
            children=move |(_, entity): (Key, DynReadModel)| {
                let children = children.clone();
                let cells_entity = entity.clone();
                view! {
                    <CrudTableRow
                        entity
                        select_label=select_label.get_value()
                        classes=row_classes.clone()
                    >
                        // Cells are created inside the row, which provides their row context.
                        {children(cells_entity)}
                    </CrudTableRow>
                }
            }
        />
    }
}

/// A `<tr>` of a [`CrudTable`] showing `entity`.
///
/// With a selection column, it renders the row's selection cell itself. Add one
/// [`CrudTableCell`] per remaining column, created inside the row's children. Descendants can read
/// the entity with [`use_crud_table_row`]. Exposes `data-selected`, `data-focused`,
/// `data-disabled`, and `data-pressed`.
#[component]
pub fn CrudTableRow(
    /// The entity shown by the row.
    entity: DynReadModel,
    /// Accessible name of the row's selection checkbox. Defaults to [`crate::config::CrudUiTexts::select_row`].
    #[prop(into, optional)]
    select_label: Option<String>,
    /// Classes of the `<tr>` element.
    #[prop(into, optional)]
    classes: Classes,
    children: Children,
) -> impl IntoView {
    let select_label = select_label.unwrap_or_else(|| use_crud_texts().select_row.to_string());
    let key = row_key(&entity);
    let table = use_crud_table_data();
    let selection_column = table.collection.with_untracked(|t| {
        t.column_at(0)
            .filter(|c| c.kind == ColumnKind::SelectionCheckbox)
            .map(|c| c.key.clone())
    });
    let UseTableRowReturn {
        row_props,
        is_selected,
        is_focused,
        is_disabled,
        is_pressed,
        ..
    } = use_table_row(UseTableRowInput {
        table: table.data,
        key: key.clone(),
        on_context_menu: None,
    });
    let (attrs, styles) = row_props.into_parts();
    view! {
        <Provider value=RowContext { key, entity }>
            <tr
                {..attrs}
                class=classes
                style=styles
                data-selected=flag(is_selected)
                data-focused=flag(is_focused)
                data-disabled=flag(is_disabled)
                data-pressed=flag(is_pressed)
            >
                {selection_column
                    .map(|column| view! { <SelectionCell column label=select_label /> })}
                {children()}
            </tr>
        </Provider>
    }
}

#[component]
fn SelectionCell(column: Key, label: String) -> impl IntoView {
    let data = use_crud_table_data().data;
    let row = expect_context::<RowContext>().key;
    let mut input = use_table_selection_checkbox(&data, row.clone());
    input.options.aria_label = label.into();
    let checkbox = use_checkbox(input);
    let (checkbox_attrs, checkbox_styles) = checkbox.input_props.into_parts();
    let input = view! { <input {..checkbox_attrs} style=checkbox_styles /> };
    grid_cell(
        &column,
        None,
        Classes::default(),
        Some(Box::new(move || input.into_any())),
    )
}

/// A `<td>` of a [`CrudTableRow`] in `column`. Exposes `data-pressed`.
///
/// Cells read their row from context, so they must be created inside the row's children, e.g. by
/// a component rendered as a child of [`CrudTableRow`], not before the row.
#[component]
// Component props are passed by value.
#[allow(clippy::needless_pass_by_value)]
pub fn CrudTableCell(
    /// The cell's column, e.g. a field or [`CrudTableColumn::Actions`].
    #[prop(into)]
    column: CrudTableColumn,
    /// What gets focus: the cell, or its first focusable child.
    #[prop(optional)]
    focus_mode: Option<CellFocusMode>,
    /// Classes of the `<td>` element.
    #[prop(into, optional)]
    classes: Classes,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    grid_cell(&column.key(), focus_mode, classes, children)
}

/// Renders the grid cell of the surrounding row in the column `column`.
fn grid_cell(
    column: &Key,
    focus_mode: Option<CellFocusMode>,
    classes: Classes,
    children: Option<Children>,
) -> AnyView {
    let table = use_crud_table_data();
    let row = expect_context::<RowContext>().key;
    let Some(index) = table
        .collection
        .with_untracked(|t| t.column(column).map(|c| c.index))
    else {
        tracing::error!(
            ?column,
            "a table cell references a column the table does not have"
        );
        return view! { <td class=classes>{children.map(|children| children())}</td> }.into_any();
    };
    let UseTableCellReturn {
        grid_cell_props,
        is_pressed,
    } = use_table_cell(UseTableCellInput {
        table: table.data,
        key: Key::cell(&row, index),
        focus_mode,
        allows_arrow_navigation: false,
        should_select_on_press_up: false,
    });
    let (attrs, styles) = grid_cell_props.into_parts();
    view! {
        <td {..attrs} class=classes style=styles data-pressed=flag(is_pressed)>
            {children.map(|children| children())}
        </td>
    }
    .into_any()
}
