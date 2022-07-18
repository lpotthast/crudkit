use crud_shared_types::{ConditionClauseValue, Value};
use sea_orm::{ColumnTrait, ActiveModelTrait};
use std::str::FromStr;

pub mod axum_routes;
pub mod context;
pub mod controller;
pub mod create;
pub mod delete;
pub mod query;
pub mod read;
pub mod resource;
pub mod update;
pub mod validate;
pub mod validation;

pub mod prelude {
    pub use super::context::CrudContext;
    pub use super::controller::CrudController;
    pub use super::resource::CrudResource;

    pub use super::AsColType;
    pub use super::CreateModelTrait;
    pub use super::CrudColumns;
    pub use super::ExcludingColumnsOnInsert;
    pub use super::MaybeColumnTrait;
    pub use super::UpdateActiveModelTrait;

    pub use super::validation::ValidationViolationType;
    pub use super::validation::ValidationViolationTypeExt;
    pub use super::validation::ValidationResultSaverTrait;
    pub use super::validation::EntityValidationsExt;
    pub use super::validation::EntityValidatorsTrait;
    pub use super::validation::EntityValidatorTrait;

    pub use super::validate::validate_max_length;
    pub use super::validate::validate_min_length;
    pub use super::validate::validate_required;

    pub use super::parse;

    pub use super::query::prune_active_model;
    pub use super::query::build_delete_many_query;
    pub use super::query::build_insert_query;
    pub use super::query::build_select_query;

    pub use super::create::create_one;
    pub use super::create::CreateOne;
    pub use super::delete::delete_by_id;
    pub use super::delete::delete_many;
    pub use super::delete::delete_one;
    pub use super::delete::DeleteById;
    pub use super::delete::DeleteMany;
    pub use super::delete::DeleteOne;
    pub use super::read::read_count;
    pub use super::read::read_many;
    pub use super::read::read_one;
    pub use super::read::ReadCount;
    pub use super::read::ReadMany;
    pub use super::read::ReadOne;
    pub use super::update::update_one;
    pub use super::update::UpdateOne;
}

pub trait CrudColumns<C: ColumnTrait, A: ActiveModelTrait> {
    fn to_sea_orm_column(&self) -> C;
    fn get_id(model: &A) -> Option<i32>;
    fn get_id_field() -> C;
    fn get_id_field_name() -> String;
}

pub trait CreateModelTrait {}

pub trait UpdateActiveModelTrait<C: CreateModelTrait> {
    fn update_with(&mut self, update: C);
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
