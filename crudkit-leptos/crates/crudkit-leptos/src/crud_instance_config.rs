use crate::crud_action::{CrudAction, CrudEntityAction};
use crate::fields::FieldRenderer;
use crate::{IntoReactiveValue, ReactiveValue};
use crudkit_condition::Condition;
use crudkit_core::{Order, Saved};
use crudkit_web::dynamic::prelude::*;
use crudkit_web::dynamic::{
    AnyCreateField, AnyCreateModel, AnyReadField, AnyReadModel, AnyUpdateField, AnyUpdateModel,
    CreateField, CreateModel, ReadField, ReadModel, UpdateField, UpdateModel,
};
use indexmap::IndexMap;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ItemsPerPage(pub u64);

impl Default for ItemsPerPage {
    fn default() -> Self {
        Self(10)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PageNr(pub u64); // TODO: NonZero type?

impl Default for PageNr {
    fn default() -> Self {
        Self::first()
    }
}

impl PageNr {
    pub fn first() -> Self {
        Self(1)
    }
}

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

/// We determine the `View` shown for a field using the following sources.
///
/// 1. If the layout, specifying where exactly a field should be rendered, also defines a renderer
///    for the field at this specific position, that renderer is used.
///
/// 2. If no specific renderer was specified through the layout, we look if the instance was
///    configured with a specific renderer that should always be used for this field.
///
/// 3. If the instance has no field renderer overrides, we pick a default renderer based on the
///    fields type.
///
/// 4. If the fields type is something we cannot show a default renderer for, an error is rendered
///    and logged, showing that the instance needs further configuration.
///
/// This registry allows to specify (2).
#[derive(Debug, Clone)]
pub struct FieldRendererRegistry<F: DynField> {
    pub(crate) reg: HashMap<F, FieldRenderer<F>>,
}

impl<F: DynField> FieldRendererRegistry<F> {
    pub fn builder() -> FieldRendererRegistryBuilder<F> {
        FieldRendererRegistryBuilder::new()
    }
}

#[derive(Debug)]
pub struct FieldRendererRegistryBuilder<F: DynField> {
    reg: HashMap<F, FieldRenderer<F>>,
}

impl<F: DynField> FieldRendererRegistryBuilder<F> {
    fn new() -> Self {
        Self {
            reg: HashMap::new(),
        }
    }

    pub fn register(mut self, field: impl Into<F>, renderer: FieldRenderer<F>) -> Self {
        self.reg.insert(field.into(), renderer);
        self
    }

    pub fn build(self) -> FieldRendererRegistry<F> {
        FieldRendererRegistry { reg: self.reg }
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
    pub base_condition: Option<Condition>,

    /* Immutable data */
    pub resource_name: String,
    pub reqwest_executor: Arc<dyn ReqwestExecutor>,
    pub model_handler: ModelHandler,
    pub actions: Vec<CrudAction>,
    pub entity_actions: Vec<CrudEntityAction>,
    pub read_field_renderer: FieldRendererRegistry<AnyReadField>,
    pub create_field_renderer: FieldRendererRegistry<AnyCreateField>,
    pub update_field_renderer: FieldRendererRegistry<AnyUpdateField>,
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
                base_condition: self.base_condition,
            },
            CrudStaticInstanceConfig {
                resource_name: self.resource_name,
                reqwest_executor: self.reqwest_executor,
                model_handler: self.model_handler,
                actions: self.actions,
                entity_actions: self.entity_actions,
                read_field_renderer: self.read_field_renderer,
                create_field_renderer: self.create_field_renderer,
                update_field_renderer: self.update_field_renderer,
            },
        )
    }
}

#[derive(Debug, Clone)] // TODO: Serialize, Deserialize
pub(crate) struct CrudMutableInstanceConfig {
    pub api_base_url: String,
    pub view: SerializableCrudView,
    pub headers: Vec<Header>,
    pub create_elements: CreateElements,
    pub elements: UpdateElements,
    pub order_by: IndexMap<AnyReadField, Order>,
    pub items_per_page: ItemsPerPage,
    pub page: PageNr,
    pub base_condition: Option<Condition>,
}

/// This config is non-serializable. Every piece of runtime-changing data relevant to be tracked and reloaded should be part of the CrudInstanceConfig struct.
#[derive(Debug, Clone)]
pub(crate) struct CrudStaticInstanceConfig {
    pub resource_name: String,
    pub reqwest_executor: Arc<dyn ReqwestExecutor>,
    pub model_handler: ModelHandler,
    pub actions: Vec<CrudAction>,
    pub entity_actions: Vec<CrudEntityAction>,
    pub read_field_renderer: FieldRendererRegistry<AnyReadField>,
    pub create_field_renderer: FieldRendererRegistry<AnyCreateField>,
    pub update_field_renderer: FieldRendererRegistry<AnyUpdateField>,
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
        Callback<serde_json::Value, Result<Saved<AnyUpdateModel>, serde_json::Error>>,
    pub deserialize_update_one_response:
        Callback<serde_json::Value, Result<Saved<AnyUpdateModel>, serde_json::Error>>,

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
        Create: CreateModel + CrudDataTrait + Default,
        Read: ReadModel + CrudDataTrait,
        Update: UpdateModel + CrudDataTrait + From<Read>,
        <Read as CrudDataTrait>::Field: ReadField,
        <Create as CrudDataTrait>::Field: CreateField,
        <Update as CrudDataTrait>::Field: UpdateField,
    {
        let deserialize_update_model = Callback::new(move |json| {
            let saved: Saved<Update> = serde_json::from_value(json)?;
            Ok(Saved {
                entity: AnyUpdateModel::from(saved.entity),
                with_validation_errors: saved.with_validation_errors,
            })
        });

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
            deserialize_create_one_response: deserialize_update_model,
            deserialize_update_one_response: deserialize_update_model,
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
