//! A data source of items in memory, paged by offset or by cursor, with versions like an API
//! with optimistic concurrency.

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
    Count,
    Read(SerializableId),
    Create,
    Update(CrudUpdateRequest),
    Delete(CrudDeleteRequest),
}

impl MemoryOperation {
    /// The name of the operation's kind.
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::List(_) => "list",
            Self::Count => "count",
            Self::Read(_) => "read",
            Self::Create => "create",
            Self::Update(_) => "update",
            Self::Delete(_) => "delete",
        }
    }
}

/// The name that a [`MemorySource`] refuses, as another item has it.
pub(crate) const TAKEN_NAME: &str = "taken";

/// Items in memory, ordered by ID unless a list asks otherwise. It orders by `name` only, and
/// records every operation with its scope.
///
/// Every item has a version, which every change raises. Like an API with optimistic concurrency,
/// the source refuses a change or deletion based on another version as outdated, and applies the
/// merge patch of a change to the item as it is now. It refuses the name [`TAKEN_NAME`] with the
/// problems of two fields, `name` and `secret`, a field that items do not have. Changes and
/// deletions complete, unless [`Self::accept_changes`] makes the source accept them: a deletion
/// then keeps the item.
#[derive(Debug)]
pub(crate) struct MemorySource {
    paging: MemoryPaging,
    items: Mutex<Vec<ReadItem>>,
    operations: Mutex<Vec<(MemoryOperation, CrudScope)>>,
    completion: Mutex<CrudCompletion>,
}

impl MemorySource {
    /// Holds the items `1..=count`, named `item {id}`, in their version `1`.
    pub(crate) fn with_items(count: i64, paging: MemoryPaging) -> Arc<Self> {
        Arc::new(Self {
            paging,
            items: Mutex::new(
                (1..=count)
                    .map(|id| ReadItem {
                        version: Some(CrudVersion::new("1")),
                        ..ReadItem::numbered(id)
                    })
                    .collect(),
            ),
            operations: Mutex::default(),
            completion: Mutex::new(CrudCompletion::Completed),
        })
    }

    /// Makes the source accept changes and deletions instead of completing them, like an API
    /// answering `202 Accepted`.
    pub(crate) fn accept_changes(&self) {
        *self
            .completion
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = CrudCompletion::Accepted;
    }

    /// Changes the note of the item `id` as another user would, which raises its version.
    pub(crate) fn change_elsewhere(&self, id: i64, note: &str) {
        let mut items = self.items();
        let item = items
            .iter_mut()
            .find(|item| item.id == id)
            .expect("the item exists");
        note.clone_into(&mut item.note);
        item.version = Some(next_version(item.version.as_ref()));
    }

    /// Returns the item `id` as the source holds it.
    pub(crate) fn item(&self, id: i64) -> Option<ReadItem> {
        self.items().iter().find(|item| item.id == id).cloned()
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
            MemoryPaging::Offset => CrudPaging::Offset,
            MemoryPaging::Cursor => CrudPaging::Cursor {
                next: (end < total).then(|| CrudCursor::new(end.to_string())),
            },
        };
        let entities = items
            .into_iter()
            .skip(usize::try_from(start).unwrap_or(usize::MAX))
            .take(usize::try_from(end.saturating_sub(start)).unwrap_or(usize::MAX))
            .map(DynReadModel::from)
            .collect();
        Ok(CrudPage::new(entities, paging))
    }
}

/// Returns the version after `version`.
fn next_version(version: Option<&CrudVersion>) -> CrudVersion {
    let number = version
        .and_then(|version| version.as_str().parse::<u64>().ok())
        .unwrap_or_default();
    CrudVersion::new((number + 1).to_string())
}

/// Refuses a request based on `version` if `item` has another version. A request without a
/// version is not checked.
fn check_version(item: &ReadItem, version: Option<&CrudVersion>) -> Result<(), RequestError> {
    match version {
        Some(version) if item.version.as_ref() != Some(version) => {
            Err(RequestError::Outdated(format!(
                "item {} changed since version {}",
                item.id,
                version.as_str()
            )))
        }
        _ => Ok(()),
    }
}

/// The refusal of the name [`TAKEN_NAME`].
fn taken_name() -> RequestError {
    RequestError::InvalidFields {
        message: "The item is invalid.".to_owned(),
        problems: vec![
            CrudFieldProblem::new("name", "Another item has this name."),
            CrudFieldProblem::new("secret", "The secret is wrong."),
        ],
    }
}

/// Applies the JSON merge patch (RFC 7396) `patch` to `target`.
fn apply_merge_patch(target: &mut serde_json::Value, patch: &serde_json::Value) {
    let serde_json::Value::Object(patch) = patch else {
        patch.clone_into(target);
        return;
    };
    if !target.is_object() {
        *target = serde_json::Value::Object(serde_json::Map::new());
    }
    if let serde_json::Value::Object(target) = target {
        for (key, value) in patch {
            if value.is_null() {
                target.remove(key);
            } else {
                apply_merge_patch(
                    target.entry(key.clone()).or_insert(serde_json::Value::Null),
                    value,
                );
            }
        }
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

    async fn count(
        &self,
        scope: &CrudScope,
        _request: CrudCountRequest,
    ) -> Result<Option<u64>, RequestError> {
        self.record(MemoryOperation::Count, scope);
        Ok(match self.paging {
            MemoryPaging::Offset => Some(self.items().len() as u64),
            MemoryPaging::Cursor => None,
        })
    }

    async fn read(
        &self,
        scope: &CrudScope,
        request: CrudReadRequest,
    ) -> Result<Option<DynReadModel>, RequestError> {
        self.record(MemoryOperation::Read(request.id.clone()), scope);
        let id = item_id(&request.id)?;
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
        request: CrudCreateRequest,
    ) -> Result<CrudSaved, RequestError> {
        self.record(MemoryOperation::Create, scope);
        let name = request.entity.downcast::<CreateItem>().name;
        if name == TAKEN_NAME {
            return Err(taken_name());
        }
        let mut items = self.items();
        let item = ReadItem {
            name,
            version: Some(CrudVersion::new("1")),
            ..ReadItem::numbered(items.iter().map(|item| item.id).max().unwrap_or_default() + 1)
        };
        items.push(item.clone());
        Ok(CrudSaved::new(DynUpdateModel::from(Item::from(item))))
    }

    async fn update(
        &self,
        scope: &CrudScope,
        request: CrudUpdateRequest,
    ) -> Result<CrudSaved, RequestError> {
        self.record(MemoryOperation::Update(request.clone()), scope);
        let id = item_id(&request.id)?;
        let patch = request
            .merge_patch()
            .map_err(|err| RequestError::InvalidRequest(err.to_string()))?;
        if patch["name"] == TAKEN_NAME {
            return Err(taken_name());
        }
        let completion = *self
            .completion
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut items = self.items();
        let item = items
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or_else(|| RequestError::NotFound(format!("item {id}")))?;
        check_version(item, request.version())?;
        let mut changed = serde_json::to_value(&*item)
            .map_err(|err| RequestError::Deserialize(err.to_string()))?;
        apply_merge_patch(&mut changed, &patch);
        *item = serde_json::from_value(changed)
            .map_err(|err| RequestError::InvalidRequest(err.to_string()))?;
        item.version = Some(next_version(item.version.as_ref()));
        let saved = DynUpdateModel::from(Item::from(item.clone()));
        Ok(match completion {
            CrudCompletion::Completed => CrudSaved::new(saved),
            CrudCompletion::Accepted => CrudSaved::accepted(saved),
        })
    }

    async fn delete(
        &self,
        scope: &CrudScope,
        request: CrudDeleteRequest,
    ) -> Result<CrudDeleted, RequestError> {
        self.record(MemoryOperation::Delete(request.clone()), scope);
        let id = item_id(&request.id)?;
        let completion = *self
            .completion
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut items = self.items();
        if let Some(item) = items.iter().find(|item| item.id == id) {
            check_version(item, request.version())?;
            if completion == CrudCompletion::Accepted {
                return Ok(CrudDeleted::accepted(DynReadModel::from(item.clone())));
            }
        }
        let before = items.len();
        items.retain(|item| item.id != id);
        Ok(CrudDeleted::new((before - items.len()) as u64))
    }
}
