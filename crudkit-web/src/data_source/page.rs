//! The pages of a list: what an instance asks a data source for, and how pages follow each other.

use crate::model::{DynReadField, DynReadModel};
use crudkit_core::Order;
use indexmap::IndexMap;

/// An opaque position in a list paged by cursor. A page ends with the cursor of the next page.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrudCursor(String);

impl CrudCursor {
    /// Wraps the cursor of a data source, as the source returned it.
    pub fn new(cursor: impl Into<String>) -> Self {
        Self(cursor.into())
    }

    /// Returns the cursor as the data source returned it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The page that [`CrudDataSource::list`](super::CrudDataSource::list) reads.
///
/// A request names the page both ways: by its offset and, once the list knows it, by the cursor
/// that opens it. A data source uses whichever way it pages by.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CrudListRequest {
    /// The maximum number of entities of the page.
    pub limit: u64,
    /// The number of entities before the page. Data sources that page by offset start here.
    pub offset: u64,
    /// The cursor with which the page before this one ended. Data sources that page by cursor read
    /// the page after it, or the first page without a cursor. `None` for the first page, and
    /// always for data sources that page by offset, as their pages end without a cursor.
    pub cursor: Option<CrudCursor>,
    /// Ordering criteria, highest priority first. Holds only fields that the data source
    /// [can order by](super::CrudDataSource::can_order_by). An empty map leaves the order to the
    /// data source.
    pub order_by: IndexMap<DynReadField, Order>,
}

impl CrudListRequest {
    /// Requests the page of at most `limit` entities after `offset` entities, or after `cursor`,
    /// ordered by `order_by`.
    #[must_use]
    pub fn new(
        limit: u64,
        offset: u64,
        cursor: Option<CrudCursor>,
        order_by: IndexMap<DynReadField, Order>,
    ) -> Self {
        Self {
            limit,
            offset,
            cursor,
            order_by,
        }
    }
}

/// What [`CrudDataSource::count`](super::CrudDataSource::count) counts: all entities of a list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct CrudCountRequest {}

impl CrudCountRequest {
    /// Requests the number of all entities in the scope.
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

/// One page of a list of entities.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CrudPage {
    /// The entities of the page, in the list's order.
    pub entities: Vec<DynReadModel>,
    /// How the page relates to the rest of the list.
    pub paging: CrudPaging,
}

impl CrudPage {
    /// A page holding `entities`, which relates to the rest of its list as `paging` says.
    #[must_use]
    pub fn new(entities: Vec<DynReadModel>, paging: CrudPaging) -> Self {
        Self { entities, paging }
    }
}

/// How the pages of a list follow each other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrudPaging {
    /// The data source opens every page by its offset. The number of all entities comes from
    /// [`CrudDataSource::count`](super::CrudDataSource::count), and an instance offers every page.
    Offset,
    /// The data source opens a page by the cursor with which the page before it ended. The list
    /// has no known total, so an instance offers only the previous and the next page.
    Cursor {
        /// The cursor that opens the next page. `None` on the last page.
        next: Option<CrudCursor>,
    },
}
