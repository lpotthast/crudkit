use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use yewdux::prelude::*;

use crate::prelude::*;

// NOTE: This type is not generic, as every crud instance should have access to the current view state of instances of arbitrary generic T.
// Therefor there must only be one single store, which only is the case if not generic itself.
// This limits what we can store, so we store a type erased ID in the form of SerializableCrudViews.
#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize, Store)]
#[store(storage = "local", storage_tab_sync)]
pub struct InstanceViewsStore {
    // serde bound used as described in: https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    instances: HashMap<String, SerializableCrudView>,
}

impl InstanceViewsStore {
    pub fn get(&self, instance_name: &str) -> Option<SerializableCrudView> {
        self.instances.get(instance_name).cloned()
    }

    pub fn save(&mut self, instance_name: String, crud_view: SerializableCrudView) {
        self.instances.insert(instance_name, crud_view);
    }
}

// impl Store for InstanceViewsStore {
//     fn new() -> Self {
//         init_listener(storage::StorageListener::<Self>::new(storage::Area::Local));

//         storage::load(storage::Area::Local)
//             .map_err(|error| {
//                 // TODO: Erase from local store
//                 error!("Unable to load state due to StorageError: {}", error);
//             })
//             .ok()
//             .flatten()
//             .unwrap_or_default()
//     }

//     fn should_notify(&self, other: &Self) -> bool {
//         self != other
//     }
// }
