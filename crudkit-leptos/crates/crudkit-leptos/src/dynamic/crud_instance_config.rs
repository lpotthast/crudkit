use crate::dynamic::crud_action::{CrudAction, CrudEntityAction};
use crate::dynamic::custom_field::{CustomCreateFields, CustomReadFields, CustomUpdateFields};
use crate::shared::crud_instance_config::{DynSelectConfig, ItemsPerPage, PageNr};
use crate::{IntoReactiveValue, ReactiveValue};
use crudkit_condition::Condition;
use crudkit_shared::{Order, SaveResult, Saved};
use crudkit_web::dynamic::prelude::*;
use crudkit_web::dynamic::{
    AnyCreateField, AnyCreateModel, AnyReadField, AnyReadModel, AnyUpdateField, AnyUpdateModel,
    CreateField, CreateModel, ReadField, ReadModel, UpdateField, UpdateModel,
};
use indexmap::IndexMap;
use leptos::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

/// Definition of a column, shown in list view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub field: AnyReadField,
    pub options: HeaderOptions,
}

impl Header {
    pub fn showing(field: impl ReadField, options: HeaderOptions) -> Header {
        Self {
            field: AnyReadField::new(field),
            options,
        }
    }
}

impl From<(AnyReadField, HeaderOptions)> for Header {
    fn from((field, options): (AnyReadField, HeaderOptions)) -> Self {
        Self { field, options }
    }
}

#[derive(Debug, Clone)]
pub struct CrudInstanceConfig {
    /* Later to-be mutable data. */
    pub api_base_url: String,
    pub view: SerializableCrudView,
    pub list_columns: Vec<Header>,
    pub create_elements: CreateElements,
    pub elements: UpdateElements,
    pub order_by: IndexMap<AnyReadField, Order>,
    /// The number of items shown per page in the list view.
    pub items_per_page: ItemsPerPage,
    /// The current page to display, e.g. `Page::first()`. One-based index.
    pub page_nr: PageNr,
    /// The active tab. If set, must reference a tab name chosen when defining `elements`.
    pub active_tab: Option<Label>,
    pub base_condition: Option<Condition>,

    /* Immutable data */
    pub resource_name: String,
    pub reqwest_executor: Arc<dyn ReqwestExecutor>,
    pub model_handler: ModelHandler,
    pub actions: Vec<CrudAction>,
    pub entity_actions: Vec<CrudEntityAction>,
    pub create_field_select_config: HashMap<AnyCreateField, DynSelectConfig>,
    pub read_field_select_config: HashMap<AnyReadField, DynSelectConfig>,
    pub update_field_select_config: HashMap<AnyUpdateField, DynSelectConfig>,
    pub custom_read_fields: CustomReadFields,
    pub custom_create_fields: CustomCreateFields,
    pub custom_update_fields: CustomUpdateFields,
}

impl CrudInstanceConfig {
    pub(crate) fn split(self) -> (CrudMutableInstanceConfig, CrudStaticInstanceConfig) {
        (
            CrudMutableInstanceConfig {
                api_base_url: self.api_base_url,
                view: self.view,
                headers: self.list_columns,
                create_elements: self.create_elements,
                elements: self.elements,
                order_by: self.order_by,
                items_per_page: self.items_per_page,
                page: self.page_nr,
                active_tab: self.active_tab,
                base_condition: self.base_condition,
            },
            CrudStaticInstanceConfig {
                resource_name: self.resource_name,
                reqwest_executor: self.reqwest_executor,
                model_handler: self.model_handler,
                actions: self.actions,
                entity_actions: self.entity_actions,
                create_field_select_config: self.create_field_select_config,
                read_field_select_config: self.read_field_select_config,
                update_field_select_config: self.update_field_select_config,
                custom_read_fields: self.custom_read_fields,
                custom_create_fields: self.custom_create_fields,
                custom_update_fields: self.custom_update_fields,
            },
        )
    }
}

#[derive(Debug, Clone, PartialEq)] // TODO: Serialize, Deserialize
pub(crate) struct CrudMutableInstanceConfig {
    pub api_base_url: String,
    pub view: SerializableCrudView,
    pub headers: Vec<Header>,
    pub create_elements: CreateElements,
    pub elements: UpdateElements,
    pub order_by: IndexMap<AnyReadField, Order>,
    pub items_per_page: ItemsPerPage,
    pub page: PageNr,
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

pub type UpdateElements = Vec<Elem<AnyUpdateField>>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CreateElements {
    None,
    Custom(Vec<Elem<AnyCreateField>>),
}

#[derive(Debug, Clone)]
pub struct ModelHandler {
    pub deserialize_read_many_response:
        Callback<serde_json::Value, Result<Vec<AnyReadModel>, serde_json::Error>>,
    pub deserialize_read_one_response:
        Callback<serde_json::Value, Result<Option<AnyReadModel>, serde_json::Error>>,
    pub deserialize_create_one_response:
        Callback<serde_json::Value, Result<SaveResult<AnyUpdateModel>, serde_json::Error>>,
    pub deserialize_update_one_response:
        Callback<serde_json::Value, Result<SaveResult<AnyUpdateModel>, serde_json::Error>>,

    pub read_model_to_update_model: Callback<AnyReadModel, AnyUpdateModel>,
    pub create_model_to_signal_map:
        Callback<AnyCreateModel, HashMap<AnyCreateField, ReactiveValue>>,
    pub read_model_to_signal_map: Callback<AnyReadModel, HashMap<AnyReadField, ReactiveValue>>,
    pub update_model_to_signal_map:
        Callback<AnyUpdateModel, HashMap<AnyUpdateField, ReactiveValue>>,
    pub get_create_model_field: Callback<String, AnyCreateField>,
    pub get_default_create_model: Callback<(), AnyCreateModel>,
}

impl ModelHandler {
    pub fn new<Create, Read, Update>() -> ModelHandler
    where
        Create: CreateModel + DeserializeOwned + CrudDataTrait + Default,
        Read: ReadModel + DeserializeOwned + CrudDataTrait,
        Update: UpdateModel + DeserializeOwned + CrudDataTrait + From<Read>,
        <Read as CrudDataTrait>::Field: ReadField,
        <Create as CrudDataTrait>::Field: CreateField,
        <Update as CrudDataTrait>::Field: UpdateField,
    {
        ModelHandler {
            deserialize_read_many_response: Callback::new(move |json| {
                Ok(serde_json::from_value::<Vec<Read>>(json)?
                    .into_iter()
                    .map(AnyReadModel::from)
                    .collect::<Vec<AnyReadModel>>())
            }),
            deserialize_read_one_response: Callback::new(move |json| {
                Ok(serde_json::from_value::<Option<Read>>(json)?.map(AnyReadModel::from))
            }),
            deserialize_create_one_response: Callback::new(move |json| {
                let result: SaveResult<Update> = serde_json::from_value(json)?;
                let result: SaveResult<AnyUpdateModel> = match result {
                    SaveResult::Saved(saved) => SaveResult::Saved(Saved {
                        entity: AnyUpdateModel::from(saved.entity),
                        with_validation_errors: saved.with_validation_errors,
                    }),
                    SaveResult::Aborted { reason } => SaveResult::Aborted { reason },
                    SaveResult::CriticalValidationErrors => SaveResult::CriticalValidationErrors,
                };
                Ok(result)
            }),
            deserialize_update_one_response: Callback::new(move |json| {
                let result: SaveResult<Update> = serde_json::from_value(json)?;
                let result: SaveResult<AnyUpdateModel> = match result {
                    SaveResult::Saved(saved) => SaveResult::Saved(Saved {
                        entity: AnyUpdateModel::from(saved.entity),
                        with_validation_errors: saved.with_validation_errors,
                    }),
                    SaveResult::Aborted { reason } => SaveResult::Aborted { reason },
                    SaveResult::CriticalValidationErrors => SaveResult::CriticalValidationErrors,
                };
                Ok(result)
            }),
            read_model_to_update_model: Callback::new(move |read_model: AnyReadModel| {
                AnyUpdateModel::from(Update::from(read_model.downcast::<Read>()))
            }),
            create_model_to_signal_map: Callback::new(move |create_model: AnyCreateModel| {
                let create_model: &Create = create_model.downcast_ref::<Create>();
                let mut map: HashMap<AnyCreateField, ReactiveValue> = HashMap::new();
                for field in Create::get_all_fields() {
                    let initial = CrudFieldValueTrait::get_value(&field, create_model);
                    map.insert(AnyCreateField::from(field), initial.into_reactive_value());
                }
                map
            }),
            read_model_to_signal_map: Callback::new(move |read_model: AnyReadModel| {
                let read_model: &Read = read_model.downcast_ref::<Read>();
                let mut map: HashMap<AnyReadField, ReactiveValue> = HashMap::new();
                for field in Read::get_all_fields() {
                    let initial = CrudFieldValueTrait::get_value(&field, read_model);
                    map.insert(AnyReadField::from(field), initial.into_reactive_value());
                }
                map
            }),
            update_model_to_signal_map: Callback::new(move |update_model: AnyUpdateModel| {
                let update_model: &Update = update_model.downcast_ref::<Update>();
                let mut map: HashMap<AnyUpdateField, ReactiveValue> = HashMap::new();
                for field in Update::get_all_fields() {
                    let initial = CrudFieldValueTrait::get_value(&field, update_model);
                    map.insert(AnyUpdateField::from(field), initial.into_reactive_value());
                }
                map
            }),
            get_create_model_field: Callback::new(move |field_name: String| {
                AnyCreateField::from(Create::get_field(&field_name))
            }),
            get_default_create_model: Callback::new(move |()| {
                AnyCreateModel::from(Create::default())
            }),
        }
    }
}

/// This config is non-serializable. Every piece of runtime-changing data relevant to be tracked and reloaded should be part of the CrudInstanceConfig struct.
#[derive(Debug, Clone)]
pub(crate) struct CrudStaticInstanceConfig {
    pub resource_name: String,
    pub reqwest_executor: Arc<dyn ReqwestExecutor>,
    pub model_handler: ModelHandler,
    pub actions: Vec<CrudAction>,
    pub entity_actions: Vec<CrudEntityAction>,
    pub create_field_select_config: HashMap<AnyCreateField, DynSelectConfig>,
    pub read_field_select_config: HashMap<AnyReadField, DynSelectConfig>,
    pub update_field_select_config: HashMap<AnyUpdateField, DynSelectConfig>,
    pub custom_read_fields: CustomReadFields,
    pub custom_create_fields: CustomCreateFields,
    pub custom_update_fields: CustomUpdateFields,
}

impl CrudMutableInstanceConfig {
    // TODO: unused?
    pub fn update_order_by(&mut self, field: AnyReadField, options: OrderByUpdateOptions) {
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
