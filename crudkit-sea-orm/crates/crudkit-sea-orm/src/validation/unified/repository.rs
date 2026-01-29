//! Unified validation result repository implementation.

use std::sync::Arc;

use async_trait::async_trait;
use error_stack::{Report, ResultExt};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, Order, QueryFilter, QueryOrder,
    Set, TransactionTrait,
};
use thiserror::Error;
use tracing::info;

use super::model::{ActiveModel, Column, Entity, Model};
use crate::validation::PersistedViolationSeverity;
use crudkit_core::id::{Id, SerializableId};
use crudkit_core::resource::ResourceName;
use crudkit_core::validation::validator::ValidatorInfo;
use crudkit_core::validation::violation::{Violation, Violations};
use crudkit_core::validation::{
    ResourceViolations, ViolationsByEntity, ViolationsByResource, ViolationsByValidator,
};
use crudkit_rs::repository::{RepositoryError, ValidationResultRepository};

/// Errors that can occur when working with the validation repository.
#[derive(Debug, Error)]
pub enum UnifiedValidationRepositoryError {
    #[error("Database error")]
    Db,

    #[error("Serialization error")]
    Serialization,

    #[error("Deserialization error")]
    Deserialization,

    #[error("Transaction error")]
    Transaction,
}

impl RepositoryError for UnifiedValidationRepositoryError {}

type Result<T> = core::result::Result<T, Report<UnifiedValidationRepositoryError>>;

/// Repository for persisting validation results.
///
/// Supports two storage modes:
/// - Unified: All resources in one table, distinguished by `resource_type` column.
/// - Per-resource: Each resource has its own table.
pub struct UnifiedValidationRepository {
    pub db: Arc<DatabaseConnection>,
}

impl UnifiedValidationRepository {
    /// Serialize an entity ID to JSON for storage.
    fn serialize_id<I: Id>(id: &I) -> Result<serde_json::Value> {
        // TODO: Can we do this serialization without requiring an intermittent allocation?
        let serializable = id.to_serializable_id();
        serde_json::to_value(&serializable)
            .change_context(UnifiedValidationRepositoryError::Serialization)
    }

    fn deserialize_id_untyped(value: &serde_json::Value) -> Result<SerializableId> {
        serde_json::from_value(value.clone())
            .change_context(UnifiedValidationRepositoryError::Deserialization)
    }

    /// Deserialize an entity ID from JSON.
    fn deserialize_id<I: Id>(value: &serde_json::Value) -> Result<I> {
        let serializable: SerializableId = Self::deserialize_id_untyped(value)?;
        I::from_serializable_id(&serializable)
            .ok_or_else(|| Report::new(UnifiedValidationRepositoryError::Deserialization))
    }

    /// Build a filter condition for all entities of a specific resource type.
    fn resource_filter(&self, resource_name: &str) -> sea_orm::Condition {
        sea_orm::Condition::all().add(Column::ResourceName.eq(resource_name))
    }

    /// Build a filter condition for a specific entity of a specific resource type.
    fn entity_filter<I: Id>(
        &self,
        resource_name: &str,
        entity_id: &I,
    ) -> Result<sea_orm::Condition> {
        let json_id = Self::serialize_id(entity_id)?;
        let cond = sea_orm::Condition::all()
            .add(Column::ResourceName.eq(resource_name))
            .add(Column::EntityId.eq(json_id));
        Ok(cond)
    }

    /// List all validation results, ordered for grouping.
    async fn list_all_ordered(&self) -> Result<Vec<Model>> {
        let query = Entity::find()
            //.order_by(Column::EntityId, Order::Asc) // JSON column not orderable!
            .order_by(Column::ValidatorName, Order::Asc)
            .order_by(Column::ValidatorVersion, Order::Desc)
            .order_by(Column::ViolationSeverity, Order::Asc);

        query
            .all(self.db.as_ref())
            .await
            .change_context(UnifiedValidationRepositoryError::Db)
    }

    /// List all validation results, ordered for grouping.
    async fn list_all_of_resource_ordered(&self, resource_name: &str) -> Result<Vec<Model>> {
        let mut query = Entity::find()
            //.order_by(Column::EntityId, Order::Asc) // JSON column not orderable!
            .order_by(Column::ValidatorName, Order::Asc)
            .order_by(Column::ValidatorVersion, Order::Desc)
            .order_by(Column::ViolationSeverity, Order::Asc);

        query = query.filter(self.resource_filter(resource_name));

        query
            .all(self.db.as_ref())
            .await
            .change_context(UnifiedValidationRepositoryError::Db)
    }

    /// Group validation results by entity and validator.
    async fn grouped<I: Id>(&self, resource_name: &str) -> Result<ViolationsByEntity<I>> {
        let mut violations_by_entity = ViolationsByEntity::new();

        for entry in self.list_all_of_resource_ordered(resource_name).await? {
            let entity_id = Self::deserialize_id(&entry.entity_id)?;

            let validator_info =
                ValidatorInfo::new_owned(entry.validator_name, entry.validator_version as u32);

            let violation = match entry.violation_severity {
                PersistedViolationSeverity::Major => Violation::major(entry.violation_message),
                PersistedViolationSeverity::Critical => {
                    Violation::critical(entry.violation_message)
                }
            };

            violations_by_entity.push(entity_id, validator_info, violation);
        }

        Ok(violations_by_entity)
    }

    /// Group all validation results by: resource type -> entity -> validator -> violations.
    async fn all(&self) -> Result<ViolationsByResource> {
        let mut all = ViolationsByResource::new();

        for entry in self.list_all_ordered().await? {
            let aggregate_name = ResourceName::new(entry.resource_name);

            let entity_id = Self::deserialize_id_untyped(&entry.entity_id)?;

            let validator_info =
                ValidatorInfo::new_owned(entry.validator_name, entry.validator_version as u32);

            let violation = match entry.violation_severity {
                PersistedViolationSeverity::Major => Violation::major(entry.violation_message),
                PersistedViolationSeverity::Critical => {
                    Violation::critical(entry.violation_message)
                }
            };

            all.map
                .entry(aggregate_name)
                .or_insert_with(ResourceViolations::new)
                .by_entity
                .map
                .entry(entity_id)
                .or_insert_with(ViolationsByValidator::new)
                .push(validator_info, violation);
        }

        Ok(all)
    }

    /// Replace all validations for a specific entity and validator.
    /// Uses a transaction to ensure atomicity.
    async fn replace_violations_for<I: Id>(
        &self,
        resource_name: &str,
        entity_id: &I,
        validator_info: ValidatorInfo<'static>,
        violations: Violations,
        now: &time::OffsetDateTime,
    ) -> Result<()> {
        let validator_name = validator_info.validator_name.as_ref();
        let validator_version = validator_info.validator_version;

        let json_id = Self::serialize_id(entity_id)?;

        // Use a transaction to ensure atomicity of delete + insert.
        let txn = self
            .db
            .begin()
            .await
            .change_context(UnifiedValidationRepositoryError::Transaction)?;

        // Build filter for deletion.
        let delete_cond = sea_orm::Condition::all()
            .add(Column::ResourceName.eq(resource_name))
            .add(Column::EntityId.eq(json_id.clone()))
            .add(Column::ValidatorName.eq(validator_name))
            .add(Column::ValidatorVersion.lte(validator_version));

        // Delete old validation results.
        let delete_result = Entity::delete_many()
            .filter(delete_cond)
            .exec(&txn)
            .await
            .change_context(UnifiedValidationRepositoryError::Db)?;

        if delete_result.rows_affected > 0 {
            info!(
                "Deleted {} old violations for entity {entity_id} from validator '{validator_name}' \
                 of versions <= {validator_version}.",
                delete_result.rows_affected
            );
        }

        // Insert new validation results.
        let mut num_saved = 0;
        for violation in violations {
            let active_model = ActiveModel {
                id: sea_orm::ActiveValue::NotSet,
                resource_name: Set(resource_name.to_owned()),
                entity_id: Set(json_id.clone()),
                validator_name: Set(validator_name.to_owned()),
                validator_version: Set(validator_version as i64),
                violation_severity: Set(violation.severity().into()),
                violation_message: Set(violation.into_message()),
                created_at: Set(*now),
            };

            active_model
                .insert(&txn)
                .await
                .change_context(UnifiedValidationRepositoryError::Db)?;

            num_saved += 1;
        }

        // Commit the transaction.
        txn.commit()
            .await
            .change_context(UnifiedValidationRepositoryError::Transaction)?;

        info!(
            "Saved {num_saved} new violations for entity {entity_id} \
             for validator '{validator_name}' v{validator_version}."
        );

        Ok(())
    }

    /// Delete all validations of a specific entity.
    async fn delete_violations_of_entity<I: Id>(
        &self,
        resource_name: &str,
        entity_id: &I,
    ) -> Result<()> {
        let cond = self.entity_filter(resource_name, entity_id)?;

        let delete_result = Entity::delete_many()
            .filter(cond)
            .exec(self.db.as_ref())
            .await
            .change_context(UnifiedValidationRepositoryError::Db)?;

        info!(
            "Deleted {} violations for entity {entity_id:?} of resource type {resource_name}.",
            delete_result.rows_affected,
        );

        Ok(())
    }

    /// Delete all violations of a specific resource.
    async fn delete_violations_of_resource(&self, resource_name: &str) -> Result<()> {
        let delete_result = Entity::delete_many()
            .filter(self.resource_filter(resource_name))
            .exec(self.db.as_ref())
            .await
            .change_context(UnifiedValidationRepositoryError::Db)?;

        info!(
            "Deleted all {} violations for resource type '{resource_name}'.",
            delete_result.rows_affected,
        );

        Ok(())
    }
}

#[async_trait]
impl ValidationResultRepository for UnifiedValidationRepository {
    type Error = Report<UnifiedValidationRepositoryError>;

    async fn delete_all_of_entity<I: Id>(
        &self,
        resource_name: &str,
        entity_id: &I,
    ) -> core::result::Result<(), Self::Error> {
        self.delete_violations_of_entity(resource_name, entity_id)
            .await
    }

    async fn delete_all_of_resource(
        &self,
        resource_name: &str,
    ) -> std::result::Result<(), Self::Error> {
        self.delete_violations_of_resource(resource_name).await
    }

    async fn save_all<I: Id>(
        &self,
        resource_name: &str,
        validation_results: ViolationsByEntity<I>,
    ) -> core::result::Result<(), Self::Error> {
        let now = time::OffsetDateTime::now_utc();

        for (entity_id, violations_by_validator) in validation_results.map {
            for (validator_info, violations) in violations_by_validator.violations_by_validator {
                // Log errors but continue processing remaining validators.
                if let Err(e) = self
                    .replace_violations_for(
                        resource_name,
                        &entity_id,
                        validator_info.clone(),
                        violations,
                        &now,
                    )
                    .await
                {
                    tracing::error!(
                        "Failed to save violations for entity {entity_id:?} of resource type '{resource_name}': {e:?}"
                    );
                }
            }
        }

        Ok(())
    }

    async fn list_all_of_resource<I: Id>(
        &self,
        resource_name: &str,
    ) -> std::result::Result<ViolationsByEntity<I>, Self::Error> {
        self.grouped(resource_name).await
    }

    async fn list_all(&self) -> std::result::Result<ViolationsByResource, Self::Error> {
        self.all().await
    }
}
