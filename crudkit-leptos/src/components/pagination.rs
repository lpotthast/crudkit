//! Paging controls.

use crate::components::with_classes;
use crate::hooks::list::CrudPaginationState;
use crate::hooks::texts::use_crud_texts;
use crudkit_web::list::ItemsPerPage;
use leptonic::atoms::prelude::{
    Button, Label, ListBox, ListBoxItem, ListBoxItemLabel, Select, SelectPopover, SelectTrigger,
    SelectValue,
};
use leptonic::hooks::{Key, use_collection};
use leptonic::utils::aria::AriaCurrent;
use leptonic::utils::classes::Classes;
use leptos::prelude::*;

/// Page navigation and page size selection for a list.
#[component]
pub fn CrudPagination(
    /// Paging state of the list, e.g. the `pagination` of its [`CrudListState`].
    ///
    /// [`CrudListState`]: crate::hooks::list::CrudListState
    pagination: CrudPaginationState,
    /// Additional classes of the root element.
    #[prop(into, optional)]
    classes: Classes,
) -> impl IntoView {
    let texts = use_crud_texts();
    view! {
        <div class=with_classes("crudkit-pagination", classes)>
            <CrudItemsPerPage pagination />
            <nav aria-label=texts.pagination.to_string() class="crudkit-pages">
                <CrudPageButtons pagination />
            </nav>
        </div>
    }
}

#[component]
fn CrudPageButtons(pagination: CrudPaginationState) -> impl IntoView {
    let texts = use_crud_texts();
    move || {
        let current = pagination.page.get();
        pagination
            .page_options
            .get()
            .options
            .into_iter()
            .map(|page| match page {
                Some(page) => view! {
                    <Button
                        aria_label=(texts.page)(page.0)
                        aria_current=(page == current).then_some(AriaCurrent::Page)
                        attr:data-current=(page == current).then_some("true")
                        on_press=move |_| pagination.set_page(page)
                        classes="crudkit-page-button"
                    >
                        {page.0}
                    </Button>
                }
                .into_any(),
                None => view! {
                    <span class="crudkit-page-gap" aria-hidden="true">
                        "\u{2026}"
                    </span>
                }
                .into_any(),
            })
            .collect_view()
    }
}

#[component]
fn CrudItemsPerPage(pagination: CrudPaginationState) -> impl IntoView {
    let label = use_crud_texts().items_per_page.to_string();
    let options = pagination.items_per_page_options;
    let collection = use_collection(move |b| {
        for option in options.get() {
            b.item(Key::from(option.0.to_string()), option.0.to_string());
        }
    });
    let value =
        Signal::derive(move || vec![Key::from(pagination.items_per_page.get().0.to_string())]);
    let set_value = Callback::new(move |keys: Vec<Key>| {
        let selected = keys
            .first()
            .and_then(|key| key.as_str())
            .and_then(|key| key.parse::<u64>().ok());
        if let Some(items_per_page) = selected {
            pagination.set_items_per_page(ItemsPerPage(items_per_page));
        }
    });
    view! {
        <Select collection value set_value classes="crudkit-items-per-page">
            <Label>{label}</Label>
            <SelectTrigger classes="crudkit-select-trigger">
                <SelectValue />
                <span aria-hidden="true">"\u{25be}"</span>
            </SelectTrigger>
            <SelectPopover classes="crudkit-select-popover">
                <ListBox classes="crudkit-listbox">
                    <For
                        each=move || options.get()
                        key=|option| option.0
                        children=|option| {
                            view! {
                                <ListBoxItem
                                    key=Key::from(option.0.to_string())
                                    classes="crudkit-listbox-item"
                                >
                                    <ListBoxItemLabel>{option.0}</ListBoxItemLabel>
                                </ListBoxItem>
                            }
                        }
                    />
                </ListBox>
            </SelectPopover>
        </Select>
    }
}
