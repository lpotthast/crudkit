#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use async_trait::async_trait;
use chrono_utc_date_time::UtcDateTime;
use crud_shared_types::{condition::ConditionClauseValue, prelude::Id, Value};
use prelude::{CrudContext, CrudResource};
use sea_orm::{ActiveModelTrait, ColumnTrait, ModelTrait};
use std::str::FromStr;
use validation::PersistableViolation;

pub mod axum_routes;
pub mod context;
pub mod controller;
pub mod create;
pub mod delete;
pub mod error;
pub mod lifetime;
pub mod query;
pub mod read;
pub mod resource;
pub mod update;
pub mod validate;
pub mod validation;

pub mod prelude {
    /* Provide convenient access to all our macros. */
    pub use derive_create_model::CreateModel;
    pub use derive_crud_columns::CrudColumns;
    pub use derive_crud_resource_context::CrudResourceContext;
    pub use derive_update_model::UpdateModel;
    pub use derive_validation_model::ValidationModel;

    pub use super::error::CrudError;

    pub use super::context::CrudContext;
    pub use super::context::CrudResourceContext;
    pub use super::controller::CrudController;
    pub use super::resource::CrudResource;

    pub use super::AsColType;
    pub use super::CreateModelTrait;
    pub use super::CrudColumns;
    pub use super::MaybeColumnTrait;
    pub use super::UpdateActiveModelTrait;
    pub use super::UpdateModelTrait;

    pub use super::validation::AlwaysValidValidator;
    pub use super::validation::EntityValidationsExt;
    pub use super::validation::EntityValidatorTrait;
    pub use super::validation::EntityValidatorsTrait;
    pub use super::validation::ValidationResultSaverTrait;
    pub use super::validation::ValidationTrigger;
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

pub trait ValidatorModel<I: Id> {
    fn get_id(&self) -> I;

    fn get_validator_name(&self) -> String;
    fn get_validator_version(&self) -> i32;
}

pub trait ValidationColumns {
    fn get_validator_name_column() -> Self;
    fn get_validator_version_column() -> Self;
    fn get_violation_severity_column() -> Self;
}

pub trait IdColumns: Sized {
    fn get_id_columns() -> Vec<Self>;
}

pub trait NewActiveValidationModel<I: Id> {
    fn new(
        entity_id: I,
        validator_name: String,
        validator_version: i32,
        violation: PersistableViolation,
        now: UtcDateTime,
    ) -> Self;
}

pub trait CrudColumns<C: ColumnTrait, M: ModelTrait, A: ActiveModelTrait> {
    type Id: Id + Clone;

    fn to_sea_orm_column(&self) -> C;

    fn get_id(model: &M) -> Self::Id;
    fn get_id_active(model: &A) -> Result<Self::Id, String>;
}

pub trait GetIdFromModel {
    type Id: Id + Clone;

    fn get_id(&self) -> Self::Id;
}

#[async_trait]
pub trait CreateModelTrait<A: ActiveModelTrait> {
    async fn into_active_model(self) -> A;
}

pub trait UpdateModelTrait {}

// TODO: define and try to use instead of CreateModelTrait?
pub trait NewCreateModelTrait<R: CrudResource, M: ModelTrait> {
    fn to_model(&self, context: CrudContext<R>) -> M;
}

/// This trait must be implemented for sea-orm *ActiveModel types. It allows to update them with arbitrary types.
pub trait UpdateActiveModelTrait<UpdateModel> {
    fn update_with(&mut self, update: UpdateModel);
}

pub trait AsColType {
    fn as_col_type(&self, condition_clause_value: ConditionClauseValue) -> Result<Value, String>;
}

pub fn parse<T>(string: &str) -> Result<T, String>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    string.parse::<T>().map_err(|e| format!("{}", e))
}

/// TODO: Can we rename this or use From/Into instead? Or rename to DeriveColumnFromStringTrait?
/// This trait is used to convert from column names (for example parsed from a request) to actual entity columns.
pub trait MaybeColumnTrait {
    type Column: ColumnTrait + AsColType;
    fn get_col(name: &str) -> Option<Self::Column>;
}

pub fn to_i32(value: ConditionClauseValue) -> Result<Value, String> {
    match value {
        ConditionClauseValue::I32(num) => Ok(Value::I32(num)),
        ConditionClauseValue::I32Vec(numbers) => Ok(Value::I32Vec(numbers)),
        ConditionClauseValue::String(string) => parse::<i32>(&string).map(Value::I32),
        _ => Err(format!(
            "{value:?} can not be converted to an i32 or Vec<i32>. Expected i32 or Vec<i32> or String."
        )),
    }
}

pub fn to_i64(value: ConditionClauseValue) -> Result<Value, String> {
    match value {
        ConditionClauseValue::I64(num) => Ok(Value::I64(num)),
        ConditionClauseValue::String(string) => parse::<i64>(&string).map(Value::I64),
        _ => Err(format!(
            "{value:?} can not be converted to an i64. Expected i64 or String."
        )),
    }
}

pub fn to_u32(value: ConditionClauseValue) -> Result<Value, String> {
    match value {
        ConditionClauseValue::U32(num) => Ok(Value::U32(num)),
        ConditionClauseValue::String(string) => parse::<u32>(&string).map(Value::U32),
        _ => Err(format!(
            "{value:?} can not be converted to an u32. Expected u32 or String."
        )),
    }
}

pub fn to_f32(value: ConditionClauseValue) -> Result<Value, String> {
    match value {
        ConditionClauseValue::F32(num) => Ok(Value::F32(num)),
        ConditionClauseValue::String(string) => parse::<f32>(&string).map(Value::F32),
        _ => Err(format!(
            "{value:?} can not be converted to an f32. Expected f32 or String."
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

pub fn to_json_value(value: ConditionClauseValue) -> Result<Value, String> {
    match value {
        ConditionClauseValue::String(string) => Ok(Value::String(string)),
        _ => Err(format!(
            "{value:?} can not be converted to a String. Expected String."
        )),
    }
}

pub fn to_uuid_v4(value: ConditionClauseValue) -> Result<Value, String> {
    match value {
        ConditionClauseValue::UuidV4(uuid) => Ok(Value::UuidV4(uuid)),
        _ => Err(format!(
            "{value:?} can not be converted to a UuidV4. Expected UuidV4."
        )),
    }
}

pub fn to_uuid_v7(value: ConditionClauseValue) -> Result<Value, String> {
    match value {
        ConditionClauseValue::UuidV7(uuid) => Ok(Value::UuidV7(uuid)),
        _ => Err(format!(
            "{value:?} can not be converted to a UuidV7. Expected UuidV7."
        )),
    }
}

pub fn to_ulid(value: ConditionClauseValue) -> Result<Value, String> {
    match value {
        ConditionClauseValue::Ulid(ulid) => Ok(Value::Ulid(ulid)),
        _ => Err(format!(
            "{value:?} can not be converted to a Ulid. Expected Ulid."
        )),
    }
}

pub fn to_date_time(value: ConditionClauseValue) -> Result<Value, String> {
    match value {
        ConditionClauseValue::String(string) => {
            UtcDateTime::parse_from_rfc3339(&string).map(Value::DateTime)
        }
        _ => Err(format!(
            "{value:?} can not be converted to a DateTime. Expected String."
        )),
    }
}
