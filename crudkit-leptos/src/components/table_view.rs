//! The built-in table view.

use crate::atoms::icon::CrudIcon;
use crate::atoms::table::{
    CrudTable, CrudTableBody, CrudTableCell, CrudTableHeader, CrudTableRows,
};
use crate::components::action_buttons::CrudResourceActionButtons;
use crate::components::button::CrudButton;
use crate::components::field::hosted_field;
use crate::components::pagination::CrudPagination;
use crate::components::with_classes;
use crate::config::CrudActionIntent;
use crate::config::{FieldRendererRegistry, Header};
use crate::hooks::delete::use_crud_delete;
use crate::hooks::field::use_crud_row_fields;
use crate::hooks::instance::use_crud_instance;
use crate::hooks::list::{CrudListState, use_crud_list};
use crate::hooks::table::{CrudTableColumn, use_crud_table_data};
use crate::hooks::texts::use_crud_texts;
use crate::instance::CrudNavigation;
use crate::instance::with_view_overrides;
use crudkit_web::field::FieldOptions;
use crudkit_web::load_state::LoadState;
use crudkit_web::prelude::*;
use crudkit_web::view::CrudView;
use leptonic::atoms::prelude::Button;
use leptonic::hooks::CellFocusMode;
use leptonic::utils::classes::Classes;
use leptos::context::Provider;
use leptos::prelude::*;

/// Shows the entities of the surrounding instance as a paged, sortable, selectable table.
///
/// A given `navigation` also receives the follow-ups of deletions requested in the table.
#[component]
pub fn CrudTableView(
    /// The shown columns. Defaults to the instance's configuration.
    #[prop(into, optional)]
    headers: Option<Signal<Vec<Header>>>,
    /// Defaults to the instance's configuration.
    #[prop(into, optional)]
    field_renderer_registry: Option<Signal<FieldRendererRegistry<DynReadField>>>,
    /// Defaults to the navigation of the current view.
    #[prop(optional)]
    navigation: Option<CrudNavigation>,
    /// Additional classes of the root element.
    #[prop(into, optional)]
    classes: Classes,
) -> impl IntoView {
    with_view_overrides(navigation, None, move || {
        let ctx = use_crud_instance();
        let headers = headers.unwrap_or_else(|| ctx.list_columns());
        let field_renderer_registry =
            field_renderer_registry.unwrap_or_else(|| Signal::stored(ctx.read_field_renderers()));
        let navigation = ctx.navigation;
        let list = use_crud_list();
        let show_delete = ctx.builtin_view_controls().show_delete;

        view! {
            <Provider value=list>
                <div class=with_classes("crudkit-table-view", classes)>
                    <CrudTableToolbar navigation />
                    <CrudSelectionBar />
                    <div class="crudkit-table-container">
                        <CrudTable
                            columns=headers
                            selectable=show_delete
                            with_actions=true
                            aria_label=ctx.static_config.read_value().resource_name.clone()
                            on_row_action=move |entity: DynReadModel| {
                                navigation.navigate(CrudView::edit(entity.id()));
                            }
                            classes="crudkit-table"
                        >
                            <CrudTableHeader />
                            <CrudTableBody>
                                <CrudTableRows row_classes="crudkit-table-row" let:entity>
                                    <CrudTableRowCells
                                        entity
                                        field_renderer_registry
                                        navigation
                                        show_delete
                                    />
                                </CrudTableRows>
                            </CrudTableBody>
                        </CrudTable>
                    </div>
                    <CrudTableStatus />
                    {move || {
                        list.item_count
                            .get()
                            .is_some()
                            .then(|| view! { <CrudPagination pagination=list.pagination /> })
                    }}
                </div>
            </Provider>
        }
    })
}

/// The toolbar above the table: create, resource actions, and reset.
#[component]
pub fn CrudTableToolbar(
    /// Navigation of the list view, through which the create button opens the create view.
    navigation: CrudNavigation,
    /// Additional classes of the root element.
    #[prop(into, optional)]
    classes: Classes,
) -> impl IntoView {
    let ctx = use_crud_instance();
    let texts = use_crud_texts();
    let (new_label, reset_label) = (texts.new.to_string(), texts.reset.to_string());
    view! {
        <div class=with_classes("crudkit-toolbar", classes) role="toolbar">
            <div class="crudkit-toolbar-start">
                <CrudButton
                    intent=CrudActionIntent::Success
                    on_press=move |_| navigation.navigate(CrudView::create())
                >
                    <CrudIcon icon=icondata::BsPlusCircle classes="crudkit-icon" />
                    {new_label}
                </CrudButton>
                <CrudResourceActionButtons />
            </div>
            <div class="crudkit-toolbar-end">
                <CrudButton on_press=move |_| ctx.reset()>
                    <CrudIcon icon=icondata::BsArrowRepeat classes="crudkit-icon" />
                    {reset_label}
                </CrudButton>
            </div>
        </div>
    }
}

/// Shows the number of selected entities and offers to delete them.
#[component]
pub fn CrudSelectionBar(
    /// Additional classes of the root element.
    #[prop(into, optional)]
    classes: Classes,
) -> impl IntoView {
    let deletion = use_crud_delete();
    let list = expect_context::<CrudListState>();
    let texts = use_crud_texts();
    move || {
        let count = list.selection.count.get();
        (count > 0).then(|| {
            let texts = texts.clone();
            view! {
                <div class=with_classes("crudkit-selection-bar", classes.clone())>
                    <span>{(texts.selected)(count as u64)}</span>
                    <CrudButton
                        intent=CrudActionIntent::Danger
                        on_press=move |_| {
                            deletion.request_many(list.selection.selected());
                        }
                    >
                        <CrudIcon icon=icondata::BsTrash classes="crudkit-icon" />
                        {texts.delete_selection.to_string()}
                    </CrudButton>
                </div>
            }
        })
    }
}

/// Explains an empty or failed list below the table.
#[component]
fn CrudTableStatus() -> impl IntoView {
    let list = expect_context::<CrudListState>();
    let texts = use_crud_texts();
    move || match list.rows.get() {
        LoadState::Loading => None,
        LoadState::Loaded(rows) if !rows.is_empty() => None,
        LoadState::Loaded(_) | LoadState::NotFound => Some(
            view! { <div class="crudkit-table-status">{texts.no_data.to_string()}</div> }
                .into_any(),
        ),
        LoadState::Failed(reason) => Some(
            view! {
                <div class="crudkit-table-status" role="alert" data-failed="true">
                    {format!("{}: {reason}", texts.data_unavailable)}
                </div>
            }
            .into_any(),
        ),
    }
}

#[component]
fn CrudTableRowCells(
    entity: DynReadModel,
    field_renderer_registry: Signal<FieldRendererRegistry<DynReadField>>,
    navigation: CrudNavigation,
    show_delete: bool,
) -> impl IntoView {
    let deletion = use_crud_delete();
    let table = use_crud_table_data();
    let texts = use_crud_texts();
    let fields = use_crud_row_fields(&entity);
    let data_cells = table
        .columns
        .get_untracked()
        .into_iter()
        .map(|Header { field, options }| {
            let display_options = FieldOptions {
                disabled: false,
                label: None,
                date_time_display: options.date_time_display,
            };
            let content = if let Some(state) = fields.display_state(&field, display_options) {
                hosted_field(field_renderer_registry, state).into_any()
            } else {
                tracing::error!(field = %field.name(), "a list column references a field the read model does not have");
                ().into_any()
            };
            view! {
                <CrudTableCell column=field classes=("crudkit-fit-content", options.min_width)>
                    {content}
                </CrudTableCell>
            }
        })
        .collect_view();

    let (read_entity, edit_entity, delete_entity) = (entity.clone(), entity.clone(), entity);
    view! {
        {data_cells}
        <CrudTableCell
            column=CrudTableColumn::Actions
            focus_mode=CellFocusMode::Child
            classes="crudkit-row-actions"
        >
            <Button
                aria_label=texts.view.to_string()
                attr:title=texts.view.to_string()
                classes="crudkit-icon-button"
                on_press=move |_| navigation.navigate(CrudView::read(read_entity.id()))
            >
                <CrudIcon icon=icondata::BsEye classes="crudkit-icon" />
            </Button>
            <Button
                aria_label=texts.edit.to_string()
                attr:title=texts.edit.to_string()
                classes="crudkit-icon-button"
                on_press=move |_| navigation.navigate(CrudView::edit(edit_entity.id()))
            >
                <CrudIcon icon=icondata::BsPencil classes="crudkit-icon" />
            </Button>
            {show_delete
                .then(|| {
                    let delete_entity = delete_entity.clone();
                    view! {
                        <Button
                            aria_label=texts.delete.to_string()
                            attr:title=texts.delete.to_string()
                            classes=["crudkit-icon-button", CrudActionIntent::Danger.class()]
                            on_press=move |_| {
                                deletion.request(delete_entity.clone());
                            }
                        >
                            <CrudIcon icon=icondata::BsTrash classes="crudkit-icon" />
                        </Button>
                    }
                })}
        </CrudTableCell>
    }
}
