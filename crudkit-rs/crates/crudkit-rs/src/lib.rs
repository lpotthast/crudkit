#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use sea_orm::{ActiveModelTrait, ColumnTrait, ModelTrait};

use crudkit_condition::ConditionClauseValue;
use crudkit_id::Id;
use crudkit_shared::Value;

use prelude::{CrudContext, CrudResource};
use validation::PersistableViolation;

pub mod axum_routes;
pub mod context;
pub mod controller;
pub mod create;
pub mod delete;
pub mod error;
pub mod lifetime;
pub mod read;
pub mod repository;
pub mod resource;
pub mod update;
pub mod validate;
pub mod validation;
pub mod websocket;

/*
* Reexport common modules.
* This allows the user to only
*
* - `use crudkit_rs::prelude::*` and
* - derive all common proc macros
*
* without the need to add more use declaration or
* to manually depend on other crud crates such as "crudkit_id",
* which are required for many derive macro implementations.
*/
pub use crudkit_condition;
pub use crudkit_id;
pub use crudkit_shared;
pub use crudkit_validation;
pub use crudkit_websocket;

pub mod prelude {
    pub use crudkit_condition;
    pub use crudkit_id;
    pub use crudkit_shared;
    pub use crudkit_validation;
    pub use crudkit_websocket;

    /* Provide convenient access to all our macros. */
    pub use derive_create_model::CkCreateModel;
    pub use derive_crud_columns::CkColumns;
    pub use derive_crud_resource_context::CkResourceContext;
    pub use derive_crudkit_id::CkId;
    pub use derive_update_model::CkUpdateModel;
    pub use derive_validation_model::CkValidationModel;

    pub use super::axum_routes::AxumCrudError;
    pub use super::error::CrudError;

    pub use super::context::CrudContext;
    pub use super::resource::CrudResource;
    pub use super::resource::CrudResourceContext;
    pub use super::websocket::CrudWebsocketService;

    pub use super::AsColType;
    pub use super::CreateModelTrait;
    pub use super::CrudColumns;
    pub use super::MaybeColumnTrait;
    pub use super::UpdateActiveModelTrait;
    pub use super::UpdateModelTrait;

    pub use super::repository::Repository;

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

pub struct RequestContext {
    pub keycloak_uuid: uuid::Uuid,
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
        now: time::OffsetDateTime,
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

pub trait CreateModelTrait<A: ActiveModelTrait> {
    fn into_active_model(self) -> impl Future<Output = A>;
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

/// TODO: Can we rename this or use From/Into instead? Or rename to DeriveColumnFromStringTrait?
/// This trait is used to convert from column names (for example parsed from a request) to actual entity columns.
pub trait MaybeColumnTrait {
    type Column: ColumnTrait + AsColType;

    fn get_col(name: &str) -> Option<Self::Column>;
}
