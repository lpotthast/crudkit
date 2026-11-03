//! CrudKit's REST routes as a data source.

use super::{CrudDataSource, CrudListRequest, CrudPage, CrudPaging, CrudScope, async_trait};
use crate::http::{CrudEndpoint, CrudOperation, RequestError, ReqwestExecutor};
use crate::model::{DynCreateModel, DynReadField, DynReadModel, DynUpdateModel};
use crate::model_handler::ModelHandler;
use crudkit_core::condition::{
    Condition, TryIntoAllEqualCondition, condition_matching_any_id, merge_conditions,
};
use crudkit_core::id::SerializableId;
use crudkit_core::request::{
    CreateOne, DeleteById, DeleteMany, ReadCount, ReadMany, ReadOne, UpdateOne,
};
use crudkit_core::{Deleted, DeletedMany, Saved};
use crudkit_wire_format::v1::{
    CreateOneV1, DeleteByIdV1, DeleteManyV1, DeletedManyV1, DeletedV1, ReadCountResponseV1,
    ReadCountV1, ReadManyResponseV1, ReadManyV1, ReadOneResponseV1, ReadOneV1, SavedV1,
    UpdateOneV1,
};
use std::sync::Arc;

/// The data source of a resource served by CrudKit's generated REST routes (`crudkit-rs`).
///
/// The routes are flat, so the source turns a [`CrudScope`] into a condition: every parent becomes
/// an equality clause on the field referring to it, and the scope's condition is added with AND.
/// Every operation except creation stays within that condition. A creation sends the entity as it
/// is, with the parents' fields that the instance filled in.
///
/// The source pages by offset and reads the number of all entities with every page. It can order
/// by every field. Its [`ModelHandler`] deserializes response entities into the resource's
/// concrete models.
#[derive(Debug, Clone)]
pub struct CrudRestDataSource {
    endpoint: CrudEndpoint,
    models: ModelHandler,
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
        }
    }

    async fn read_count(&self, read_count: ReadCount) -> Result<u64, RequestError> {
        let response: ReadCountResponseV1 = self
            .endpoint
            .post(CrudOperation::ReadCount, ReadCountV1::from(read_count))
            .await?;
        Ok(response.count)
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
    fn saved(
        &self,
        saved: Saved<serde_json::Value>,
    ) -> Result<Saved<DynUpdateModel>, RequestError> {
        let entity = self
            .models
            .deserialize_update_model(saved.entity)
            .map_err(|err| RequestError::Deserialize(err.to_string()))?;
        Ok(Saved {
            entity,
            violations: saved.violations,
        })
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
        let condition = scope_condition(scope)?;
        let (entities, total) = futures::future::join(
            self.read_many(ReadMany {
                limit: Some(request.limit),
                skip: Some(request.offset),
                order_by: Some(request.order_by),
                condition: condition.clone(),
            }),
            self.read_count(ReadCount { condition }),
        )
        .await;
        Ok(CrudPage {
            entities: entities?,
            paging: CrudPaging::Offset { total: total? },
        })
    }

    async fn read(
        &self,
        scope: &CrudScope,
        id: &SerializableId,
    ) -> Result<Option<DynReadModel>, RequestError> {
        let read_one = ReadOne::<DynReadField> {
            skip: None,
            order_by: None,
            condition: merge_conditions(scope_condition(scope)?, Some(id_condition(id)?)),
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
        entity: DynCreateModel,
    ) -> Result<Saved<DynUpdateModel>, RequestError> {
        let entity = entity
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
        id: &SerializableId,
        entity: DynUpdateModel,
    ) -> Result<Saved<DynUpdateModel>, RequestError> {
        let condition = merge_conditions(scope_condition(scope)?, Some(id_condition(id)?));
        let entity = entity
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
        id: &SerializableId,
    ) -> Result<Deleted, RequestError> {
        let delete_by_id = DeleteById {
            id: id.clone(),
            condition: scope_condition(scope)?,
        };
        let response: DeletedV1 = self
            .endpoint
            .post(CrudOperation::DeleteById, DeleteByIdV1::from(delete_by_id))
            .await?;
        Ok(response.into())
    }

    /// Deletes the entities with one request, whose condition matches exactly `ids` in `scope`.
    async fn delete_many(
        &self,
        scope: &CrudScope,
        ids: &[SerializableId],
    ) -> Result<DeletedMany, RequestError> {
        // Each entity's ID fields become an AND condition, and all entities are OR'd together.
        let ids_condition = condition_matching_any_id(ids.iter().cloned()).map_err(|err| {
            tracing::error!(%err, "refusing to build a mass deletion condition");
            RequestError::InvalidRequest(err.to_string())
        })?;
        let delete_many = DeleteMany {
            condition: merge_conditions(scope_condition(scope)?, Some(ids_condition)),
        };
        let response: DeletedManyV1 = self
            .endpoint
            .post(CrudOperation::DeleteMany, DeleteManyV1::from(delete_many))
            .await?;
        Ok(response.into())
    }
}

/// Returns the condition of the generated routes that keeps an operation within `scope`.
///
/// # Errors
///
/// Returns `RequestError::InvalidRequest` if the ID of a parent cannot be compared for equality,
/// because leaving the parent out would widen the operation beyond the scope.
fn scope_condition(scope: &CrudScope) -> Result<Option<Condition>, RequestError> {
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
    Ok(merge_conditions(parents, scope.condition.clone()))
}

/// Returns the condition matching the entity `id`.
fn id_condition(id: &SerializableId) -> Result<Condition, RequestError> {
    id.entries()
        .cloned()
        .try_into_all_equal_condition()
        .map_err(|e| {
            RequestError::InvalidRequest(format!("ID contains unsupported field types: {e:?}"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use crudkit_core::condition::{
        ConditionClause, ConditionClauseValue, ConditionElement, Operator,
    };
    use crudkit_core::id::IdValue;

    fn equals(column_name: &str, value: ConditionClauseValue) -> ConditionElement {
        ConditionElement::Clause(ConditionClause {
            column_name: column_name.to_owned(),
            operator: Operator::Equal,
            value,
        })
    }

    #[test]
    fn scopes_without_parents_or_condition_restrict_nothing() {
        assert_that!(scope_condition(&CrudScope::default())).is_equal_to(Ok(None));
    }

    #[test]
    fn parents_become_equality_clauses_combined_with_the_condition() {
        let base = Condition::All(vec![equals(
            "name",
            ConditionClauseValue::String("x".to_owned()),
        )]);
        let scope = CrudScope {
            condition: Some(base.clone()),
            ..CrudScope::default()
        }
        .with_parent("club_id", IdValue::I64(4));

        let mut expected = Condition::all();
        expected.push_condition(Condition::All(vec![equals(
            "club_id",
            ConditionClauseValue::I64(4),
        )]));
        expected.push_condition(base);
        assert_that!(scope_condition(&scope)).is_equal_to(Ok(Some(expected)));
    }

    #[test]
    fn parents_that_cannot_be_compared_are_refused() {
        let scope = CrudScope::default().with_parent(
            "season",
            IdValue::OffsetDateTime(time::OffsetDateTime::UNIX_EPOCH),
        );

        assert_that!(matches!(
            scope_condition(&scope),
            Err(RequestError::InvalidRequest(_))
        ))
        .is_true();
    }
}
