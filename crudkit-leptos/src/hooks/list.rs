//! Paged, ordered, and selectable entity lists.

use crate::hooks::instance::use_crud_instance;
use crate::instance::CrudInstanceContext;
use crudkit_core::Order;
use crudkit_web::list::{self, ItemsPerPage, PageNr, PageOptions, Selection};
use crudkit_web::load_state::LoadState;
use crudkit_web::prelude::*;
use indexmap::IndexMap;
use leptos::prelude::*;
use std::sync::Arc;

/// State of one mounted list of entities.
///
/// Loading, paging, and ordering belong to the surrounding instance, so they survive switching
/// between views. Selection belongs to the list that created this state.
#[derive(Debug, Clone, Copy)]
pub struct CrudListState {
    /// Entities of the current page.
    pub rows: Signal<LoadState<Arc<Vec<DynReadModel>>>>,
    /// Whether the current page contains at least one entity.
    pub has_rows: Signal<bool>,
    /// Total number of entities matching the instance's base condition, once known.
    pub item_count: Signal<Option<u64>>,
    /// Paging of the list.
    pub pagination: CrudPaginationState,
    /// Ordering of the list.
    pub ordering: CrudOrderingState,
    /// Selected entities of the current page.
    pub selection: CrudSelectionState,
}

/// Creates the state of a list of entities of the surrounding instance.
///
/// Each call loads data independently and owns its own selection, so call it once per mounted
/// list and share the returned `Copy` handle, e.g. through context.
///
/// # Panics
///
/// Panics when called outside of a CrudKit instance.
#[must_use]
pub fn use_crud_list() -> CrudListState {
    let ctx = use_crud_instance();

    // TODO: Do not use LocalResource, allow loading on the server.
    let page_resource = LocalResource::new(move || async move {
        let _ = ctx.reload.get();
        let items_per_page = ctx.items_per_page.get();
        let page = ctx.current_page.get();
        ctx.data_provider
            .get()
            .read_many(ReadMany {
                limit: Some(items_per_page.0),
                skip: Some(page.skip(items_per_page)),
                order_by: Some(ctx.order_by.get()),
                condition: ctx.base_condition.get(),
            })
            .await
    });

    let rows = Memo::new(move |_prev| match page_resource.get() {
        Some(result) => LoadState::from(result.map(Arc::new)),
        None => LoadState::Loading,
    });

    let count_resource = LocalResource::new(move || async move {
        let _ = ctx.reload.get();
        ctx.data_provider
            .get()
            .read_count(ReadCount {
                condition: ctx.base_condition.get(),
            })
            .await
    });
    let item_count = Signal::derive(move || {
        count_resource.get().and_then(|result| {
            result
                .inspect_err(|err| tracing::warn!(%err, "could not count entities"))
                .ok()
        })
    });

    let has_rows =
        Signal::derive(move || rows.read().loaded().is_some_and(|rows| !rows.is_empty()));

    CrudListState {
        rows: rows.into(),
        has_rows,
        item_count,
        pagination: CrudPaginationState::new(&ctx, item_count),
        ordering: CrudOrderingState { ctx },
        selection: CrudSelectionState::new(rows.into()),
    }
}

/// Paging of a list. Page and page size are owned by the instance.
#[derive(Debug, Clone, Copy)]
pub struct CrudPaginationState {
    ctx: CrudInstanceContext,
    /// The currently displayed page.
    pub page: Signal<PageNr>,
    /// The number of entities per page.
    pub items_per_page: Signal<ItemsPerPage>,
    /// Total number of entities, once known.
    pub item_count: Signal<Option<u64>>,
    /// Number of pages. Zero while the count is unknown or no entity exists.
    pub page_count: Signal<u64>,
    /// Pages a pagination control should offer.
    pub page_options: Signal<PageOptions>,
    /// Page sizes a page size control should offer. Always contains the current page size.
    pub items_per_page_options: Signal<Vec<ItemsPerPage>>,
}

impl CrudPaginationState {
    fn new(ctx: &CrudInstanceContext, item_count: Signal<Option<u64>>) -> Self {
        let page: Signal<PageNr> = ctx.current_page.into();
        let items_per_page: Signal<ItemsPerPage> = ctx.items_per_page.into();
        let page_count = Signal::derive(move || {
            list::page_count(item_count.get().unwrap_or_default(), items_per_page.get())
        });
        Self {
            ctx: *ctx,
            page,
            items_per_page,
            item_count,
            page_count,
            page_options: Signal::derive(move || PageOptions::new(page_count.get(), page.get())),
            items_per_page_options: Signal::derive(move || {
                ItemsPerPage::options_including(items_per_page.get())
            }),
        }
    }

    /// Shows `page`.
    pub fn set_page(&self, page: PageNr) {
        self.ctx.set_page(page);
    }

    /// Changes the page size and keeps the current page within the new page range.
    pub fn set_items_per_page(&self, items_per_page: ItemsPerPage) {
        self.ctx.set_items_per_page(items_per_page);
        if let Some(item_count) = self.item_count.get_untracked() {
            let current = self.page.get_untracked();
            let clamped = current.clamp_to(list::page_count(item_count, items_per_page));
            if clamped != current {
                self.ctx.set_page(clamped);
            }
        }
    }

    /// Returns whether a page before the current one exists.
    #[must_use]
    pub fn has_previous(&self) -> bool {
        self.page.get().0 > 1
    }

    /// Returns whether a page after the current one exists.
    #[must_use]
    pub fn has_next(&self) -> bool {
        self.page.get().0 < self.page_count.get()
    }

    /// Shows the previous page, if any.
    pub fn previous(&self) {
        let page = self.page.get_untracked();
        if page.0 > 1 {
            self.set_page(PageNr(page.0 - 1));
        }
    }

    /// Shows the next page, if any.
    pub fn next(&self) {
        let page = self.page.get_untracked();
        if page.0 < self.page_count.get_untracked() {
            self.set_page(PageNr(page.0 + 1));
        }
    }
}

/// Ordering of a list. The ordering is owned by the instance.
#[derive(Debug, Clone, Copy)]
pub struct CrudOrderingState {
    ctx: CrudInstanceContext,
}

impl CrudOrderingState {
    /// Returns the ordering, in priority order.
    #[must_use]
    pub fn order_by(&self) -> Signal<IndexMap<DynReadField, Order>> {
        self.ctx.order_by.into()
    }

    /// Returns the order applied to `field`, if any. Tracks the ordering.
    #[must_use]
    pub fn order_of(&self, field: &DynReadField) -> Option<Order> {
        self.ctx.order_by.read().get(field).copied()
    }

    /// Orders by `field` only, in `order`.
    pub fn set(&self, field: DynReadField, order: Order) {
        let mut order_by = IndexMap::new();
        order_by.insert(field, order);
        self.ctx.set_order_by(order_by);
    }

    /// Removes all ordering.
    pub fn clear(&self) {
        self.ctx.set_order_by(IndexMap::new());
    }

    /// Toggles `field` between ascending and descending order.
    ///
    /// Without `append`, `field` replaces any existing ordering. With `append`, it is added as the
    /// lowest-priority ordering criterion.
    pub fn toggle(&self, field: DynReadField, append: bool) {
        self.ctx.toggle_order_by(field, append);
    }
}

/// Selected entities of one mounted list.
///
/// Entities that are no longer displayed are deselected whenever the displayed rows change.
#[derive(Debug, Clone, Copy)]
pub struct CrudSelectionState {
    rows: Signal<LoadState<Arc<Vec<DynReadModel>>>>,
    selection: RwSignal<Selection<DynReadModel>>,
    /// Whether every displayed entity is selected. `false` while nothing is displayed.
    pub all_selected: Signal<bool>,
    /// Number of selected entities.
    pub count: Signal<usize>,
}

impl CrudSelectionState {
    fn new(rows: Signal<LoadState<Arc<Vec<DynReadModel>>>>) -> Self {
        let selection = RwSignal::new(Selection::<DynReadModel>::default());

        // Deselect entities that are no longer displayed, e.g. after paging, reordering, or deletion.
        Effect::new(move || match rows.read().loaded() {
            Some(rows) => selection.update(|selection| selection.retain_present(rows)),
            None => selection.update(Selection::clear),
        });

        Self {
            rows,
            selection,
            all_selected: Signal::derive(move || {
                rows.read()
                    .loaded()
                    .is_some_and(|rows| selection.read().all_selected(rows))
            }),
            count: Signal::derive(move || selection.read().len()),
        }
    }

    /// Returns whether `entity` is selected. Tracks the selection.
    #[must_use]
    pub fn is_selected(&self, entity: &DynReadModel) -> bool {
        self.selection.read().is_selected(entity)
    }

    /// Returns the selected entities in selection order. Tracks the selection.
    #[must_use]
    pub fn selected(&self) -> Vec<DynReadModel> {
        self.selection.read().selected().to_vec()
    }

    /// Selects or deselects `entity`.
    pub fn set(&self, entity: DynReadModel, selected: bool) {
        self.selection
            .update(|selection| selection.set(entity, selected));
    }

    /// Toggles the selection of `entity`.
    pub fn toggle(&self, entity: DynReadModel) {
        self.selection.update(|selection| selection.toggle(entity));
    }

    /// Selects every displayed entity, or deselects everything if all of them are selected.
    pub fn toggle_all(&self) {
        if let Some(rows) = self.rows.get_untracked().into_loaded() {
            self.selection
                .update(|selection| selection.toggle_all(&rows));
        }
    }

    /// Deselects everything.
    pub fn clear(&self) {
        self.selection.update(Selection::clear);
    }

    /// Selects exactly `entities`.
    pub fn replace(&self, entities: Vec<DynReadModel>) {
        self.selection.update(|selection| {
            selection.clear();
            for entity in entities {
                selection.set(entity, true);
            }
        });
    }
}
