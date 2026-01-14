use crate::generic::crud_action::{CrudAction, CrudEntityAction};
use crate::generic::custom_field::{CustomCreateFields, CustomReadFields, CustomUpdateFields};
use crate::shared::crud_instance_config::{ItemsPerPage, PageNr};
use crudkit_condition::Condition;
use crudkit_core::Order;
use crudkit_web::generic::prelude::*;
use crudkit_web::reqwest_executor::NewClientPerRequestExecutor;
use indexmap::{IndexMap, indexmap};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrudInstanceConfig<T: CrudMainTrait> {
    pub api_base_url: String,
    pub view: CrudView<T::ReadModelId, T::UpdateModelId>,
    pub headers: Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions)>,
    // serde bound used as described in: https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    pub create_elements: CreateElements<T>,
    // serde bound used as described in: https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    pub elements: Vec<Elem<T::UpdateModel>>,
    pub order_by: IndexMap<<T::ReadModel as CrudDataTrait>::Field, Order>,
    pub items_per_page: ItemsPerPage,
    pub page_nr: PageNr,
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
pub enum CreateElements<T: CrudMainTrait> {
    None,
    Custom(Vec<Elem<T::CreateModel>>),
}

/// This config is non-serializable. Every piece of runtime-changing data relevant to be tracked and reloaded should be part of the CrudInstanceConfig struct.
#[derive(Debug, Clone)]
pub struct CrudStaticInstanceConfig<T: CrudMainTrait + 'static> {
    pub reqwest_executor: Arc<dyn ReqwestExecutor>,
    pub actions: Vec<CrudAction<T>>,
    pub entity_actions: Vec<CrudEntityAction<T>>,
    pub custom_read_fields: CustomReadFields<T>,
    pub custom_create_fields: CustomCreateFields<T>,
    pub custom_update_fields: CustomUpdateFields<T>,
}

impl<T: CrudMainTrait> Default for CrudStaticInstanceConfig<T> {
    fn default() -> Self {
        Self {
            reqwest_executor: Arc::new(NewClientPerRequestExecutor),
            actions: Default::default(),
            entity_actions: Default::default(),
            custom_read_fields: Default::default(),
            custom_create_fields: Default::default(),
            custom_update_fields: Default::default(),
        }
    }
}

impl<T: CrudMainTrait> Default for CrudInstanceConfig<T> {
    fn default() -> Self {
        Self {
            api_base_url: "".to_owned(),
            view: CrudView::default(),
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
            items_per_page: ItemsPerPage::default(),
            page_nr: PageNr::first(),
            base_condition: None,
        }
    }
}
