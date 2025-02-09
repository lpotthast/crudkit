use crate::{CrudDataTrait, FieldOptions};
use serde::{Deserialize, Serialize};

pub mod crud_rest_data_provider;
pub mod requests;

pub mod prelude {
    pub use crate::prelude::*;

    pub use super::Elem;
    pub use super::Enclosing;
    pub use super::Group;
    pub use super::Tab;

    pub use super::crud_rest_data_provider::CreateOne;
    pub use super::crud_rest_data_provider::CrudRestDataProvider;
    pub use super::crud_rest_data_provider::DeleteById;
    pub use super::crud_rest_data_provider::ReadCount;
    pub use super::crud_rest_data_provider::ReadMany;
    pub use super::crud_rest_data_provider::ReadOne;
    pub use super::crud_rest_data_provider::UpdateOne;

    pub use super::requests::request;
    pub use super::requests::request_delete;
    pub use super::requests::request_get;
    pub use super::requests::request_post;
    pub use super::requests::request_put;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Elem<T: CrudDataTrait> {
    // serde bound used as described in: https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    Enclosing(Enclosing<T>),
    Field((T::Field, FieldOptions)),
    Separator,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Enclosing<T: CrudDataTrait> {
    #[serde(bound = "")]
    None(Group<T>),
    #[serde(bound = "")]
    Tabs(Vec<Tab<T>>),
    #[serde(bound = "")]
    Card(Group<T>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tab<T: CrudDataTrait> {
    /// A unique identifier for this tab.
    pub id: crate::TabId,
    pub label: crate::Label,
    #[serde(bound = "")]
    pub group: Group<T>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group<T: CrudDataTrait> {
    pub layout: crate::Layout,
    // serde bound used as described in: https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    pub children: Vec<Elem<T>>,
}
