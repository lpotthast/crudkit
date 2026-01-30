//! Adapter traits for SeaORM integration.
//!
//! These traits define the mapping between crudkit-rs's storage-agnostic types
//! and SeaORM-specific types (Entity, ActiveModel, Column).
//!
//! To use `SeaOrmRepo`, your resource must implement `SeaOrmResource`.

use std::future::Future;

use crudkit_rs::prelude::CrudResource;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, EntityTrait, FromQueryResult,
    IntoActiveModel, ModelTrait, PrimaryKeyTrait,
};

// Re-export dependencies used in generated code.
pub use crudkit_core;
pub use crudkit_core::condition as crudkit_condition;
pub use crudkit_core::id as crudkit_id;

/// Trait that maps crudkit-rs abstract types to SeaORM types.
///
/// Implement this trait for your resource to use `SeaOrmRepo`.
pub trait SeaOrmResource: CrudResource {
    // =========================================================================
    // Main Entity (for fetch/update/delete operations)
    // =========================================================================

    /// The SeaORM entity for the main table.
    type Entity: EntityTrait<Model = Self::SeaOrmModel, Column = Self::Column, PrimaryKey = Self::PrimaryKey>;

    /// The SeaORM model for the main table.
    type SeaOrmModel: ModelTrait<Entity = Self::Entity>
        + IntoActiveModel<Self::ActiveModel>
        + FromQueryResult
        + Into<Self::Model>
        + Clone
        + Send
        + Sync
        + 'static;

    /// The SeaORM active model for the main table.
    type ActiveModel: ActiveModelTrait<Entity = Self::Entity>
        + ActiveModelBehavior
        + Send
        + Sync
        + 'static;

    /// The SeaORM column enum for the main table.
    type Column: ColumnTrait + Send + Sync + 'static;

    /// The SeaORM primary key for the main table.
    type PrimaryKey: PrimaryKeyTrait + Send + Sync + 'static;

    // =========================================================================
    // Read View (for read operations, may be a SQL view)
    // =========================================================================

    /// The SeaORM entity for the read view.
    type ReadViewEntity: EntityTrait<
        Model = Self::ReadViewSeaOrmModel,
        Column = Self::ReadViewColumn,
        PrimaryKey = Self::ReadViewPrimaryKey,
    >;

    /// The SeaORM model for the read view.
    type ReadViewSeaOrmModel: ModelTrait<Entity = Self::ReadViewEntity>
        + FromQueryResult
        + Into<Self::ReadModel>
        + Clone
        + Send
        + Sync
        + 'static;

    /// The SeaORM active model for the read view.
    type ReadViewActiveModel: ActiveModelTrait<Entity = Self::ReadViewEntity> + Send + Sync + 'static;

    /// The SeaORM column enum for the read view.
    type ReadViewColumn: ColumnTrait + Send + Sync + 'static;

    /// The SeaORM primary key for the read view.
    type ReadViewPrimaryKey: PrimaryKeyTrait + Send + Sync + 'static;

    // =========================================================================
    // Column Field Mapping
    // =========================================================================

    /// Mapping from ModelField to SeaORM Column.
    fn model_field_to_column(field: &Self::ModelField) -> Self::Column;

    /// Mapping from ReadModelField to SeaORM ReadViewColumn.
    fn read_model_field_to_column(field: &Self::ReadModelField) -> Self::ReadViewColumn;
}

/// Trait for converting a CreateModel into a SeaORM ActiveModel.
pub trait IntoSeaOrmActiveModel<A: ActiveModelTrait> {
    /// Convert this create model into a SeaORM active model for insertion.
    fn into_active_model(self) -> impl Future<Output = A> + Send;
}

/// Blanket implementation: any type implementing SeaOrmCreateModel also implements IntoSeaOrmActiveModel.
impl<T, A> IntoSeaOrmActiveModel<A> for T
where
    T: SeaOrmCreateModel<A>,
    A: ActiveModelTrait,
{
    fn into_active_model(self) -> impl Future<Output = A> + Send {
        SeaOrmCreateModel::into_active_model(self)
    }
}

/// Trait for applying an UpdateModel to a SeaORM ActiveModel.
pub trait ApplyToActiveModel<A: ActiveModelTrait> {
    /// Apply this update model's changes to the given active model.
    fn apply_to(self, active_model: &mut A);
}

/// Blanket implementation: UpdateModel can apply to ActiveModel if ActiveModel implements SeaOrmUpdateModel.
impl<U, A> ApplyToActiveModel<A> for U
where
    A: ActiveModelTrait + SeaOrmUpdateModel<U>,
{
    fn apply_to(self, active_model: &mut A) {
        active_model.update_with(self);
    }
}

/// Trait for converting a SeaORM Model to an ActiveModel for updates.
pub trait IntoActiveModelForUpdate<A: ActiveModelTrait> {
    /// Convert this model to an active model for updating.
    fn into_active_model_for_update(self) -> A;
}

/// Blanket implementation: any type implementing SeaORM's IntoActiveModel also implements IntoActiveModelForUpdate.
impl<T, A> IntoActiveModelForUpdate<A> for T
where
    T: IntoActiveModel<A>,
    A: ActiveModelTrait,
{
    fn into_active_model_for_update(self) -> A {
        IntoActiveModel::into_active_model(self)
    }
}

// ===========================================================================
// Traits generated by derive macros
// ===========================================================================

/// Trait for mapping field enums to SeaORM columns.
/// Generated by the `CkSeaOrmBridge` derive macro.
pub trait CrudColumns<C: ColumnTrait> {
    fn to_sea_orm_column(&self) -> C;
}

/// Trait for CreateModel -> ActiveModel conversion.
///
/// This is a SeaORM-specific trait that defines how to convert a create model DTO
/// into a SeaORM `ActiveModel` for insertion.
///
/// Generated by the `CkSeaOrmCreateModel` derive macro.
pub trait SeaOrmCreateModel<A: ActiveModelTrait>: Clone + Send + Sync {
    fn into_active_model(self) -> impl Future<Output = A> + Send;
}

/// Trait for applying UpdateModel fields to a SeaORM ActiveModel.
///
/// This is a SeaORM-specific trait that defines how to apply update model fields
/// to an existing `ActiveModel`.
///
/// Generated by the `CkSeaOrmUpdateModel` derive macro.
pub trait SeaOrmUpdateModel<U> {
    fn update_with(&mut self, update: U);
}

/// Trait for creating validation ActiveModel.
/// Generated by the `CkValidationModel` derive macro.
pub trait NewActiveValidationModel<ParentId> {
    fn new(
        entity_id: ParentId,
        validator_name: String,
        validator_version: i32,
        violation: PersistableViolation,
        now: time::OffsetDateTime,
    ) -> Self;
}

/// Trait for validation Model.
/// Generated by the `CkValidationModel` derive macro.
pub trait ValidatorModel<ParentId> {
    fn get_id(&self) -> ParentId;
    fn get_validator_name(&self) -> String;
    fn get_validator_version(&self) -> i32;
}

/// Trait for validation columns.
/// Generated by the `CkValidationModel` derive macro.
pub trait ValidationColumns {
    fn get_validator_name_column() -> Self;
    fn get_validator_version_column() -> Self;
    fn get_violation_severity_column() -> Self;
}

/// Trait for ID columns.
/// Generated by the `CkValidationModel` derive macro.
pub trait IdColumns {
    fn get_id_columns() -> Vec<Self>
    where
        Self: Sized;
}

/// A violation that can be persisted to the database.
/// Wraps a validation violation with its severity and message.
#[derive(Debug, Clone)]
pub struct PersistableViolation {
    severity: crate::validation::PersistedViolationSeverity,
    message: String,
}

impl PersistableViolation {
    pub fn new(severity: crate::validation::PersistedViolationSeverity, message: String) -> Self {
        Self { severity, message }
    }

    pub fn severity(&self) -> crate::validation::PersistedViolationSeverity {
        self.severity
    }

    pub fn into_message(self) -> String {
        self.message
    }
}
