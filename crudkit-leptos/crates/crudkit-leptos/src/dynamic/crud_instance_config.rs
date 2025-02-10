use crate::dynamic::crud_action::{CrudAction, CrudEntityAction};
use crate::dynamic::custom_field::{CustomCreateFields, CustomReadFields, CustomUpdateFields};
use crate::shared::crud_instance_config::DynSelectConfig;
use crate::{IntoReactiveValue, ReactiveValue};
use crudkit_condition::Condition;
use crudkit_shared::{Order, SaveResult, Saved};
use crudkit_web::dynamic::prelude::*;
use indexmap::{indexmap, IndexMap};
use leptos::prelude::*;
use serde::de::DeserializeOwned;
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

#[derive(Debug, Clone)]
pub struct ModelHandler {
    pub deserialize_read_many_response:
        Callback<(serde_json::Value,), Result<Vec<AnyModel>, serde_json::Error>>,
    pub deserialize_read_one_response:
        Callback<(serde_json::Value,), Result<Option<AnyModel>, serde_json::Error>>,
    pub deserialize_create_one_response:
        Callback<(serde_json::Value,), Result<SaveResult<AnyModel>, serde_json::Error>>,
    pub deserialize_update_one_response:
        Callback<(serde_json::Value,), Result<SaveResult<AnyModel>, serde_json::Error>>,

    pub read_model_to_update_model: Callback<(AnyModel,), AnyModel>,
    pub create_model_to_signal_map: Callback<(AnyModel,), HashMap<AnyField, ReactiveValue>>,
    pub read_model_to_signal_map: Callback<(AnyModel,), HashMap<AnyField, ReactiveValue>>,
    pub update_model_to_signal_map: Callback<(AnyModel,), HashMap<AnyField, ReactiveValue>>,
    pub get_create_model_field: Callback<(String,), AnyField>,
    pub get_default_create_model: Callback<(), AnyModel>,
}

impl ModelHandler {
    pub fn new<Create, Read, Update>() -> ModelHandler
    where
        Create: Model + DeserializeOwned + CrudDataTrait + Default,
        Read: Model + DeserializeOwned + CrudDataTrait,
        Update: Model + DeserializeOwned + CrudDataTrait + From<Read>,
        <Read as CrudDataTrait>::Field: Field,
        <Create as CrudDataTrait>::Field: Field,
        <Update as CrudDataTrait>::Field: Field,
    {
        ModelHandler {
            deserialize_read_many_response: Callback::from(move |json| {
                Ok(serde_json::from_value::<Vec<Read>>(json)?
                    .into_iter()
                    .map(AnyModel::from)
                    .collect::<Vec<AnyModel>>())
            }),
            deserialize_read_one_response: Callback::from(move |json| {
                Ok(serde_json::from_value::<Option<Read>>(json)?.map(AnyModel::from))
            }),
            deserialize_create_one_response: Callback::from(move |json| {
                let result: SaveResult<Update> = serde_json::from_value(json)?;
                let result: SaveResult<AnyModel> = match result {
                    SaveResult::Saved(saved) => SaveResult::Saved(Saved {
                        entity: AnyModel::from(saved.entity),
                        with_validation_errors: saved.with_validation_errors,
                    }),
                    SaveResult::Aborted { reason } => SaveResult::Aborted { reason },
                    SaveResult::CriticalValidationErrors => SaveResult::CriticalValidationErrors,
                };
                Ok(result)
            }),
            deserialize_update_one_response: Callback::from(move |json| {
                let result: SaveResult<Update> = serde_json::from_value(json)?;
                let result: SaveResult<AnyModel> = match result {
                    SaveResult::Saved(saved) => SaveResult::Saved(Saved {
                        entity: AnyModel::from(saved.entity),
                        with_validation_errors: saved.with_validation_errors,
                    }),
                    SaveResult::Aborted { reason } => SaveResult::Aborted { reason },
                    SaveResult::CriticalValidationErrors => SaveResult::CriticalValidationErrors,
                };
                Ok(result)
            }),
            read_model_to_update_model: Callback::from(move |read_model: AnyModel| {
                AnyModel::from(Update::from(read_model.downcast::<Read>()))
            }),
            create_model_to_signal_map: Callback::from(move |create_model: AnyModel| {
                let create_model: &Create = create_model.downcast_ref::<Create>();
                let mut map: HashMap<AnyField, ReactiveValue> = HashMap::new();
                for field in Create::get_all_fields() {
                    let initial = CrudFieldValueTrait::get_value(&field, create_model);
                    map.insert(AnyField::from(field), initial.into_reactive_value());
                }
                map
            }),
            read_model_to_signal_map: Callback::from(move |read_model: AnyModel| {
                let read_model: &Read = read_model.downcast_ref::<Read>();
                let mut map: HashMap<AnyField, ReactiveValue> = HashMap::new();
                for field in Read::get_all_fields() {
                    let initial = CrudFieldValueTrait::get_value(&field, read_model);
                    map.insert(AnyField::from(field), initial.into_reactive_value());
                }
                map
            }),
            update_model_to_signal_map: Callback::from(move |update_model: AnyModel| {
                let update_model: &Update = update_model.downcast_ref::<Update>();
                let mut map: HashMap<AnyField, ReactiveValue> = HashMap::new();
                for field in Update::get_all_fields() {
                    let initial = CrudFieldValueTrait::get_value(&field, update_model);
                    map.insert(AnyField::from(field), initial.into_reactive_value());
                }
                map
            }),
            get_create_model_field: Callback::from(move |field_name: String| {
                AnyField::from(Create::get_field(&field_name))
            }),
            get_default_create_model: Callback::from(move || AnyModel::from(Create::default())),
        }
    }
}

/// This config is non-serializable. Every piece of runtime-changing data relevant to be tracked and reloaded should be part of the CrudInstanceConfig struct.
#[derive(Debug, Clone)]
pub struct CrudStaticInstanceConfig {
    pub resource_name: String,
    pub reqwest_executor: Arc<dyn ReqwestExecutor>,
    pub model_handler: ModelHandler,
    pub actions: Vec<CrudAction>,
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
