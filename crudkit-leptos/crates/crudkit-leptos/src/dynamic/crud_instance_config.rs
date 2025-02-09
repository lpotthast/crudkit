use crate::dynamic::crud_action::{CrudAction, CrudEntityAction};
use crate::dynamic::custom_field::{CustomCreateFields, CustomReadFields, CustomUpdateFields};
use crate::shared::crud_instance_config::DynSelectConfig;
use crate::ReactiveValue;
use crudkit_condition::Condition;
use crudkit_shared::Order;
use crudkit_web::prelude::*;
use crudkit_web::{AnyElem, AnyField, AnyModel, SerializableCrudView};
use indexmap::{indexmap, IndexMap};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)] // TODO: Serialize, Deserialize
pub struct CrudInstanceConfig {
    pub api_base_url: String,
    pub view: SerializableCrudView,
    pub headers: Vec<(AnyField, HeaderOptions)>, // Read model field!
    pub create_elements: CreateElements,
    pub elements: Vec<AnyElem>,              // UpdateModel
    pub order_by: IndexMap<AnyField, Order>, // Read model field name!
    pub items_per_page: u64,
    pub page: u64,
    pub active_tab: Option<Label>,
    pub base_condition: Option<Condition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrudParentConfig {
    /// The name of the parent instance from which the referenced id should be loaded.
    pub name: &'static str,

    /// The field of the parent instance from which the referenced id should be loaded. For example: "id".
    pub referenced_field: String,

    /// The `own` field in which the reference is stored. For example: "user_id", when referencing a User entity.
    pub referencing_field: String, // TODO: This should be: T::ReadModel::Field? (ClusterCertificateField::CreatedAt)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CreateElements {
    None,
    Custom(Vec<AnyElem>), // CreateModel
}

/// This config is non-serializable. Every piece of runtime-changing data relevant to be tracked and reloaded should be part of the CrudInstanceConfig struct.
#[derive(Debug, Clone)]
pub struct CrudStaticInstanceConfig {
    pub resource_name: String,
    pub reqwest_executor: Arc<dyn ReqwestExecutor>,
    pub actions: Vec<CrudAction>,
    pub deserialize_read_many_response:
        Callback<(serde_json::Value,), Result<Vec<AnyModel>, serde_json::Error>>,
    pub deserialize_read_one_response:
        Callback<(serde_json::Value,), Result<Option<AnyModel>, serde_json::Error>>,

    pub read_model_to_update_model: Callback<(AnyModel,), AnyModel>,
    pub read_model_to_signal_map: Callback<(AnyModel,), HashMap<AnyField, ReactiveValue>>,
    pub update_model_to_signal_map: Callback<(AnyModel,), HashMap<AnyField, ReactiveValue>>,

    pub entity_actions: Vec<CrudEntityAction>,
    pub create_field_select_config: HashMap<AnyField, DynSelectConfig>, // CreateModel field
    pub read_field_select_config: HashMap<AnyField, DynSelectConfig>,   // ReadModel field
    pub update_field_select_config: HashMap<AnyField, DynSelectConfig>, // UpdateModel field
    pub custom_read_fields: CustomReadFields,
    pub custom_create_fields: CustomCreateFields,
    pub custom_update_fields: CustomUpdateFields,
}

//impl Default for CrudStaticInstanceConfig {
//    fn default() -> Self {
//        Self {
//            resource_name: "undefined".to_owned(),
//            reqwest_executor: Arc::new(NewClientPerRequestExecutor),
//            actions: Default::default(),
//            deserialize_read_many_response: Default::default(),
//            //entity_actions: Default::default(),
//            //create_field_select_config: Default::default(),
//            //read_field_select_config: Default::default(),
//            //update_field_select_config: Default::default(),
//            //custom_read_fields: Default::default(),
//            //custom_create_fields: Default::default(),
//            //custom_update_fields: Default::default(),
//        }
//    }
//}

impl Default for CrudInstanceConfig {
    fn default() -> Self {
        Self {
            api_base_url: "".to_owned(),
            view: SerializableCrudView::List,
            // headers: vec![( // TODO: build from id fields_iter
            //     T::ReadModel::get_id_field(),
            //     HeaderOptions {
            //         display_name: "ID".to_owned(),
            //         min_width: true,
            //         ordering_allowed: true,
            //         date_time_display: DateTimeDisplay::LocalizedLocal,
            //     },
            // )],
            headers: vec![],
            create_elements: CreateElements::None,
            elements: vec![],
            // order_by: indexmap! { // TODO: Nothing? First id field? All id fields?
            //     T::ReadModel::get_id_field() => Order::Asc,
            // },
            order_by: indexmap! {},
            items_per_page: 10,
            page: 1,
            active_tab: None,
            base_condition: None,
        }
    }
}

impl CrudInstanceConfig {
    // TODO: unused?
    pub fn update_order_by(&mut self, field: AnyField, options: OrderByUpdateOptions) {
        let prev = self.order_by.get(&field).cloned();
        if !options.append {
            self.order_by.clear();
        }
        self.order_by.insert(
            field,
            match prev {
                Some(order) => match order {
                    Order::Asc => Order::Desc,
                    Order::Desc => Order::Asc,
                },
                None => Order::Asc,
            },
        );
    }
}
