//! Test fixtures: an `items` resource served from memory.

use crate::config::{CrudInstanceConfig, FieldRendererRegistry};
use crate::instance::{CrudInstanceContext, ProvideCrudInstanceInput, provide_crud_instance};
use crate::prelude::*;
use crate::test_support::{CrudTestResponse, CrudTestServer, with_crud_manager};
use indexmap::IndexMap;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone, PartialEq, Eq, Debug, CkId, CkField, CkResource, Serialize, Deserialize)]
#[ck_resource(resource_name = "items")]
#[ck_field(model = Update)]
pub struct Item {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Default, CkField, Serialize, Deserialize)]
#[ck_field(model = Create)]
pub struct CreateItem {
    pub name: String,
}

impl ErasedIdentifiable for CreateItem {
    fn id(&self) -> SerializableId {
        panic!("Create models are not identifiable!")
    }
}

#[derive(Clone, PartialEq, Eq, Debug, CkId, CkField, Serialize, Deserialize)]
#[ck_field(model = Read)]
pub struct ReadItem {
    pub id: i64,
    pub name: String,
}

impl From<ReadItem> for Item {
    fn from(read: ReadItem) -> Self {
        Self {
            id: read.id,
            name: read.name,
        }
    }
}

/// Serves CrudKit's REST routes for items from memory, through a [`CrudTestServer`].
#[derive(Debug)]
pub(crate) struct FakeServer {
    pub(crate) http: Arc<CrudTestServer>,
}

impl FakeServer {
    pub(crate) fn with_items(count: i64) -> Arc<Self> {
        let items = Mutex::new(
            (1..=count)
                .map(|id| ReadItem {
                    id,
                    name: format!("item {id}"),
                })
                .collect::<Vec<_>>(),
        );
        Arc::new(Self {
            http: CrudTestServer::new(move |request| {
                CrudTestResponse::json(respond(&items, &request.operation, &request.body))
            }),
        })
    }

    pub(crate) fn requests_to(&self, operation: &str) -> Vec<serde_json::Value> {
        self.http.requests_to(operation)
    }
}

/// Answers `operation` like CrudKit's server, on `items`.
fn respond(
    items: &Mutex<Vec<ReadItem>>,
    operation: &str,
    body: &serde_json::Value,
) -> serde_json::Value {
    let items = items.lock().expect("lock");
    let saved = |name: &serde_json::Value| {
        serde_json::json!({
            "wire_format_version": 1,
            "entity": { "id": 99, "name": name },
            "violations": { "general": null, "create": null, "by_entity": [] },
        })
    };
    match operation {
        "read-count" => serde_json::json!({ "wire_format_version": 1, "count": items.len() }),
        "read-many" => {
            let skip =
                usize::try_from(body["skip"].as_u64().unwrap_or_default()).unwrap_or(usize::MAX);
            let limit =
                usize::try_from(body["limit"].as_u64().unwrap_or(u64::MAX)).unwrap_or(usize::MAX);
            serde_json::json!({
                "wire_format_version": 1,
                "entities": items.iter().skip(skip).take(limit).collect::<Vec<_>>(),
            })
        }
        "read-one" => serde_json::json!({ "wire_format_version": 1, "entity": items.first() }),
        "create-one" => saved(&body["entity"]["name"]),
        // Lets tests make a save fail: the response does not deserialize.
        "update-one" if body["entity"]["name"] == "reject" => {
            serde_json::json!({ "rejected": true })
        }
        // Like a real server, normalize what is stored.
        "update-one" => serde_json::json!({
            "wire_format_version": 1,
            "entity": {
                "id": body["entity"]["id"],
                "name": body["entity"]["name"].as_str().map(str::trim),
            },
            "violations": { "general": null, "create": null, "by_entity": [] },
        }),
        "delete-by-id" => serde_json::json!({ "wire_format_version": 1, "entities_affected": 1 }),
        "delete-many" => serde_json::json!({
            "wire_format_version": 1,
            "deleted_count": 2,
            "deleted_ids": [],
            "aborted": [],
            "validation_failed": [],
            "errors": [],
        }),
        other => panic!("unexpected operation {other}"),
    }
}

pub(crate) fn config(server: &Arc<FakeServer>) -> CrudInstanceConfig {
    CrudInstanceConfig {
        api_base_url: "http://test.local/api".to_owned(),
        initial_view: CrudView::table(),
        list_columns: Vec::new(),
        create_elements: Vec::new(),
        elements: Vec::new(),
        order_by: IndexMap::default(),
        items_per_page: ItemsPerPage(2),
        page_nr: PageNr::first(),
        base_condition: None,
        resource_name: "items".to_owned(),
        reqwest_executor: server.http.executor(),
        model_handler: ModelHandler::new::<CreateItem, ReadItem, Item>(),
        actions: Vec::new(),
        entity_actions: Vec::new(),
        view_registry: None,
        read_field_renderer: FieldRendererRegistry::default(),
        create_field_renderer: FieldRendererRegistry::default(),
        update_field_renderer: FieldRendererRegistry::default(),
    }
}

/// Mounts an instance backed by `server` and runs `test` inside it.
pub(crate) fn with_instance(server: &Arc<FakeServer>, test: impl FnOnce(CrudInstanceContext)) {
    with_crud_manager(|| {
        let ctx = provide_crud_instance(instance_input("items", config(server)));
        test(ctx);
    });
}

pub(crate) fn item(id: i64) -> DynReadModel {
    DynReadModel::from(ReadItem {
        id,
        name: format!("item {id}"),
    })
}

/// The input mounting `config` as `name`, without parent or own navigation.
pub(crate) fn instance_input(
    name: &'static str,
    config: CrudInstanceConfig,
) -> ProvideCrudInstanceInput {
    ProvideCrudInstanceInput {
        name,
        config,
        parent: None,
        navigation: None,
    }
}

/// The input editing the entity `id`, without callbacks.
pub(crate) fn edit_input(id: SerializableId) -> UseCrudEditFormInput {
    UseCrudEditFormInput {
        id: Signal::stored(id),
        on_saved: None,
        on_save_failed: None,
        notifications: CrudSaveNotifications::All,
    }
}
