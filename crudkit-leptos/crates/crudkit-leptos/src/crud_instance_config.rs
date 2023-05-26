use crudkit_shared::Order;
use crudkit_web::prelude::*;
use indexmap::{indexmap, IndexMap};
use serde::{Deserialize, Serialize};

use crate::crud_action::{CrudAction, CrudEntityAction};

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
    pub items_per_page: u64,
    pub page: u64,
    pub active_tab: Option<Label>,
    pub nested: Option<NestedConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NestedConfig {
    /// The name of the parent instance from which the referenced id should be loaded.
    pub parent_instance: String,

    /// The field of the parent instance from which the referenced id should be loaded. For example: "id".
    pub parent_field: String,

    /// The `own` field in which the reference is stored. For example: "server_id", when referencing the Server entity.
    pub reference_field: String, // TODO: This should be: T::ReadModel::Field? (ClusterCertificateField::CreatedAt)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CreateElements<T: CrudMainTrait> {
    None,
    Custom(Vec<Elem<T::CreateModel>>),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct CrudStaticInstanceConfig<T: CrudMainTrait> {
    pub actions: Vec<CrudAction<T>>,
    pub entity_actions: Vec<CrudEntityAction<T>>,
    pub custom_read_fields: CustomReadFields<T, leptos::View>,
    pub custom_create_fields: CustomCreateFields<T, leptos::View>,
    pub custom_update_fields: CustomUpdateFields<T, leptos::View>,
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
            items_per_page: 10,
            page: 1,
            active_tab: None,
            nested: None,
        }
    }
}

impl<T: CrudMainTrait> CrudInstanceConfig<T> {
    pub fn update_order_by(
        &mut self,
        field: <T::ReadModel as CrudDataTrait>::Field,
        options: OrderByUpdateOptions,
    ) {
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
