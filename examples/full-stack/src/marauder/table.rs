//! Marauder's list view: CrudKit's headless table atoms in Marauder markup.

use crate::marauder::{icon, use_noun};
use crudkit_leptos::prelude::*;
use leptonic::atoms::prelude::Button;
use leptonic::hooks::CellFocusMode;
use leptos::prelude::*;

/// Formats a listed value as text.
fn display(value: &Value) -> String {
    match value {
        Value::Null => "\u{2014}".to_owned(),
        Value::Bool(true) => "Yes".to_owned(),
        Value::Bool(false) => "No".to_owned(),
        Value::String(text) => text.clone(),
        Value::I32(it) => it.to_string(),
        Value::I64(it) => it.to_string(),
        Value::U32(it) => it.to_string(),
        Value::U64(it) => it.to_string(),
        Value::PrimitiveDateTime(it) => {
            format!(
                "{} {}, {}",
                &it.month().to_string()[..3],
                it.day(),
                it.year()
            )
        }
        other => format!("{other:?}"),
    }
}

/// Numbers are right-aligned with tabular figures. Everything else reads as text.
fn is_numeric(value: Option<&Value>) -> bool {
    matches!(
        value,
        Some(Value::I32(_) | Value::I64(_) | Value::U32(_) | Value::U64(_))
    )
}

#[component]
pub fn MarauderTableView(navigation: CrudNavigation) -> impl IntoView {
    let ctx = use_crud_instance();
    let noun = use_noun();
    let deletion = use_crud_delete();
    let list = use_crud_list();
    let actions = use_crud_actions();
    let selected = list.selection.count;
    let item_count = list.item_count;

    let resource_actions = actions
        .resource_actions()
        .into_iter()
        .map(|action| {
            // The handle shows the action's view first if it has one.
            let CrudActionHandle {
                press,
                is_disabled,
                view,
            } = actions.bind_resource_action(&action);
            view! {
                <Button classes="mrd-button" is_disabled on_press=move |_| press.run(())>
                    {icon(icondata::LuRefreshCw)}
                    {action.name}
                </Button>
                {view}
            }
        })
        .collect_view();

    let summary = move || match (selected.get(), item_count.get()) {
        (0, Some(1)) => format!("1 {}", noun.singular),
        (0, Some(count)) => format!("{count} {}", noun.plural),
        (0, None) => "\u{2026}".to_owned(),
        (count, _) => format!("{count} selected"),
    };
    let rows = list.rows;
    let is_empty = Signal::derive(move || rows.read().loaded().is_some_and(|rows| rows.is_empty()));

    view! {
        <div class="mrd-panel">
            <div class="mrd-toolbar">
                <div class="mrd-toolbar-start">
                    <span class="mrd-count" class:mrd-count-selected=move || { selected.get() > 0 }>
                        {summary}
                    </span>
                    {move || {
                        (selected.get() > 0)
                            .then(|| {
                                view! {
                                    <Button
                                        classes=[
                                            "mrd-button",
                                            "mrd-button-small",
                                            "mrd-button-ghost-danger",
                                        ]
                                        on_press=move |_| {
                                            deletion.request_many(list.selection.selected())
                                        }
                                    >
                                        {icon(icondata::LuTrash2)}
                                        "Delete"
                                    </Button>
                                }
                            })
                    }}
                </div>
                <div class="mrd-button-row">
                    {resource_actions}
                    <Button
                        classes=["mrd-button", "mrd-button-primary"]
                        on_press=move |_| navigation.navigate(CrudView::create())
                    >
                        {icon(icondata::LuPlus)}
                        {format!("New {}", noun.singular)}
                    </Button>
                </div>
            </div>
            <div class="mrd-table-scroll">
                <CrudTable
                    list
                    columns=ctx.list_columns()
                    selectable=true
                    with_actions=true
                    aria_label=ctx.resource_name()
                    on_row_action=move |entity: DynReadModel| {
                        navigation.navigate(CrudView::edit(entity.id()))
                    }
                    classes="mrd-table"
                >
                    <CrudTableHeader select_all_label="Select all" />
                    <CrudTableBody>
                        <CrudTableRows select_label="Select row" row_classes="mrd-row" let:entity>
                            <MarauderRowCells entity navigation />
                        </CrudTableRows>
                    </CrudTableBody>
                </CrudTable>
            </div>
            {move || {
                is_empty
                    .get()
                    .then(|| {
                        view! {
                            <div class="mrd-empty">
                                <span class="mrd-empty-icon">{icon(icondata::LuScrollText)}</span>
                                <strong>{format!("No {} yet", noun.plural)}</strong>
                                <span>
                                    {format!("Create the first {} to see it here.", noun.singular)}
                                </span>
                            </div>
                        }
                    })
            }}
            <MarauderPager pagination=list.pagination />
        </div>
    }
}

#[component]
fn MarauderRowCells(entity: DynReadModel, navigation: CrudNavigation) -> impl IntoView {
    let ctx = use_crud_instance();
    let deletion = use_crud_delete();
    let table = use_crud_table_data();
    let fields = use_crud_row_fields(&entity);
    let renderers = ctx.read_field_renderers();
    let cells = table
        .columns
        .get_untracked()
        .into_iter()
        .map(|Header { field, .. }| {
            let value = fields.value(&field);
            // Registered renderers, such as the validation badge, take precedence over plain text.
            let content = match (
                renderers.get(&field),
                fields.display_state(&field, FieldOptions::default()),
            ) {
                (Some(renderer), Some(state)) => renderer.render(state),
                (None, Some(_)) => value.as_ref().map(display).unwrap_or_default().into_any(),
                (_, None) => ().into_any(),
            };
            let classes = ("mrd-cell-number", is_numeric(value.as_ref()));
            view! {
                <CrudTableCell column=field classes>
                    {content}
                </CrudTableCell>
            }
        })
        .collect_view();
    let (edit, delete) = (entity.clone(), entity);
    view! {
        {cells}
        <CrudTableCell
            column=CrudTableColumn::Actions
            focus_mode=CellFocusMode::Child
            classes="mrd-row-actions"
        >
            <Button
                classes="mrd-icon-button"
                on_press=move |_| navigation.navigate(CrudView::edit(edit.id()))
            >
                {icon(icondata::LuPencil)}
                <span class="mrd-sr-only">"Edit"</span>
            </Button>
            <Button
                classes=["mrd-icon-button", "mrd-icon-button-danger"]
                on_press=move |_| deletion.request(delete.clone())
            >
                {icon(icondata::LuTrash2)}
                <span class="mrd-sr-only">"Delete"</span>
            </Button>
        </CrudTableCell>
    }
}

#[component]
fn MarauderPager(pagination: CrudPaginationState) -> impl IntoView {
    let range = move || {
        let total = pagination.item_count.get().unwrap_or_default();
        let per_page = pagination.items_per_page.get().0;
        let first = pagination.page.get().0.saturating_sub(1) * per_page + 1;
        let last = (first + per_page - 1).min(total);
        if total == 0 {
            String::new()
        } else {
            format!("Showing {first}\u{2013}{last} of {total}")
        }
    };
    let pages = move || {
        let current = pagination.page.get();
        pagination
            .page_options
            .get()
            .options
            .into_iter()
            .map(|page| match page {
                Some(page) => view! {
                    <Button
                        classes="mrd-page"
                        attr:aria-current=(page == current).then_some("page")
                        on_press=move |_| pagination.set_page(page)
                    >
                        {page.0}
                    </Button>
                }
                .into_any(),
                None => view! { <span class="mrd-page-gap">"\u{2026}"</span> }.into_any(),
            })
            .collect_view()
    };
    view! {
        <nav class="mrd-pager" aria-label="Pages">
            <span class="mrd-muted">{range}</span>
            <div class="mrd-pages">
                <Button
                    classes="mrd-page"
                    is_disabled=Signal::derive(move || !pagination.has_previous())
                    on_press=move |_| pagination.previous()
                >
                    {icon(icondata::LuChevronLeft)}
                    <span class="mrd-sr-only">"Previous page"</span>
                </Button>
                {pages}
                <Button
                    classes="mrd-page"
                    is_disabled=Signal::derive(move || !pagination.has_next())
                    on_press=move |_| pagination.next()
                >
                    {icon(icondata::LuChevronRight)}
                    <span class="mrd-sr-only">"Next page"</span>
                </Button>
            </div>
        </nav>
    }
}
