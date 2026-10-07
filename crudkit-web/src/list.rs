//! UI-independent list state: pagination, ordering, and selection.

use crudkit_core::Order;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Number of entities shown on one list page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ItemsPerPage(pub u64);

impl Default for ItemsPerPage {
    fn default() -> Self {
        Self(10)
    }
}

impl ItemsPerPage {
    /// Page sizes offered by default list controls.
    pub const DEFAULT_OPTIONS: [ItemsPerPage; 4] = [
        ItemsPerPage(10),
        ItemsPerPage(25),
        ItemsPerPage(50),
        ItemsPerPage(100),
    ];

    /// Returns [`Self::DEFAULT_OPTIONS`] plus `current`, sorted and without duplicates.
    ///
    /// The current page size is always included so that a selection control can display it.
    #[must_use]
    pub fn options_including(current: ItemsPerPage) -> Vec<ItemsPerPage> {
        let mut options = Self::DEFAULT_OPTIONS.to_vec();
        if !options.contains(&current) {
            options.push(current);
        }
        options.sort();
        options
    }
}

/// One-based page number.
// TODO: Consider a `NonZero` representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PageNr(pub u64);

impl Default for PageNr {
    fn default() -> Self {
        Self::first()
    }
}

impl PageNr {
    /// The first page.
    #[must_use]
    pub const fn first() -> Self {
        Self(1)
    }

    /// Returns the number of entities to skip before this page.
    #[must_use]
    pub fn skip(self, items_per_page: ItemsPerPage) -> u64 {
        items_per_page.0.saturating_mul(self.0.saturating_sub(1))
    }

    /// Clamps this page into `1..=page_count`. An empty list keeps the first page.
    #[must_use]
    pub fn clamp_to(self, page_count: u64) -> Self {
        Self(self.0.clamp(1, page_count.max(1)))
    }
}

/// Returns the number of pages needed to show `item_count` entities.
///
/// A page size of zero yields zero pages.
#[must_use]
pub fn page_count(item_count: u64, items_per_page: ItemsPerPage) -> u64 {
    if items_per_page.0 == 0 {
        0
    } else {
        item_count.div_ceil(items_per_page.0)
    }
}

/// Page numbers a pagination control offers for navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageOptions {
    /// Pages to which navigation can occur.
    ///
    /// `None` describes a gap in the offered numbers. A control can ignore gaps or render them as a
    /// non-interactive ellipsis. Example for page 13 of 42:
    /// `[1, 2, None, 11, 12, 13, 14, 15, None, 41, 42]`.
    pub options: Vec<Option<PageNr>>,
}

impl PageOptions {
    /// Computes the offered pages for `current_page` out of `page_count` pages.
    ///
    /// Up to ten pages are listed completely. Longer ranges keep the first and last pages and a
    /// window around the current page.
    #[must_use]
    pub fn new(page_count: u64, current_page: PageNr) -> Self {
        let current = current_page.clamp_to(page_count).0;
        let pages = |range: std::ops::RangeInclusive<u64>| range.map(|it| Some(PageNr(it)));

        let options = if page_count <= 10 {
            pages(1..=page_count).collect()
        } else if current <= 5 {
            pages(1..=7)
                .chain([None])
                .chain(pages(page_count - 1..=page_count))
                .collect()
        } else if current < page_count - 4 {
            pages(1..=2)
                .chain([None])
                .chain(pages(current - 2..=current + 2))
                .chain([None])
                .chain(pages(page_count - 1..=page_count))
                .collect()
        } else {
            pages(1..=2)
                .chain([None])
                .chain(pages(page_count - 6..=page_count))
                .collect()
        };
        Self { options }
    }
}

/// Applies an ordering interaction for `field`.
///
/// The interaction toggles the field between ascending and descending, starting with ascending. It
/// clears the existing ordering first unless `append` is set.
pub fn toggle_order<F: Hash + Eq>(order_by: &mut IndexMap<F, Order>, field: F, append: bool) {
    let next = match order_by.get(&field) {
        Some(Order::Asc) => Order::Desc,
        Some(Order::Desc) | None => Order::Asc,
    };
    if !append {
        order_by.clear();
    }
    order_by.insert(field, next);
}

/// Entities selected in a list.
///
/// Selection is compared by entity equality. Owners must call [`Self::retain_present`] whenever the
/// displayed dataset changes, so that no stale entity remains selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection<T> {
    selected: Vec<T>,
}

impl<T> Default for Selection<T> {
    fn default() -> Self {
        Self {
            selected: Vec::new(),
        }
    }
}

impl<T: PartialEq + Clone> Selection<T> {
    /// Returns the selected entities in selection order.
    #[must_use]
    pub fn selected(&self) -> &[T] {
        &self.selected
    }

    /// Returns the number of selected entities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.selected.len()
    }

    /// Returns whether nothing is selected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    /// Returns whether `entity` is selected.
    #[must_use]
    pub fn is_selected(&self, entity: &T) -> bool {
        self.selected.contains(entity)
    }

    /// Selects or deselects `entity`.
    pub fn set(&mut self, entity: T, selected: bool) {
        let position = self.selected.iter().position(|it| it == &entity);
        match (position, selected) {
            (None, true) => self.selected.push(entity),
            (Some(position), false) => {
                self.selected.remove(position);
            }
            (None, false) | (Some(_), true) => {}
        }
    }

    /// Toggles the selection of `entity`.
    pub fn toggle(&mut self, entity: T) {
        let selected = !self.is_selected(&entity);
        self.set(entity, selected);
    }

    /// Deselects everything.
    pub fn clear(&mut self) {
        self.selected.clear();
    }

    /// Returns whether `displayed` is non-empty and every displayed entity is selected.
    #[must_use]
    pub fn all_selected(&self, displayed: &[T]) -> bool {
        !displayed.is_empty() && displayed.iter().all(|it| self.is_selected(it))
    }

    /// Selects all `displayed` entities, or deselects everything if all of them are already selected.
    pub fn toggle_all(&mut self, displayed: &[T]) {
        if self.all_selected(displayed) {
            self.clear();
        } else {
            self.selected = displayed.to_vec();
        }
    }

    /// Deselects every entity that is not part of `displayed`.
    pub fn retain_present(&mut self, displayed: &[T]) {
        self.selected.retain(|it| displayed.contains(it));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;

    fn numbers(options: &PageOptions) -> Vec<Option<u64>> {
        options.options.iter().map(|it| it.map(|it| it.0)).collect()
    }

    #[test]
    fn page_count_rounds_up_and_handles_zero() {
        assert_that!(page_count(0, ItemsPerPage(10))).is_equal_to(0);
        assert_that!(page_count(10, ItemsPerPage(10))).is_equal_to(1);
        assert_that!(page_count(11, ItemsPerPage(10))).is_equal_to(2);
        assert_that!(page_count(11, ItemsPerPage(0))).is_equal_to(0);
    }

    #[test]
    fn page_is_clamped_into_valid_range() {
        assert_that!(PageNr(0).clamp_to(3)).is_equal_to(PageNr(1));
        assert_that!(PageNr(5).clamp_to(3)).is_equal_to(PageNr(3));
        assert_that!(PageNr(5).clamp_to(0)).is_equal_to(PageNr(1));
        assert_that!(PageNr(2).clamp_to(3)).is_equal_to(PageNr(2));
    }

    #[test]
    fn skip_is_zero_on_first_page_and_never_underflows() {
        assert_that!(PageNr(1).skip(ItemsPerPage(25))).is_equal_to(0);
        assert_that!(PageNr(3).skip(ItemsPerPage(25))).is_equal_to(50);
        assert_that!(PageNr(0).skip(ItemsPerPage(25))).is_equal_to(0);
    }

    #[test]
    fn items_per_page_options_include_current_value_once() {
        assert_that!(ItemsPerPage::options_including(ItemsPerPage(25)).len()).is_equal_to(4);
        assert_that!(ItemsPerPage::options_including(ItemsPerPage(30))).is_equal_to(vec![
            ItemsPerPage(10),
            ItemsPerPage(25),
            ItemsPerPage(30),
            ItemsPerPage(50),
            ItemsPerPage(100),
        ]);
    }

    #[test]
    fn page_options_list_all_pages_when_there_are_few() {
        assert_that!(numbers(&PageOptions::new(0, PageNr(1)))).is_empty();
        assert_that!(numbers(&PageOptions::new(3, PageNr(2)))).is_equal_to(vec![
            Some(1),
            Some(2),
            Some(3),
        ]);
    }

    #[test]
    fn page_options_window_around_current_page() {
        assert_that!(numbers(&PageOptions::new(42, PageNr(2)))).is_equal_to(vec![
            Some(1),
            Some(2),
            Some(3),
            Some(4),
            Some(5),
            Some(6),
            Some(7),
            None,
            Some(41),
            Some(42),
        ]);
        assert_that!(numbers(&PageOptions::new(42, PageNr(13)))).is_equal_to(vec![
            Some(1),
            Some(2),
            None,
            Some(11),
            Some(12),
            Some(13),
            Some(14),
            Some(15),
            None,
            Some(41),
            Some(42),
        ]);
        assert_that!(numbers(&PageOptions::new(42, PageNr(40)))).is_equal_to(vec![
            Some(1),
            Some(2),
            None,
            Some(36),
            Some(37),
            Some(38),
            Some(39),
            Some(40),
            Some(41),
            Some(42),
        ]);
    }

    #[test]
    fn page_options_clamp_an_out_of_range_current_page() {
        assert_that!(numbers(&PageOptions::new(42, PageNr(100))))
            .is_equal_to(numbers(&PageOptions::new(42, PageNr(42))));
    }

    #[test]
    fn toggle_order_cycles_and_respects_append() {
        let mut order_by = IndexMap::new();
        toggle_order(&mut order_by, "a", false);
        assert_that!(order_by.get("a").copied()).is_equal_to(Some(Order::Asc));
        toggle_order(&mut order_by, "a", false);
        assert_that!(order_by.get("a").copied()).is_equal_to(Some(Order::Desc));
        toggle_order(&mut order_by, "b", true);
        assert_that!(order_by.len()).is_equal_to(2);
        toggle_order(&mut order_by, "c", false);
        assert_that!(order_by.len()).is_equal_to(1);
        assert_that!(order_by.get("c").copied()).is_equal_to(Some(Order::Asc));
    }

    #[test]
    fn all_selected_checks_membership_not_length() {
        let mut selection = Selection::default();
        selection.set(1, true);
        selection.set(2, true);
        assert_that!(selection.all_selected(&[1, 2])).is_true();
        assert_that!(selection.all_selected(&[3, 4])).is_false();
        assert_that!(selection.all_selected(&[])).is_false();
    }

    #[test]
    fn toggle_all_selects_displayed_or_clears() {
        let mut selection = Selection::default();
        selection.set(9, true);
        selection.toggle_all(&[1, 2]);
        assert_that!(selection.selected()).is_equal_to([1, 2].as_slice());
        selection.toggle_all(&[1, 2]);
        assert_that!(selection.is_empty()).is_true();
    }

    #[test]
    fn retain_present_drops_stale_entities() {
        let mut selection = Selection::default();
        selection.set(1, true);
        selection.set(2, true);
        selection.retain_present(&[2, 3]);
        assert_that!(selection.selected()).is_equal_to([2].as_slice());
    }

    #[test]
    fn toggle_and_set_are_idempotent_where_expected() {
        let mut selection = Selection::default();
        selection.set(1, true);
        selection.set(1, true);
        assert_that!(selection.len()).is_equal_to(1);
        selection.toggle(1);
        assert_that!(selection.is_empty()).is_true();
        selection.set(1, false);
        assert_that!(selection.is_empty()).is_true();
    }
}
