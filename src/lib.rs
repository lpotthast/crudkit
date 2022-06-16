use crud_shared_types::{ConditionClauseValue, ConditionElement, Order, Value};
use indexmap::IndexMap;
use sea_orm::{ColumnTrait, DatabaseConnection};
use serde::Deserialize;
use std::{str::FromStr, sync::Arc};

mod create;
mod delete;
mod query;
mod read;
mod resource;
mod update;

pub mod axum_routes;
pub mod validate;

pub use resource::CrudResource;

pub struct CrudController {
    db: Arc<DatabaseConnection>,
}

impl CrudController {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[derive(Deserialize)]
pub struct ReadCount {
    pub condition: Option<Vec<ConditionElement>>,
}

#[derive(Deserialize)]
pub struct ReadOne<R: CrudResource> {
    pub skip: Option<u64>,
    #[serde(bound = "")]
    pub order_by: Option<IndexMap<R::CrudColumn, Order>>,
    pub condition: Option<Vec<ConditionElement>>,
}

#[derive(Deserialize)]
pub struct ReadMany<R: CrudResource> {
    pub limit: Option<u64>,
    pub skip: Option<u64>,
    #[serde(bound = "")]
    pub order_by: Option<IndexMap<R::CrudColumn, Order>>,
    pub condition: Option<Vec<ConditionElement>>,
}

#[derive(Deserialize)]
pub struct CreateOne {
    pub entity: Box<serde_json::value::RawValue>,
}

#[derive(Deserialize)]
pub struct UpdateOne {
    pub condition: Option<Vec<ConditionElement>>,
    pub entity: Box<serde_json::value::RawValue>,
}

#[derive(Deserialize)]
pub struct DeleteById {
    pub id: u32,
}

#[derive(Deserialize)]
pub struct DeleteOne<R: CrudResource> {
    pub skip: Option<u64>,
    pub order_by: Option<IndexMap<R::CrudColumn, Order>>,
    pub condition: Option<Vec<ConditionElement>>,
}

#[derive(Deserialize)]
pub struct DeleteMany {
    pub condition: Option<Vec<ConditionElement>>,
}

pub trait CrudColumns<C: ColumnTrait> {
    fn to_sea_orm_column(&self) -> C;
    fn get_id_field() -> C;
    fn get_id_field_name() -> String;
}

pub trait CreateModelTrait {}

pub trait UpdateActiveModelTrait<C: CreateModelTrait> {
    fn update_with(&mut self, update: C);
}

pub trait FieldValidatorTrait {
    fn validate(&self) -> Vec<String>;
}

pub trait AsColType {
    fn as_col_type(&self, string: ConditionClauseValue) -> Result<Value, String>;
}

pub fn parse<T>(string: &str) -> Result<T, String>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    string.parse::<T>().map_err(|e| format!("{}", e))
}

/// This trait is used to convert from column names (for example parsed from a request) to actual entity columns.
pub trait MaybeColumnTrait {
    type Column: ColumnTrait + AsColType;
    fn get_col(name: &str) -> Option<Self::Column>;
}

pub trait ExcludingColumnsOnInsert<C: ColumnTrait> {
    fn excluding_columns() -> &'static [C];
}
