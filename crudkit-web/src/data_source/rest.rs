//! CrudKit's REST routes as a data source.

use super::{
    CrudCountRequest, CrudCreateRequest, CrudDataSource, CrudDeleteManyRequest, CrudDeleteRequest,
    CrudDeleted, CrudDeletedMany, CrudListRequest, CrudPage, CrudPaging, CrudReadRequest,
    CrudSaved, CrudScope, CrudUpdateRequest, async_trait,
};
use crate::http::{CrudEndpoint, CrudOperation, RequestError, ReqwestExecutor};
use crate::model::{
    DynReadField, DynReadModel, DynReadOrUpdateModel, ErasedCreateField, ErasedCreateModel,
    ErasedReadField, ErasedReadModel, ErasedUpdateField, ErasedUpdateModel, Model,
};
use crate::model_handler::ModelHandler;
use crate::resource::Resource;
use crudkit_core::condition::{
    Condition, TryIntoAllEqualCondition, condition_matching_any_id, merge_conditions,
};
use crudkit_core::id::SerializableId;
use crudkit_core::request::{
    CreateOne, DeleteById, DeleteMany, ReadCount, ReadMany, ReadOne, UpdateOne,
};
use crudkit_core::validation::PartialSerializableAggregateViolations;
use crudkit_core::{DeletedMany, Saved};
use crudkit_wire_format::v1::{
    CreateOneV1, DeleteByIdV1, DeleteManyV1, DeletedManyV1, DeletedV1, ReadCountResponseV1,
    ReadCountV1, ReadManyResponseV1, ReadManyV1, ReadOneResponseV1, ReadOneV1, SavedV1,
    UpdateOneV1,
};
use std::sync::Arc;

/// The data source of a resource served by CrudKit's generated REST routes (`crudkit-rs`).
///
/// The routes are flat, so the source turns a [`CrudScope`] into a condition: every parent becomes
/// an equality clause on the field referring to it. Its base condition, if any, is added with AND.
/// Every operation except creation stays within that condition. A creation sends the entity as it
/// is, with the parents' fields that the instance filled in.
///
/// The source pages by offset, counts with a request of its own, and can order by every field.
/// Its [`ModelHandler`] deserializes response entities into the resource's concrete models.
#[derive(Debug, Clone)]
pub struct CrudRestDataSource {
    endpoint: CrudEndpoint,
    models: ModelHandler,
    base_condition: Option<Condition>,
}

impl CrudRestDataSource {
    /// Creates the data source of the resource `resource_name`, whose routes are served below
    /// `api_base_url` (`{api_base_url}/{resource_name}/crud/v1/...`) and requested through
    /// `executor`.
    pub fn new(
        api_base_url: String,
        executor: Arc<dyn ReqwestExecutor>,
        resource_name: String,
        models: ModelHandler,
    ) -> Self {
        Self {
            endpoint: CrudEndpoint::new(api_base_url, executor, resource_name),
            models,
            base_condition: None,
        }
    }

    /// Creates the data source of resource `R`, whose routes are served below `api_base_url` and
    /// requested through `executor`.
    pub fn for_resource<R>(api_base_url: String, executor: Arc<dyn ReqwestExecutor>) -> Self
    where
        R: Resource,
        R::CreateModel: ErasedCreateModel,
        R::ReadModel: ErasedReadModel,
        R::UpdateModel: ErasedUpdateModel + From<R::ReadModel>,
        <R::ReadModel as Model>::Field: ErasedReadField,
        <R::CreateModel as Model>::Field: ErasedCreateField,
        <R::UpdateModel as Model>::Field: ErasedUpdateField,
    {
        Self::new(
            api_base_url,
            executor,
            R::resource_name().to_owned(),
            ModelHandler::new::<R::CreateModel, R::ReadModel, R::UpdateModel>(),
        )
    }

    /// Returns this data source restricted to `condition`: every operation except creation stays
    /// within it, in addition to the scope.
    #[must_use]
    pub fn with_base_condition(mut self, condition: Condition) -> Self {
        self.base_condition = Some(condition);
        self
    }

    async fn read_many(
        &self,
        read_many: ReadMany<DynReadField>,
    ) -> Result<Vec<DynReadModel>, RequestError> {
        let response: ReadManyResponseV1<serde_json::Value> = self
            .endpoint
            .post(CrudOperation::ReadMany, ReadManyV1::from(read_many))
            .await?;
        response
            .entities
            .into_iter()
            .map(|json| self.models.deserialize_read_model(json))
            .collect::<Result<_, _>>()
            .map_err(|err| RequestError::Deserialize(err.to_string()))
    }

    /// Deserializes the entity of a create or update response into an update model.
    fn saved(&self, saved: Saved<serde_json::Value>) -> Result<CrudSaved, RequestError> {
        let entity = self
            .models
            .deserialize_update_model(saved.entity)
            .map_err(|err| RequestError::Deserialize(err.to_string()))?;
        Ok(CrudSaved::new(entity).with_violations(saved.violations))
    }

    /// Returns the condition of the generated routes that keeps an operation within `scope` and
    /// the base condition.
    fn scope_condition(&self, scope: &CrudScope) -> Result<Option<Condition>, RequestError> {
        scope_condition(scope, self.base_condition.as_ref())
    }
}

#[async_trait(?Send)]
impl CrudDataSource for CrudRestDataSource {
    fn resource_name(&self) -> &str {
        self.endpoint.resource_name()
    }

    fn can_order_by(&self, _field: &DynReadField) -> bool {
        true
    }

    async fn list(
        &self,
        scope: &CrudScope,
        request: CrudListRequest,
    ) -> Result<CrudPage, RequestError> {
        let entities = self
            .read_many(ReadMany {
                limit: Some(request.limit),
                skip: Some(request.offset),
                order_by: Some(request.order_by),
                condition: self.scope_condition(scope)?,
            })
            .await?;
        Ok(CrudPage::new(entities, CrudPaging::Offset))
    }

    async fn count(
        &self,
        scope: &CrudScope,
        _request: CrudCountRequest,
    ) -> Result<Option<u64>, RequestError> {
        let read_count = ReadCount {
            condition: self.scope_condition(scope)?,
        };
        let response: ReadCountResponseV1 = self
            .endpoint
            .post(CrudOperation::ReadCount, ReadCountV1::from(read_count))
            .await?;
        Ok(Some(response.count))
    }

    async fn read(
        &self,
        scope: &CrudScope,
        request: CrudReadRequest,
    ) -> Result<Option<DynReadModel>, RequestError> {
        let read_one = ReadOne::<DynReadField> {
            skip: None,
            order_by: None,
            condition: merge_conditions(
                self.scope_condition(scope)?,
                Some(id_condition(&request.id)?),
            ),
        };
        let response: ReadOneResponseV1<serde_json::Value> = self
            .endpoint
            .post(CrudOperation::ReadOne, ReadOneV1::from(read_one))
            .await?;
        response
            .entity
            .map(|json| self.models.deserialize_read_model(json))
            .transpose()
            .map_err(|err| RequestError::Deserialize(err.to_string()))
    }

    async fn create(
        &self,
        _scope: &CrudScope,
        request: CrudCreateRequest,
    ) -> Result<CrudSaved, RequestError> {
        let entity = request
            .entity
            .to_untagged_json()
            .map_err(|e| RequestError::InvalidRequest(e.to_string()))?;
        let response: SavedV1<serde_json::Value> = self
            .endpoint
            .post(
                CrudOperation::CreateOne,
                CreateOneV1::from(CreateOne { entity }),
            )
            .await?;
        self.saved(response.into())
    }

    async fn update(
        &self,
        scope: &CrudScope,
        request: CrudUpdateRequest,
    ) -> Result<CrudSaved, RequestError> {
        let condition = merge_conditions(
            self.scope_condition(scope)?,
            Some(id_condition(&request.id)?),
        );
        let entity = request
            .entity
            .to_untagged_json()
            .map_err(|e| RequestError::InvalidRequest(e.to_string()))?;
        let response: SavedV1<serde_json::Value> = self
            .endpoint
            .post(
                CrudOperation::UpdateOne,
                UpdateOneV1::from(UpdateOne { entity, condition }),
            )
            .await?;
        self.saved(response.into())
    }

    async fn delete(
        &self,
        scope: &CrudScope,
        request: CrudDeleteRequest,
    ) -> Result<CrudDeleted, RequestError> {
        let delete_by_id = DeleteById {
            id: request.id,
            condition: self.scope_condition(scope)?,
        };
        let response: DeletedV1 = self
            .endpoint
            .post(CrudOperation::DeleteById, DeleteByIdV1::from(delete_by_id))
            .await?;
        Ok(CrudDeleted::new(response.entities_affected))
    }

    /// Deletes the entities with one request, whose condition matches exactly their IDs in
    /// `scope`.
    async fn delete_many(
        &self,
        scope: &CrudScope,
        request: CrudDeleteManyRequest,
    ) -> Result<CrudDeletedMany, RequestError> {
        let ids = request.entities.iter().map(DynReadOrUpdateModel::id);
        // Each entity's ID fields become an AND condition, and all entities are OR'd together.
        let ids_condition = condition_matching_any_id(ids).map_err(|err| {
            tracing::error!(%err, "refusing to build a mass deletion condition");
            RequestError::InvalidRequest(err.to_string())
        })?;
        let delete_many = DeleteMany {
            condition: merge_conditions(self.scope_condition(scope)?, Some(ids_condition)),
        };
        let response: DeletedManyV1 = self
            .endpoint
            .post(CrudOperation::DeleteMany, DeleteManyV1::from(delete_many))
            .await?;
        Ok(deleted_many(response.into()))
    }
}

/// Returns the condition of the generated routes that keeps an operation within `scope` and
/// `base_condition`: an equality clause per parent, and the base condition, combined with AND.
///
/// # Errors
///
/// Returns `RequestError::InvalidRequest` if the ID of a parent cannot be compared for equality,
/// because leaving the parent out would widen the operation beyond the scope.
fn scope_condition(
    scope: &CrudScope,
    base_condition: Option<&Condition>,
) -> Result<Option<Condition>, RequestError> {
    let parents = if scope.parents.is_empty() {
        None
    } else {
        let condition = scope
            .parents
            .iter()
            .cloned()
            .try_into_all_equal_condition()
            .map_err(|_| {
                RequestError::InvalidRequest(format!(
                    "the parents of the scope cannot be compared for equality: {:?}",
                    scope.parents
                ))
            })?;
        Some(condition)
    };
    Ok(merge_conditions(parents, base_condition.cloned()))
}

/// Returns the condition matching the entity `id`.
fn id_condition(id: &SerializableId) -> Result<Condition, RequestError> {
    id.entries()
        .cloned()
        .try_into_all_equal_condition()
        .map_err(|e| RequestError::InvalidRequest(format!("Invalid entity ID: {e}")))
}

/// Converts the outcome of the `delete-many` route.
///
/// The route reports the deletions that a lifecycle hook aborted with their reason only, without
/// telling a refusal (403) from a rejection (422), so they become rejections. Its other failures
/// carry a message only, so they become server errors.
fn deleted_many(deleted: DeletedMany) -> CrudDeletedMany {
    let aborted = deleted
        .aborted
        .into_iter()
        .map(|(id, reason)| (id, RequestError::UnprocessableEntity(reason)));
    let invalid = deleted.validation_failed.into_iter().map(|id| {
        let error = RequestError::CriticalValidationErrors {
            message: "Critical validation errors prevent the deletion.".to_owned(),
            violations: PartialSerializableAggregateViolations::default(),
        };
        (id, error)
    });
    let failed = deleted
        .errors
        .into_iter()
        .map(|(id, message)| (id, RequestError::InternalServerError(message)));

    let mut outcome = CrudDeletedMany::new();
    outcome.deleted = deleted.deleted_ids;
    outcome.failed = aborted.chain(invalid).chain(failed).collect();
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use crudkit_core::condition::{
        ConditionClause, ConditionClauseValue, ConditionElement, Operator,
    };
    use crudkit_core::id::{IdValue, SerializableIdEntry};

    fn equals(column_name: &str, value: ConditionClauseValue) -> ConditionElement {
        ConditionElement::Clause(ConditionClause {
            column_name: column_name.to_owned(),
            operator: Operator::Equal,
            value,
        })
    }

    fn id(value: i64) -> SerializableId {
        SerializableId(vec![SerializableIdEntry {
            field_name: "id".to_owned(),
            value: IdValue::I64(value),
        }])
    }

    #[test]
    fn scopes_without_parents_or_base_condition_restrict_nothing() {
        assert_that!(scope_condition(&CrudScope::default(), None)).is_equal_to(Ok(None));
    }

    #[test]
    fn parents_become_equality_clauses_combined_with_the_base_condition() {
        let base = Condition::All(vec![equals(
            "name",
            ConditionClauseValue::String("x".to_owned()),
        )]);
        let scope = CrudScope::default().with_parent("club_id", IdValue::I64(4));

        let mut expected = Condition::all();
        expected.push_condition(Condition::All(vec![equals(
            "club_id",
            ConditionClauseValue::I64(4),
        )]));
        expected.push_condition(base.clone());
        assert_that!(scope_condition(&scope, Some(&base))).is_equal_to(Ok(Some(expected)));
    }

    #[test]
    fn parents_that_cannot_be_compared_are_refused() {
        let scope = CrudScope::default().with_parent(
            "season",
            IdValue::OffsetDateTime(time::OffsetDateTime::UNIX_EPOCH),
        );

        assert_that!(matches!(
            scope_condition(&scope, None),
            Err(RequestError::InvalidRequest(_))
        ))
        .is_true();
    }

    #[test]
    fn mass_deletion_outcomes_keep_the_kind_of_each_failure() {
        let outcome = deleted_many(DeletedMany {
            deleted_count: 1,
            deleted_ids: vec![id(1)],
            aborted: vec![(id(2), "captain".to_owned())],
            validation_failed: vec![id(3)],
            errors: vec![(id(4), "database".to_owned())],
        });

        assert_that!(outcome.deleted).is_equal_to(vec![id(1)]);
        assert_that!(outcome.skipped).is_empty();
        let failed = outcome
            .failed
            .iter()
            .map(|(id, error)| (id.clone(), std::mem::discriminant(error)))
            .collect::<Vec<_>>();
        let kind = std::mem::discriminant::<RequestError>;
        assert_that!(failed).is_equal_to(vec![
            (
                id(2),
                kind(&RequestError::UnprocessableEntity(String::new())),
            ),
            (
                id(3),
                kind(&RequestError::CriticalValidationErrors {
                    message: String::new(),
                    violations: PartialSerializableAggregateViolations::default(),
                }),
            ),
            (
                id(4),
                kind(&RequestError::InternalServerError(String::new())),
            ),
        ]);
    }
}
