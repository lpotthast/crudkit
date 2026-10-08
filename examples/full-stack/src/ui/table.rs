//! The app's list view: CrudKit's table, pagination, and button atoms in the app's markup.

use crate::ui::{icon, is_read_only, use_noun};
use crudkit_leptos::prelude::*;
use leptos::prelude::*;

#[component]
pub fn TableView() -> impl IntoView {
    view! {
        <CrudList>
            <ListPanel />
        </CrudList>
    }
}

#[component]
fn ListPanel() -> impl IntoView {
    let noun = use_noun();
    let read_only = is_read_only();
    let list = use_crud_list();
    let selected = list.selection.count;
    // Changes only when the selection becomes empty or not, so the button is not rebuilt as it grows.
    let has_selection = Memo::new(move |_| selected.get() > 0);
    let item_count = list.item_count;
    let summary = move || match (selected.get(), item_count.get()) {
        (0, Some(count)) => noun.count(usize::try_from(count).unwrap_or(usize::MAX)),
        (0, None) => "\u{2026}".to_owned(),
        (count, _) => format!("{count} selected"),
    };

    view! {
        <div class="ui-panel">
            <div class="ui-toolbar">
                <div class="ui-toolbar-start">
                    <span class="ui-count" class:ui-count-selected=has_selection role="status">
                        {summary}
                    </span>
                    {move || {
                        (has_selection.get() && !read_only)
                            .then(|| {
                                view! {
                                    <CrudDeleteSelectedButton classes=[
                                        "ui-button",
                                        "ui-button-small",
                                        "ui-button-ghost-danger",
                                    ]>{icon(icondata::LuTrash2)} "Delete"</CrudDeleteSelectedButton>
                                }
                            })
                    }}
                </div>
                <div class="ui-button-row">
                    <CrudResourceActions classes="ui-button" />
                    {(!read_only)
                        .then(|| {
                            view! {
                                <CrudCreateButton classes=[
                                    "ui-button",
                                    "ui-button-primary",
                                ]>{icon(icondata::LuPlus)} {format!("New {}", noun.singular)}</CrudCreateButton>
                            }
                        })}
                </div>
            </div>
            <div class="ui-table-scroll">
                <CrudTable
                    selectable=!read_only
                    with_actions=true
                    row_action=if read_only { CrudRowAction::Read } else { CrudRowAction::Edit }
                    classes="ui-table"
                >
                    <CrudTableHeader>
                        <CrudTableHeaderRow>
                            {(!read_only)
                                .then(|| {
                                    view! {
                                        <CrudTableSelectAllHeader>
                                            <CrudSelectAllCheckbox aria_label="Select all" />
                                        </CrudTableSelectAllHeader>
                                    }
                                })}
                            <CrudTableColumnHeaders />
                            <CrudTableActionsHeader>
                                <span class="ui-sr-only">"Actions"</span>
                            </CrudTableActionsHeader>
                        </CrudTableHeaderRow>
                    </CrudTableHeader>
                    <CrudTableBody>
                        <CrudTableRows>
                            <CrudTableRow classes="ui-row">
                                {(!read_only)
                                    .then(|| {
                                        view! {
                                            <CrudTableSelectionCell>
                                                <CrudRowCheckbox aria_label="Select row" />
                                            </CrudTableSelectionCell>
                                        }
                                    })}
                                <CrudTableCells />
                                <CrudTableActionsCell classes="ui-row-actions">
                                    <CrudReadButton classes="ui-icon-button" aria_label="View">
                                        {icon(icondata::LuEye)}
                                    </CrudReadButton>
                                    {(!read_only)
                                        .then(|| {
                                            view! {
                                                <CrudEditButton classes="ui-icon-button" aria_label="Edit">
                                                    {icon(icondata::LuPencil)}
                                                </CrudEditButton>
                                                <CrudDeleteButton
                                                    classes=["ui-icon-button", "ui-icon-button-danger"]
                                                    aria_label="Delete"
                                                >
                                                    {icon(icondata::LuTrash2)}
                                                </CrudDeleteButton>
                                            }
                                        })}
                                </CrudTableActionsCell>
                            </CrudTableRow>
                        </CrudTableRows>
                    </CrudTableBody>
                </CrudTable>
            </div>
            <CrudListStatus
                classes="ui-empty"
                loading=|| view! { <span class="ui-muted">"Consulting the registry\u{2026}"</span> }
                empty=move || {
                    view! {
                        <span class="ui-empty-icon">{icon(icondata::LuScrollText)}</span>
                        <strong>{format!("No {} yet", noun.plural)}</strong>
                        {(!read_only)
                            .then(|| {
                                view! {
                                    <span>
                                        {format!("Create the first {} to see it here.", noun.singular)}
                                    </span>
                                }
                            })}
                    }
                }
                failed=|| view! { <p class="ui-error">"The registry could not be read."</p> }
            />
            <Pager />
        </div>
    }
}

#[component]
fn Pager() -> impl IntoView {
    let pagination = use_crud_list().pagination;
    let range = move || {
        pagination.range().map(|range| {
            format!(
                "Showing {}\u{2013}{} of {}",
                range.first, range.last, range.total
            )
        })
    };
    view! {
        <CrudPagination classes="ui-pager" aria_label="Pages">
            <span class="ui-muted">{range}</span>
            <div class="ui-pages">
                <CrudPreviousPageButton classes="ui-page" aria_label="Previous page">
                    {icon(icondata::LuChevronLeft)}
                </CrudPreviousPageButton>
                <CrudPageButtons button_classes="ui-page" gap_classes="ui-page-gap" />
                <CrudNextPageButton classes="ui-page" aria_label="Next page">
                    {icon(icondata::LuChevronRight)}
                </CrudNextPageButton>
            </div>
        </CrudPagination>
    }
}
