//! Paging through the surrounding list.

use crate::atoms::content::content_or_default;
use crate::hooks::list::expect_crud_list;
use crate::hooks::texts::use_crud_texts;
use crudkit_web::list::{ItemsPerPage, PageNr};
use leptonic::atoms::prelude::{Button, ListBoxItem, Select};
use leptonic::hooks::{Key, use_collection};
use leptonic::utils::aria::AriaCurrent;
use leptonic::utils::classes::{Classes, MergeStrategy};
use leptonic::utils::styles::Styles;
use leptos::prelude::*;

/// The `<nav>` holding the paging controls of the surrounding list, e.g.
/// [`CrudPreviousPageButton`], [`CrudPageButtons`], [`CrudNextPageButton`], and
/// [`CrudItemsPerPage`].
///
/// Labeled [`CrudUiTexts::pagination`] unless given `aria_label`.
///
/// Default class: `crudkit-Pagination`.
///
/// [`CrudUiTexts::pagination`]: crate::config::CrudUiTexts::pagination
#[component]
pub fn CrudPagination(
    #[prop(into, optional)] aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-Pagination")
        .merge(classes, MergeStrategy::UnionConditions);
    let texts = use_crud_texts();
    let aria_label = move || {
        aria_label
            .get()
            .unwrap_or_else(|| texts.read().pagination.to_string())
    };
    view! {
        <nav class=classes style=styles aria-label=aria_label>
            {children()}
        </nav>
    }
}

/// A [`CrudPageButton`] per page worth offering, and a [`CrudPageGap`] where pages are left out:
/// the first and last page, and the pages around the current one. Renders no element of its own.
#[component]
pub fn CrudPageButtons(
    /// Classes of each page button.
    #[prop(into, optional)]
    button_classes: Classes,
    /// Classes of each gap.
    #[prop(into, optional)]
    gap_classes: Classes,
) -> impl IntoView {
    let pagination = expect_crud_list("CrudPageButtons").pagination;
    let items = move || page_items(&pagination.page_options.get().options);
    view! {
        <For
            each=items
            key=|item| *item
            children=move |item| match item {
                PageItem::Page(page) => {
                    view! { <CrudPageButton page classes=button_classes.clone() /> }.into_any()
                }
                PageItem::Gap { .. } => {
                    view! { <CrudPageGap classes=gap_classes.clone() /> }.into_any()
                }
            }
        />
    }
}

/// An entry of [`CrudPageButtons`]. Entries are keyed by the pages they stand for, so that a
/// page's button survives changing the current page and keeps its focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PageItem {
    Page(PageNr),
    /// Pages left out after the page `after`.
    Gap {
        after: PageNr,
    },
}

/// Returns the entries for the offered page `options`, where `None` is a gap.
fn page_items(options: &[Option<PageNr>]) -> Vec<PageItem> {
    let mut previous = PageNr::first();
    options
        .iter()
        .map(|option| match option {
            Some(page) => {
                previous = *page;
                PageItem::Page(*page)
            }
            None => PageItem::Gap { after: previous },
        })
        .collect()
}

/// Opens `page` of the surrounding list. The current page's button has `aria-current="page"`.
///
/// Default content: the page number. Labeled [`CrudUiTexts::page`] unless given `aria_label`.
///
/// Data attributes: those of Leptonic's `Button`.
///
/// Default classes: `leptonic-Button crudkit-PageButton`.
///
/// [`CrudUiTexts::page`]: crate::config::CrudUiTexts::page
#[component]
pub fn CrudPageButton(
    /// The page this button opens.
    page: PageNr,
    #[prop(into, optional)] aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-PageButton")
        .merge(classes, MergeStrategy::UnionConditions);
    let pagination = expect_crud_list("CrudPageButton").pagination;
    let texts = use_crud_texts();
    let aria_label = MaybeProp::derive(move || {
        Some(
            aria_label
                .get()
                .unwrap_or_else(|| (texts.read().page)(page.0)),
        )
    });
    let aria_current =
        Signal::derive(move || (pagination.page.get() == page).then_some(AriaCurrent::Page));
    view! {
        <Button on_press=move |_| pagination.set_page(page) aria_label aria_current classes styles>
            {content_or_default(children, page.0)}
        </Button>
    }
}

/// Marks pages left out between [`CrudPageButton`]s: a `<span>` hidden from assistive technology.
///
/// Default content: an ellipsis.
///
/// Default class: `crudkit-PageGap`.
#[component]
pub fn CrudPageGap(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-PageGap")
        .merge(classes, MergeStrategy::UnionConditions);
    view! {
        <span class=classes style=styles aria-hidden="true">
            {content_or_default(children, "\u{2026}")}
        </span>
    }
}

/// Opens the previous page of the surrounding list. Disabled on the first page.
///
/// Default content: [`CrudUiTexts::previous_page`].
///
/// Data attributes: those of Leptonic's `Button`.
///
/// Default classes: `leptonic-Button crudkit-PreviousPageButton`.
///
/// [`CrudUiTexts::previous_page`]: crate::config::CrudUiTexts::previous_page
#[component]
pub fn CrudPreviousPageButton(
    #[prop(into, optional)] is_disabled: Signal<bool>,
    /// Labels the button when its content doesn't, e.g. an icon.
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-PreviousPageButton")
        .merge(classes, MergeStrategy::UnionConditions);
    let pagination = expect_crud_list("CrudPreviousPageButton").pagination;
    let texts = use_crud_texts();
    let text = move || texts.read().previous_page.to_string();
    view! {
        <Button
            on_press=move |_| pagination.previous()
            is_disabled=Signal::derive(move || !pagination.has_previous() || is_disabled.get())
            aria_label
            classes
            styles
        >
            {content_or_default(children, text)}
        </Button>
    }
}

/// Opens the next page of the surrounding list. Disabled on the last page.
///
/// Default content: [`CrudUiTexts::next_page`].
///
/// Data attributes: those of Leptonic's `Button`.
///
/// Default classes: `leptonic-Button crudkit-NextPageButton`.
///
/// [`CrudUiTexts::next_page`]: crate::config::CrudUiTexts::next_page
#[component]
pub fn CrudNextPageButton(
    #[prop(into, optional)] is_disabled: Signal<bool>,
    /// Labels the button when its content doesn't, e.g. an icon.
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-NextPageButton")
        .merge(classes, MergeStrategy::UnionConditions);
    let pagination = expect_crud_list("CrudNextPageButton").pagination;
    let texts = use_crud_texts();
    let text = move || texts.read().next_page.to_string();
    view! {
        <Button
            on_press=move |_| pagination.next()
            is_disabled=Signal::derive(move || !pagination.has_next() || is_disabled.get())
            aria_label
            classes
            styles
        >
            {content_or_default(children, text)}
        </Button>
    }
}

/// Chooses how many entities a page of the surrounding list shows: a Leptonic `Select` bound to the
/// list's page size.
///
/// Compose it like a Leptonic `Select`: a `Label` (e.g. [`CrudUiTexts::items_per_page`]), a
/// `SelectTrigger` with a `SelectValue`, and a `SelectPopover` holding a `ListBox` with
/// [`CrudItemsPerPageOptions`].
///
/// Data attributes: those of Leptonic's `Select`.
///
/// Default classes: `leptonic-Select crudkit-ItemsPerPage`.
///
/// [`CrudUiTexts::items_per_page`]: crate::config::CrudUiTexts::items_per_page
#[component]
pub fn CrudItemsPerPage(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-ItemsPerPage")
        .merge(classes, MergeStrategy::UnionConditions);
    let pagination = expect_crud_list("CrudItemsPerPage").pagination;
    let options = pagination.items_per_page_options;
    let collection = use_collection(move |builder| {
        for option in options.get() {
            builder.item(option_key(option), option.0.to_string());
        }
    });
    let value = Signal::derive(move || vec![option_key(pagination.items_per_page.get())]);
    let set_value = move |keys: Vec<Key>| {
        let selected = keys
            .first()
            .and_then(Key::as_str)
            .and_then(|key| key.parse::<u64>().ok());
        if let Some(items_per_page) = selected {
            pagination.set_items_per_page(ItemsPerPage(items_per_page));
        }
    };
    view! {
        <Select collection value set_value classes styles>
            {children()}
        </Select>
    }
}

/// A Leptonic `ListBoxItem` per page size offered by the surrounding [`CrudItemsPerPage`]. Renders
/// no element of its own.
#[component]
pub fn CrudItemsPerPageOptions(
    /// Classes of each option.
    #[prop(into, optional)]
    classes: Classes,
) -> impl IntoView {
    let options = expect_crud_list("CrudItemsPerPageOptions")
        .pagination
        .items_per_page_options;
    view! {
        <For
            each=move || options.get()
            key=|option| option.0
            children=move |option| {
                view! {
                    <ListBoxItem key=option_key(option) classes=classes.clone()>
                        {option.0}
                    </ListBoxItem>
                }
            }
        />
    }
}

/// The key of the page size `option` in the `Select`'s collection.
fn option_key(option: ItemsPerPage) -> Key {
    Key::from(option.0.to_string())
}

#[cfg(test)]
mod tests {
    use super::{PageItem, page_items};
    use assertr::prelude::*;
    use crudkit_web::list::PageNr;

    #[test]
    fn page_items_key_gaps_by_the_page_before_them() {
        let items = page_items(&[
            Some(PageNr(1)),
            None,
            Some(PageNr(5)),
            None,
            Some(PageNr(9)),
        ]);
        assert_that!(items).is_equal_to(vec![
            PageItem::Page(PageNr(1)),
            PageItem::Gap { after: PageNr(1) },
            PageItem::Page(PageNr(5)),
            PageItem::Gap { after: PageNr(5) },
            PageItem::Page(PageNr(9)),
        ]);
    }
}
