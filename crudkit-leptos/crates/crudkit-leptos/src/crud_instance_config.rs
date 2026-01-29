use crate::crud_action::{CrudAction, CrudEntityAction};
use crate::fields::FieldRenderer;
use crate::{IntoReactiveValue, ReactiveValue};
use crudkit_core::condition::Condition;
use crudkit_core::{Order, Saved};
use crudkit_web::prelude::*;
use crudkit_web::reqwest_executor::ReqwestExecutor;
use crudkit_web::view::SerializableCrudView;
use crudkit_web::{CrudFieldValueTrait, CrudModel, HeaderOptions};
use indexmap::IndexMap;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
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
    pub field: DynReadField,
    pub options: HeaderOptions,
}

impl Header {
    pub fn showing(field: impl ErasedReadField, options: HeaderOptions) -> Header {
        Self {
            field: DynReadField::new(field),
            options,
        }
    }
}

impl From<(DynReadField, HeaderOptions)> for Header {
    fn from((field, options): (DynReadField, HeaderOptions)) -> Self {
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
pub struct FieldRendererRegistry<F: TypeErasedField> {
    pub(crate) reg: HashMap<F, FieldRenderer<F>>,
}

impl<F: TypeErasedField> FieldRendererRegistry<F> {
    pub fn builder() -> FieldRendererRegistryBuilder<F> {
        FieldRendererRegistryBuilder::new()
    }
}

#[derive(Debug)]
pub struct FieldRendererRegistryBuilder<F: TypeErasedField> {
    reg: HashMap<F, FieldRenderer<F>>,
}

impl<F: TypeErasedField> FieldRendererRegistryBuilder<F> {
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
    pub order_by: IndexMap<DynReadField, Order>,
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
    pub read_field_renderer: FieldRendererRegistry<DynReadField>,
    pub create_field_renderer: FieldRendererRegistry<DynCreateField>,
    pub update_field_renderer: FieldRendererRegistry<DynUpdateField>,
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
    pub order_by: IndexMap<DynReadField, Order>,
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
    pub read_field_renderer: FieldRendererRegistry<DynReadField>,
    pub create_field_renderer: FieldRendererRegistry<DynCreateField>,
    pub update_field_renderer: FieldRendererRegistry<DynUpdateField>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrudParentConfig {
    /// The name of the parent instance from which the referenced id should be loaded.
    pub name: &'static str,

    /// The field of the parent instance from which the referenced id should be loaded. For example: "id".
    pub referenced_field: Cow<'static, str>,

    /// The `own` field in which the reference is stored. For example: "user_id", when referencing a User entity.
    pub referencing_field: Cow<'static, str>, // TODO: This should be: T::ReadModel::Field? (ClusterCertificateField::CreatedAt)
}

pub type UpdateElements = Vec<Elem<DynUpdateField>>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CreateElements {
    None,
    Custom(Vec<Elem<DynCreateField>>),
}

#[derive(Debug, Clone)]
pub struct ModelHandler {
    pub deserialize_read_many_response:
        Callback<serde_json::Value, Result<Vec<DynReadModel>, serde_json::Error>>,
    pub deserialize_read_one_response:
        Callback<serde_json::Value, Result<Option<DynReadModel>, serde_json::Error>>,
    pub deserialize_create_one_response:
        Callback<serde_json::Value, Result<Saved<DynUpdateModel>, serde_json::Error>>,
    pub deserialize_update_one_response:
        Callback<serde_json::Value, Result<Saved<DynUpdateModel>, serde_json::Error>>,

    pub read_model_to_update_model: Callback<DynReadModel, DynUpdateModel>,
    pub create_model_to_signal_map:
        Callback<DynCreateModel, HashMap<DynCreateField, ReactiveValue>>,
    pub read_model_to_signal_map: Callback<DynReadModel, HashMap<DynReadField, ReactiveValue>>,
    pub update_model_to_signal_map:
        Callback<DynUpdateModel, HashMap<DynUpdateField, ReactiveValue>>,
    pub get_create_model_field: Callback<Cow<'static, str>, DynCreateField>,
    pub get_default_create_model: Callback<(), DynCreateModel>,
}

impl ModelHandler {
    pub fn new<Create, Read, Update>() -> ModelHandler
    where
        Create: ErasedCreateModel + CrudModel + Default,
        Read: ErasedReadModel + CrudModel,
        Update: ErasedUpdateModel + CrudModel + From<Read>,
        <Read as CrudModel>::Field: ErasedReadField,
        <Create as CrudModel>::Field: ErasedCreateField,
        <Update as CrudModel>::Field: ErasedUpdateField,
    {
        let deserialize_update_model = Callback::new(move |json| {
            let saved: Saved<Update> = serde_json::from_value(json)?;
            Ok(Saved {
                entity: DynUpdateModel::from(saved.entity),
                violations: saved.violations,
            })
        });

        ModelHandler {
            deserialize_read_many_response: Callback::new(move |json| {
                Ok(serde_json::from_value::<Vec<Read>>(json)?
                    .into_iter()
                    .map(DynReadModel::from)
                    .collect::<Vec<DynReadModel>>())
            }),
            deserialize_read_one_response: Callback::new(move |json| {
                Ok(serde_json::from_value::<Option<Read>>(json)?.map(DynReadModel::from))
            }),
            deserialize_create_one_response: deserialize_update_model,
            deserialize_update_one_response: deserialize_update_model,
            read_model_to_update_model: Callback::new(move |read_model: DynReadModel| {
                DynUpdateModel::from(Update::from(read_model.downcast::<Read>()))
            }),
            create_model_to_signal_map: Callback::new(move |create_model: DynCreateModel| {
                let create_model: &Create = create_model.downcast_ref::<Create>();
                let mut map: HashMap<DynCreateField, ReactiveValue> = HashMap::new();
                for field in Create::all_fields() {
                    let initial = CrudFieldValueTrait::value(&field, create_model);
                    map.insert(DynCreateField::from(field), initial.into_reactive_value());
                }
                map
            }),
            read_model_to_signal_map: Callback::new(move |read_model: DynReadModel| {
                let read_model: &Read = read_model.downcast_ref::<Read>();
                let mut map: HashMap<DynReadField, ReactiveValue> = HashMap::new();
                for field in Read::all_fields() {
                    let initial = CrudFieldValueTrait::value(&field, read_model);
                    map.insert(DynReadField::from(field), initial.into_reactive_value());
                }
                map
            }),
            update_model_to_signal_map: Callback::new(move |update_model: DynUpdateModel| {
                let update_model: &Update = update_model.downcast_ref::<Update>();
                let mut map: HashMap<DynUpdateField, ReactiveValue> = HashMap::new();
                for field in Update::all_fields() {
                    let initial = CrudFieldValueTrait::value(&field, update_model);
                    map.insert(DynUpdateField::from(field), initial.into_reactive_value());
                }
                map
            }),
            get_create_model_field: Callback::new(move |field_name: Cow<'static, str>| {
                DynCreateField::from(Create::field(field_name.as_ref()))
            }),
            get_default_create_model: Callback::new(move |()| {
                DynCreateModel::from(Create::default())
            }),
        }
    }
}
