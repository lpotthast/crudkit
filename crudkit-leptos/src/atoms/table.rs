//! The surrounding list as an accessible grid.
//!
//! [`CrudTable`] wires the list into Leptonic's table hooks: arrow keys move between cells, column
//! headers sort, rows are selected through their checkboxes, and activating a row runs the
//! table's [`CrudRowAction`]. Every other atom renders one element of the table and must be placed
//! where that element belongs, e.g. a [`CrudTableCell`] inside a [`CrudTableRow`].

use crate::atoms::content::content_or_default;
use crate::atoms::value::{CrudValue, data_kind};
use crate::config::{CrudUiTexts, Header};
use crate::hooks::delete::use_crud_delete;
use crate::hooks::entity::CrudEntityHandle;
use crate::hooks::instance::{use_crud_instance, use_crud_navigation};
use crate::hooks::list::expect_crud_list;
use crate::hooks::table::{
    ACTIONS_COLUMN, CrudTableData, RowEntity, UseCrudTableInput, UseCrudTableReturn, column_key,
    expect_crud_table_data, use_crud_table, use_row_entity,
};
use crate::hooks::texts::use_crud_texts;
use crudkit_web::field::FieldOptions;
use crudkit_web::prelude::*;
use crudkit_web::view::CrudView;
use leptonic::hooks::{
    CellFocusMode, ColumnKind, IntoAttrs, Key, SortDirection, UseFocusRingInput,
    UseFocusRingReturn, UseHoverInput, UseTableCellInput, UseTableCellReturn,
    UseTableColumnHeaderInput, UseTableColumnHeaderReturn, UseTableRowInput, UseTableRowReturn,
    UseTableSelectAllCheckboxInput, UseTableSelectionCheckboxInput, use_checkbox, use_focus_ring,
    use_grid_row_group, use_hover, use_table_cell, use_table_column_header, use_table_header_row,
    use_table_row, use_table_select_all_checkbox, use_table_selection_checkbox,
};
use leptonic::utils::classes::{Classes, MergeStrategy};
use leptonic::utils::data_attributes::flag;
use leptonic::utils::styles::Styles;
use leptos::context::Provider;
use leptos::prelude::*;
use std::fmt;
use std::sync::Arc;

/// What activating a row of a [`CrudTable`] does: pressing it, or pressing Enter on it.
#[derive(Clone, Copy, Default)]
pub enum CrudRowAction {
    /// Nothing.
    #[default]
    None,
    /// Opens the row's entity in the read view.
    Read,
    /// Opens the row's entity in the edit view.
    Edit,
    /// Calls the callback with the row's entity.
    Custom(Callback<DynReadModel>),
}

impl fmt::Debug for CrudRowAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::None => "None",
            Self::Read => "Read",
            Self::Edit => "Edit",
            Self::Custom(_) => "Custom",
        })
    }
}

/// The `<table>` showing the surrounding [`CrudList`](crate::atoms::CrudList): an accessible grid
/// of the instance's configured columns.
///
/// With `selectable`, a leading column selects rows: render a [`CrudTableSelectAllHeader`] and a
/// [`CrudTableSelectionCell`] per row. With `with_actions`, a trailing column holds row actions:
/// render a [`CrudTableActionsHeader`] and a [`CrudTableActionsCell`] per row.
///
/// Default class: `crudkit-Table`.
#[component]
pub fn CrudTable(
    /// The data columns. Defaults to the instance's [`list_columns`].
    ///
    /// [`list_columns`]: crate::config::CrudInstanceConfig::list_columns
    #[prop(into, optional)]
    columns: Option<Signal<Vec<Header>>>,
    /// Whether rows can be selected through a leading checkbox column.
    #[prop(optional)]
    selectable: bool,
    /// Whether a trailing column holds row actions.
    #[prop(optional)]
    with_actions: bool,
    /// What activating a row does.
    #[prop(optional)]
    row_action: CrudRowAction,
    /// Accessible name of the table. Defaults to the instance's resource name, unless
    /// `aria_labelledby` names the table.
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    /// Ids of elements naming the table, e.g. a heading.
    #[prop(into, optional)]
    aria_labelledby: Option<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-Table")
        .merge(classes, MergeStrategy::UnionConditions);
    let ctx = use_crud_instance();
    let navigation = use_crud_navigation();
    let on_row_action = match row_action {
        CrudRowAction::None => None,
        CrudRowAction::Read => Some(Callback::new(move |entity: DynReadModel| {
            navigation.navigate(CrudView::read(entity.id()));
        })),
        CrudRowAction::Edit => Some(Callback::new(move |entity: DynReadModel| {
            navigation.navigate(CrudView::edit(entity.id()));
        })),
        CrudRowAction::Custom(callback) => Some(callback),
    };
    let UseCrudTableReturn { props, table } = use_crud_table(UseCrudTableInput {
        list: expect_crud_list("CrudTable"),
        columns: columns.unwrap_or_else(|| ctx.list_columns()),
        selectable: Signal::stored(selectable),
        with_actions: Signal::stored(with_actions),
        aria_label: table_label(aria_label, aria_labelledby.is_some(), ctx.resource_name()),
        aria_labelledby,
        on_row_action,
    });
    view! {
        <Provider value=table>
            <table {..props.into_attrs()} class=classes style=styles>
                {children()}
            </table>
        </Provider>
    }
}

/// The accessible name of a table: `aria_label`, else `resource_name` unless the table is named by
/// other elements.
fn table_label(
    aria_label: MaybeProp<String>,
    is_labelled_by: bool,
    resource_name: String,
) -> MaybeProp<String> {
    if is_labelled_by {
        aria_label
    } else {
        Signal::derive(move || Some(aria_label.get().unwrap_or_else(|| resource_name.clone())))
            .into()
    }
}

/// The `<thead>` of a [`CrudTable`], holding a [`CrudTableHeaderRow`].
///
/// Default class: `crudkit-TableHeader`.
#[component]
pub fn CrudTableHeader(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-TableHeader")
        .merge(classes, MergeStrategy::UnionConditions);
    let row_group = use_grid_row_group();
    view! {
        <thead {..row_group.row_group_props.into_attrs()} class=classes style=styles>
            {children()}
        </thead>
    }
}

/// The `<tr>` of column headers inside a [`CrudTableHeader`]: a [`CrudTableSelectAllHeader`] for a
/// selectable table, [`CrudTableColumnHeaders`], and a [`CrudTableActionsHeader`] for a table with
/// actions, in this order.
///
/// Default class: `crudkit-TableHeaderRow`.
#[component]
pub fn CrudTableHeaderRow(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-TableHeaderRow")
        .merge(classes, MergeStrategy::UnionConditions);
    view! {
        <tr {..use_table_header_row().into_attrs()} class=classes style=styles>
            {children()}
        </tr>
    }
}

/// A [`CrudTableColumnHeader`] per data column of the surrounding [`CrudTable`]. Renders no element
/// of its own.
#[component]
pub fn CrudTableColumnHeaders(
    /// Classes of each column header.
    #[prop(into, optional)]
    classes: Classes,
) -> impl IntoView {
    let columns = expect_crud_table_data("CrudTableColumnHeaders").columns;
    view! {
        <For
            each=move || columns.get()
            key=|header| header.field.clone()
            children=move |header: Header| {
                view! { <CrudTableColumnHeader column=header.field classes=classes.clone() /> }
            }
        />
    }
}

/// The `<th>` of the data column showing `column`. Pressing it sorts the list by the column, if
/// the column allows it, replacing the current ordering.
///
/// Default content: the column's configured display name.
///
/// Data attributes: `data-kind` (the column's kind of value, as [`CrudValue`]'s, e.g. to align
/// numbers), `data-allows-sorting`, `data-sort-direction` (`ascending` or `descending`),
/// `data-focused`, `data-focus-visible`, `data-hovered` (sortable columns), `data-pressed`.
///
/// Default class: `crudkit-TableColumnHeader`.
#[component]
pub fn CrudTableColumnHeader(
    /// The field shown in the column.
    #[prop(into)]
    column: DynReadField,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-TableColumnHeader")
        .merge(classes, MergeStrategy::UnionConditions);
    let key = column_key(&column);
    let kind = data_kind(column.value_kind());
    let collection = expect_crud_table_data("CrudTableColumnHeader").collection;
    let title_key = key.clone();
    let title = move || {
        collection.with(|table| {
            table
                .column(&title_key)
                .map(|column| column.text_value.to_string())
                .unwrap_or_default()
        })
    };
    view! {
        <ColumnHeader atom="CrudTableColumnHeader" key kind classes styles>
            {content_or_default(children, title)}
        </ColumnHeader>
    }
}

/// The `<th>` of the selection column of a selectable [`CrudTable`], holding a
/// [`CrudSelectAllCheckbox`].
///
/// Data attributes: `data-focused`, `data-focus-visible`, `data-pressed`.
///
/// Default class: `crudkit-TableSelectAllHeader`.
#[component]
pub fn CrudTableSelectAllHeader(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-TableSelectAllHeader")
        .merge(classes, MergeStrategy::UnionConditions);
    let key = selection_column(
        &expect_crud_table_data("CrudTableSelectAllHeader"),
        "CrudTableSelectAllHeader",
    );
    view! {
        <ColumnHeader atom="CrudTableSelectAllHeader" key classes styles>
            {children()}
        </ColumnHeader>
    }
}

/// The checkbox selecting all rows of a selectable [`CrudTable`]: a native `<input>` inside a
/// [`CrudTableSelectAllHeader`]. It is checked while all rows are selected and indeterminate while
/// some are.
///
/// Labeled [`CrudUiTexts::select_all`] unless given `aria_label`.
///
/// Default class: `crudkit-SelectAllCheckbox`.
///
/// [`CrudUiTexts::select_all`]: crate::config::CrudUiTexts::select_all
#[component]
pub fn CrudSelectAllCheckbox(
    #[prop(into, optional)] aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-SelectAllCheckbox")
        .merge(classes, MergeStrategy::UnionConditions);
    let table = expect_crud_table_data("CrudSelectAllCheckbox");
    selection_column(&table, "CrudSelectAllCheckbox");
    let mut input =
        use_table_select_all_checkbox(UseTableSelectAllCheckboxInput { table: table.data });
    input.options.aria_label = checkbox_label(aria_label, |texts| texts.select_all.to_string());
    let (attrs, checkbox_styles) = use_checkbox(input).input_props.into_parts();
    view! { <input {..attrs} class=classes style=checkbox_styles.merge(styles) /> }
}

/// The `<th>` of the actions column of a [`CrudTable`] with actions.
///
/// Default content: [`CrudUiTexts::actions`].
///
/// Data attributes: `data-focused`, `data-focus-visible`, `data-pressed`.
///
/// Default class: `crudkit-TableActionsHeader`.
///
/// [`CrudUiTexts::actions`]: crate::config::CrudUiTexts::actions
#[component]
pub fn CrudTableActionsHeader(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-TableActionsHeader")
        .merge(classes, MergeStrategy::UnionConditions);
    let texts = use_crud_texts();
    let text = move || texts.read().actions.to_string();
    view! {
        <ColumnHeader atom="CrudTableActionsHeader" key=Key::from(ACTIONS_COLUMN) classes styles>
            {content_or_default(children, text)}
        </ColumnHeader>
    }
}

/// The `<th>` of the column `key`, shared by all column header atoms (`atom`).
#[component]
fn ColumnHeader(
    atom: &'static str,
    key: Key,
    /// The column's kind of value, as `data-kind`.
    #[prop(optional_no_strip)]
    kind: Option<&'static str>,
    classes: Classes,
    styles: Styles,
    children: Children,
) -> impl IntoView {
    let table = expect_crud_table_data(atom);
    let state = table.data.state;
    let collection = table.collection;
    let column_key = key.clone();
    let allows_sorting = Memo::new(move |_| {
        collection.with(|collection| {
            collection
                .column(&column_key)
                .is_some_and(|column| column.allows_sorting && column.kind == ColumnKind::Data)
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
        Signal::derive(move || selection.is_focused() && selection.is_focused_key(&focus_key));
    let focus_visible = table.focus_visible;
    let is_focus_visible = Signal::derive(move || is_focused.get() && focus_visible.get());
    // Sortable headers show hover, as Leptonic's table atoms do.
    let hover = use_hover(UseHoverInput {
        is_disabled: Signal::derive(move || !allows_sorting.get()),
        ..UseHoverInput::default()
    });
    let UseTableColumnHeaderReturn {
        column_header_props,
        is_pressed,
    } = use_table_column_header(UseTableColumnHeaderInput {
        table: table.data,
        key,
        allows_arrow_navigation: false,
    });
    let (attrs, header_styles) = column_header_props.into_parts();
    view! {
        <th
            {..attrs}
            {..hover.props.into_attrs()}
            class=classes
            style=header_styles.merge(styles)
            data-kind=kind
            data-allows-sorting=flag(allows_sorting.into())
            data-sort-direction=sort_direction
            data-focused=flag(is_focused)
            data-focus-visible=flag(is_focus_visible)
            data-hovered=flag(hover.is_hovered)
            data-pressed=flag(is_pressed)
        >
            {children()}
        </th>
    }
}

/// The `<tbody>` of a [`CrudTable`], holding its [`CrudTableRows`].
///
/// Default class: `crudkit-TableBody`.
#[component]
pub fn CrudTableBody(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-TableBody")
        .merge(classes, MergeStrategy::UnionConditions);
    let row_group = use_grid_row_group();
    view! {
        <tbody {..row_group.row_group_props.into_attrs()} class=classes style=styles>
            {children()}
        </tbody>
    }
}

/// Renders its children once per loaded entity of the surrounding [`CrudTable`], typically a
/// [`CrudTableRow`]. Inside, [`use_crud_table_row`](crate::hooks::use_crud_table_row) returns the
/// entity. Renders no element of its own.
///
/// Rows are keyed by their entity's id: when a reload changes an entity, its row stays and
/// updates its values.
#[component]
pub fn CrudTableRows(children: ChildrenFn) -> impl IntoView {
    let table = expect_crud_table_data("CrudTableRows");
    view! {
        <For
            each=move || table.rows.get()
            key=|(key, _)| key.clone()
            children=move |(key, entity): (Key, DynReadModel)| {
                let children = children.clone();
                let row = RowEntity::new(&table, key, entity);
                view! { <Provider value=Some(row)>{children()}</Provider> }
            }
        />
    }
}

/// The `<tr>` of the entity of the surrounding [`CrudTableRows`] iteration. Its cells (a
/// [`CrudTableSelectionCell`], [`CrudTableCells`] or [`CrudTableCell`]s, a
/// [`CrudTableActionsCell`]) follow the table's columns. Buttons inside act on the row's entity,
/// e.g. [`CrudEditButton`](crate::atoms::CrudEditButton).
///
/// Data attributes: `data-selected`, `data-focused`, `data-focus-visible`, `data-hovered` (rows
/// that can be selected or activated), `data-disabled`, `data-pressed`.
///
/// Default class: `crudkit-TableRow`.
#[component]
pub fn CrudTableRow(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-TableRow")
        .merge(classes, MergeStrategy::UnionConditions);
    let RowEntity { key, entity, .. } = use_row_entity("CrudTableRow");
    let deletion = use_crud_delete();
    let handle = CrudEntityHandle {
        id: Signal::derive(move || Some(entity.read().id())),
        delete: Callback::new(move |()| deletion.request(entity.get_untracked())),
        can_delete: Signal::stored(true),
    };
    let table = expect_crud_table_data("CrudTableRow");
    let UseTableRowReturn {
        row_props,
        is_selected,
        is_focused,
        is_disabled,
        is_pressed,
        allows_selection,
        has_action,
    } = use_table_row(UseTableRowInput {
        table: table.data,
        key: key.get_value(),
        on_context_menu: None,
    });
    let (attrs, row_styles) = row_props.into_parts();
    let focus_visible = table.focus_visible;
    let is_focus_visible = Signal::derive(move || is_focused.get() && focus_visible.get());
    // Rows that can be selected or activated show hover, as Leptonic's table atoms do.
    let hover = use_hover(UseHoverInput {
        is_disabled: Signal::derive(move || !allows_selection.get() && !has_action.get()),
        ..UseHoverInput::default()
    });
    view! {
        <Provider value=Some(handle)>
            <tr
                {..attrs}
                {..hover.props.into_attrs()}
                class=classes
                style=row_styles.merge(styles)
                data-selected=flag(is_selected)
                data-focused=flag(is_focused)
                data-focus-visible=flag(is_focus_visible)
                data-hovered=flag(hover.is_hovered)
                data-disabled=flag(is_disabled)
                data-pressed=flag(is_pressed)
            >
                {children()}
            </tr>
        </Provider>
    }
}

/// The `<td>` of the selection column in a row of a selectable [`CrudTable`], holding a
/// [`CrudRowCheckbox`].
///
/// Data attributes: `data-pressed`, `data-focus-visible`, `data-hovered`.
///
/// Default class: `crudkit-TableSelectionCell`.
#[component]
pub fn CrudTableSelectionCell(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: ChildrenFn,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-TableSelectionCell")
        .merge(classes, MergeStrategy::UnionConditions);
    let column = selection_column(
        &expect_crud_table_data("CrudTableSelectionCell"),
        "CrudTableSelectionCell",
    );
    view! { <GridCell atom="CrudTableSelectionCell" column classes styles children /> }
}

/// The checkbox selecting the surrounding row: a native `<input>` inside a
/// [`CrudTableSelectionCell`].
///
/// Labeled [`CrudUiTexts::select_row`] unless given `aria_label`.
///
/// Default class: `crudkit-RowCheckbox`.
///
/// [`CrudUiTexts::select_row`]: crate::config::CrudUiTexts::select_row
#[component]
pub fn CrudRowCheckbox(
    #[prop(into, optional)] aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-RowCheckbox")
        .merge(classes, MergeStrategy::UnionConditions);
    let row = use_row_entity("CrudRowCheckbox").key.get_value();
    let mut input = use_table_selection_checkbox(UseTableSelectionCheckboxInput {
        table: expect_crud_table_data("CrudRowCheckbox").data,
        key: row,
    });
    input.options.aria_label = checkbox_label(aria_label, |texts| texts.select_row.to_string());
    let (attrs, checkbox_styles) = use_checkbox(input).input_props.into_parts();
    view! { <input {..attrs} class=classes style=checkbox_styles.merge(styles) /> }
}

/// A [`CrudTableCell`] per data column of the surrounding [`CrudTable`]. Renders no element of its
/// own.
#[component]
pub fn CrudTableCells(
    /// Classes of each cell.
    #[prop(into, optional)]
    classes: Classes,
) -> impl IntoView {
    let columns = expect_crud_table_data("CrudTableCells").columns;
    view! {
        <For
            each=move || columns.get()
            key=|header| header.field.clone()
            children=move |header: Header| {
                view! { <CrudTableCell column=header.field classes=classes.clone() /> }
            }
        />
    }
}

/// The `<td>` showing the field `column` of the surrounding row's entity.
///
/// Default content: the read renderer registered for the field in the instance configuration, else
/// the value as a [`CrudValue`], with date-times shown as the column's header configures.
///
/// Data attributes: `data-kind` (the column's kind of value, as [`CrudValue`]'s, e.g. to align
/// numbers), `data-pressed`, `data-focus-visible`, `data-hovered`.
///
/// Default class: `crudkit-TableCell`.
#[component]
pub fn CrudTableCell(
    /// The field shown in the cell.
    #[prop(into)]
    column: DynReadField,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<ChildrenFn>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-TableCell")
        .merge(classes, MergeStrategy::UnionConditions);
    let key = column_key(&column);
    let kind = data_kind(column.value_kind());
    let children = children.unwrap_or_else(|| {
        let field = StoredValue::new(column);
        Arc::new(move || view! { <CellContent field=field.get_value() /> }.into_any())
    });
    view! { <GridCell atom="CrudTableCell" column=key kind classes styles children /> }
}

/// The default content of a cell showing `field` of the surrounding row's entity: the read
/// renderer registered for the field, else its value as text. Follows the options of the
/// column's header.
#[component]
fn CellContent(field: DynReadField) -> impl IntoView {
    let ctx = use_crud_instance();
    let fields = use_row_entity("CrudTableCell").fields;
    let columns = expect_crud_table_data("CrudTableCell").columns;
    move || {
        let date_time_display = columns.with(|columns| {
            columns
                .iter()
                .find(|header| header.field == field)
                .map(|header| header.options.date_time_display)
                .unwrap_or_default()
        });
        let options = FieldOptions {
            date_time_display,
            ..FieldOptions::default()
        };
        let Some(state) = fields.display_state(&field, options) else {
            tracing::error!(field = %field.name(), "a table cell shows a field the read model does not have");
            return ().into_any();
        };
        if let Some(renderer) = ctx.read_field_renderers().get(&field) {
            renderer.render(state)
        } else {
            let value = state.value;
            view! { <CrudValue value=Signal::derive(move || value.get()) date_time_display /> }
                .into_any()
        }
    }
}

/// The `<td>` of the actions column in a row of a [`CrudTable`] with actions, holding buttons
/// acting on the row's entity, e.g. [`CrudEditButton`](crate::atoms::CrudEditButton) and
/// [`CrudDeleteButton`](crate::atoms::CrudDeleteButton). Arrow keys focus its first button.
///
/// Data attributes: `data-pressed`, `data-focus-visible`, `data-hovered`.
///
/// Default class: `crudkit-TableActionsCell`.
#[component]
pub fn CrudTableActionsCell(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: ChildrenFn,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-TableActionsCell")
        .merge(classes, MergeStrategy::UnionConditions);
    view! {
        <GridCell
            atom="CrudTableActionsCell"
            column=Key::from(ACTIONS_COLUMN)
            focus_mode=CellFocusMode::Child
            classes
            styles
            children
        />
    }
}

/// The accessible name of a selection checkbox: `aria_label`, else CrudKit's `text`. Leptonic
/// names the checkboxes in English until its localized strings land; overriding the hook input is
/// the supported way to use CrudKit's texts meanwhile.
fn checkbox_label(
    aria_label: MaybeProp<String>,
    text: fn(&CrudUiTexts) -> String,
) -> MaybeProp<String> {
    let texts = use_crud_texts();
    Signal::derive(move || Some(aria_label.get().unwrap_or_else(|| text(&texts.read())))).into()
}

/// Returns the key of the selection column of `table`, for `atom`.
///
/// # Panics
///
/// Panics when `table` is not selectable.
fn selection_column(table: &CrudTableData, atom: &'static str) -> Key {
    table
        .collection
        .with_untracked(|collection| {
            collection
                .column_at(0)
                .filter(|column| column.kind == ColumnKind::SelectionCheckbox)
                .map(|column| column.key.clone())
        })
        .unwrap_or_else(|| {
            panic!("`{atom}` must be rendered inside a `CrudTable` with `selectable`")
        })
}

/// The `<td>` of the surrounding row in `column`, shared by all cell atoms (`atom`).
///
/// The cell's key follows its column's position, which changes when columns before it are added
/// or removed. The cell, with its children, is then rendered again, as Leptonic's `TableCell` is.
#[component]
fn GridCell(
    atom: &'static str,
    column: Key,
    /// What gets focus: the cell (the default), or its first focusable child.
    #[prop(optional)]
    focus_mode: Option<CellFocusMode>,
    /// The column's kind of value, as `data-kind`.
    #[prop(optional_no_strip)]
    kind: Option<&'static str>,
    classes: Classes,
    styles: Styles,
    children: ChildrenFn,
) -> impl IntoView {
    let table = expect_crud_table_data(atom);
    let row = use_row_entity(atom).key.get_value();
    let collection = table.collection;
    let cell = Memo::new(move |_| {
        collection.with(|collection| {
            collection
                .column(&column)
                .map(|column| Key::cell(&row, column.index))
        })
    });
    move || {
        let Some(key) = cell.get() else {
            tracing::error!(
                atom,
                "a table cell references a column the table does not have"
            );
            return view! {
                <td class=classes.clone() style=styles.clone()>
                    {children()}
                </td>
            }
            .into_any();
        };
        let UseTableCellReturn {
            grid_cell_props,
            is_pressed,
        } = use_table_cell(UseTableCellInput {
            table: table.data.clone(),
            key,
            focus_mode,
            allows_arrow_navigation: false,
            should_select_on_press_up: false,
        });
        let (attrs, cell_styles) = grid_cell_props.into_parts();
        // Focus on the cell itself, and hover, as Leptonic's table atoms show them.
        let UseFocusRingReturn {
            props: focus_ring,
            is_focus_visible,
            ..
        } = use_focus_ring(UseFocusRingInput::default());
        let hover = use_hover(UseHoverInput::default());
        view! {
            <td
                {..attrs}
                {..focus_ring.into_attrs()}
                {..hover.props.into_attrs()}
                class=classes.clone()
                style=cell_styles.merge(styles.clone())
                data-kind=kind
                data-pressed=flag(is_pressed)
                data-focus-visible=flag(is_focus_visible)
                data-hovered=flag(hover.is_hovered)
            >
                {children()}
            </td>
        }
        .into_any()
    }
}
