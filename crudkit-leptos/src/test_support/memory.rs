//! A data source of items in memory, paged by offset or by cursor.

use crate::config::CrudInstanceConfig;
use crate::prelude::*;
use crate::test_support::items::{CreateItem, CrudItemResource, Item, ReadItem};
use crudkit_web::data_source::async_trait;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// How a [`MemorySource`] pages its lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MemoryPaging {
    Offset,
    Cursor,
}

/// An operation that a [`MemorySource`] received, with the scope it received it in.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MemoryOperation {
    List(CrudListRequest),
    Read(SerializableId),
    Create,
    Update(SerializableId),
    Delete(SerializableId),
}

/// Items in memory, ordered by ID unless a list asks otherwise. It orders by `name` only, and
/// records every operation with its scope.
#[derive(Debug)]
pub(crate) struct MemorySource {
    paging: MemoryPaging,
    items: Mutex<Vec<ReadItem>>,
    operations: Mutex<Vec<(MemoryOperation, CrudScope)>>,
}

impl MemorySource {
    /// Holds the items `1..=count`, named `item {id}`.
    pub(crate) fn with_items(count: i64, paging: MemoryPaging) -> Arc<Self> {
        Arc::new(Self {
            paging,
            items: Mutex::new(
                (1..=count)
                    .map(|id| ReadItem {
                        id,
                        name: format!("item {id}"),
                    })
                    .collect(),
            ),
            operations: Mutex::default(),
        })
    }

    /// The configuration of the items of this source, two per page.
    pub(crate) fn config(self: &Arc<Self>) -> CrudInstanceConfig {
        CrudInstanceConfig {
            items_per_page: ItemsPerPage(2),
            ..CrudInstanceConfig::with_data_source::<CrudItemResource>(self.clone())
        }
    }

    /// Returns every operation received so far with its scope, oldest first.
    pub(crate) fn operations(&self) -> Vec<(MemoryOperation, CrudScope)> {
        self.operations
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Returns the list requests received so far, oldest first.
    pub(crate) fn list_requests(&self) -> Vec<CrudListRequest> {
        self.operations()
            .into_iter()
            .filter_map(|(operation, _)| match operation {
                MemoryOperation::List(request) => Some(request),
                _ => None,
            })
            .collect()
    }

    /// Returns the IDs of the items held.
    pub(crate) fn ids(&self) -> Vec<i64> {
        self.items().iter().map(|item| item.id).collect()
    }

    fn items(&self) -> MutexGuard<'_, Vec<ReadItem>> {
        self.items.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn record(&self, operation: MemoryOperation, scope: &CrudScope) {
        self.operations
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((operation, scope.clone()));
    }

    /// Returns the page that `request` asks for.
    fn page(&self, request: &CrudListRequest) -> Result<CrudPage, RequestError> {
        let mut items = self.items().clone();
        for (field, order) in request.order_by.iter().rev() {
            let compare = match field.name().as_ref() {
                "name" => |a: &ReadItem, b: &ReadItem| a.name.cmp(&b.name),
                other => {
                    return Err(RequestError::InvalidRequest(format!(
                        "cannot order by {other}"
                    )));
                }
            };
            items.sort_by(|a, b| match order {
                Order::Asc => compare(a, b),
                Order::Desc => compare(a, b).reverse(),
            });
        }
        let start = match (self.paging, &request.cursor) {
            (MemoryPaging::Offset, _) => request.offset,
            (MemoryPaging::Cursor, None) => 0,
            (MemoryPaging::Cursor, Some(cursor)) => cursor
                .as_str()
                .parse()
                .map_err(|_| RequestError::InvalidRequest("unknown cursor".to_owned()))?,
        };
        let total = items.len() as u64;
        let end = start.saturating_add(request.limit).min(total);
        let paging = match self.paging {
            MemoryPaging::Offset => CrudPaging::Offset { total },
            MemoryPaging::Cursor => CrudPaging::Cursor {
                next: (end < total).then(|| CrudCursor::new(end.to_string())),
            },
        };
        Ok(CrudPage {
            entities: items
                .into_iter()
                .skip(usize::try_from(start).unwrap_or(usize::MAX))
                .take(usize::try_from(end.saturating_sub(start)).unwrap_or(usize::MAX))
                .map(DynReadModel::from)
                .collect(),
            paging,
        })
    }
}

/// The ID value of the item `id`.
fn item_id(id: &SerializableId) -> Result<i64, RequestError> {
    match id.entries().next().map(|entry| &entry.value) {
        Some(IdValue::I64(id)) => Ok(*id),
        _ => Err(RequestError::InvalidRequest(format!(
            "not an item ID: {id}"
        ))),
    }
}

#[async_trait(?Send)]
impl CrudDataSource for MemorySource {
    fn resource_name(&self) -> &'static str {
        "items"
    }

    fn can_order_by(&self, field: &DynReadField) -> bool {
        field.name() == "name"
    }

    async fn list(
        &self,
        scope: &CrudScope,
        request: CrudListRequest,
    ) -> Result<CrudPage, RequestError> {
        self.record(MemoryOperation::List(request.clone()), scope);
        self.page(&request)
    }

    async fn read(
        &self,
        scope: &CrudScope,
        id: &SerializableId,
    ) -> Result<Option<DynReadModel>, RequestError> {
        self.record(MemoryOperation::Read(id.clone()), scope);
        let id = item_id(id)?;
        Ok(self
            .items()
            .iter()
            .find(|item| item.id == id)
            .cloned()
            .map(DynReadModel::from))
    }

    async fn create(
        &self,
        scope: &CrudScope,
        entity: DynCreateModel,
    ) -> Result<Saved<DynUpdateModel>, RequestError> {
        self.record(MemoryOperation::Create, scope);
        let mut items = self.items();
        let item = ReadItem {
            id: items.iter().map(|item| item.id).max().unwrap_or_default() + 1,
            name: entity.downcast::<CreateItem>().name,
        };
        items.push(item.clone());
        Ok(Saved {
            entity: DynUpdateModel::from(Item::from(item)),
            violations: PartialSerializableAggregateViolations::default(),
        })
    }

    async fn update(
        &self,
        scope: &CrudScope,
        id: &SerializableId,
        entity: DynUpdateModel,
    ) -> Result<Saved<DynUpdateModel>, RequestError> {
        self.record(MemoryOperation::Update(id.clone()), scope);
        let id = item_id(id)?;
        let entity = entity.downcast::<Item>();
        let mut items = self.items();
        let item = items
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or_else(|| RequestError::NotFound(format!("item {id}")))?;
        item.name.clone_from(&entity.name);
        Ok(Saved {
            entity: DynUpdateModel::from(entity),
            violations: PartialSerializableAggregateViolations::default(),
        })
    }

    async fn delete(
        &self,
        scope: &CrudScope,
        id: &SerializableId,
    ) -> Result<Deleted, RequestError> {
        self.record(MemoryOperation::Delete(id.clone()), scope);
        let id = item_id(id)?;
        let mut items = self.items();
        let before = items.len();
        items.retain(|item| item.id != id);
        Ok(Deleted {
            entities_affected: (before - items.len()) as u64,
        })
    }
}
