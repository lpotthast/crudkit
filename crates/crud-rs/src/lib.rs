#![forbid(unsafe_code)]

use crud_shared_types::{ConditionClauseValue, Value};
use prelude::{CrudContext, CrudResource};
use sea_orm::{ActiveModelTrait, ColumnTrait, ModelTrait};
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
    /* Provide convenient access to all our macros. */
    pub use derive_crud_columns::CrudColumns;

    pub use super::context::CrudContext;
    pub use super::controller::CrudController;
    pub use super::resource::CrudResource;

    pub use super::AsColType;
    pub use super::CreateModelTrait;
    pub use super::UpdateModelTrait;
    pub use super::CrudColumns;
    pub use super::ExcludingColumnsOnInsert;
    pub use super::MaybeColumnTrait;
    pub use super::UpdateActiveModelTrait;

    pub use super::validation::EntityValidationsExt;
    pub use super::validation::EntityValidatorTrait;
    pub use super::validation::EntityValidatorsTrait;
    pub use super::validation::ValidationResultSaverTrait;
    pub use super::validation::ValidationViolationType;
    pub use super::validation::ValidationViolationTypeExt;

    pub use super::validate::validate_max_length;
    pub use super::validate::validate_min_length;
    pub use super::validate::validate_required;

    pub use super::parse;
    pub use super::to_bool;
    pub use super::to_date_time;
    pub use super::to_i32;
    pub use super::to_string;

    pub use super::query::build_delete_many_query;
    pub use super::query::build_insert_query;
    pub use super::query::build_select_query;
    pub use super::query::prune_active_model;

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
pub trait UpdateModelTrait {}

// TODO: define and try to use instead of CreateModelTrait?
pub trait NewCreateModelTrait<R: CrudResource, M: ModelTrait> {
    fn to_model(&self, context: CrudContext<R>) -> M;
}

/// This trait must be implemented for sea-orm *ActiveModel types. It allows to update them with arbitrary types.
pub trait UpdateActiveModelTrait<C> {
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

pub fn to_i32(value: ConditionClauseValue) -> Result<Value, String> {
    match value {
        ConditionClauseValue::I32(num) => Ok(Value::I32(num)),
        ConditionClauseValue::String(string) => parse::<i32>(&string).map(Value::I32),
        _ => Err(format!(
            "{value:?} can not be converted to an i32. Expected i32 or String."
        )),
    }
}

pub fn to_bool(value: ConditionClauseValue) -> Result<Value, String> {
    match value {
        ConditionClauseValue::Bool(bool) => Ok(Value::Bool(bool)),
        ConditionClauseValue::String(string) => parse::<bool>(&string).map(Value::Bool),
        _ => Err(format!(
            "{value:?} can not be converted to a bool. Expected bool or String."
        )),
    }
}

pub fn to_string(value: ConditionClauseValue) -> Result<Value, String> {
    match value {
        ConditionClauseValue::String(string) => Ok(Value::String(string)),
        _ => Err(format!(
            "{value:?} can not be converted to a String. Expected String."
        )),
    }
}

pub fn to_date_time(value: ConditionClauseValue) -> Result<Value, String> {
    match value {
        ConditionClauseValue::String(string) => chrono::DateTime::parse_from_rfc3339(&string)
            .map_err(|e| format!("{}", e))
            .map(|value| value.naive_utc())
            .map(Value::DateTime),
        _ => Err(format!(
            "{value:?} can not be converted to a DateTime. Expected String."
        )),
    }
}