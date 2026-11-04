//! The default `CrudDataSource::delete_many`, which deletes one entity after the other.

use assertr::prelude::*;
use crudkit_core::id::{IdValue, SerializableId, SerializableIdEntry};
use crudkit_core::validation::PartialSerializableAggregateViolations;
use crudkit_core_macros::CkId;
use crudkit_web::data_source::{
    CrudCreateRequest, CrudDataSource, CrudDeleteManyRequest, CrudDeleteRequest, CrudDeleted,
    CrudListRequest, CrudPage, CrudReadRequest, CrudSaved, CrudScope, CrudUpdateRequest,
    async_trait,
};
use crudkit_web::http::RequestError;
use crudkit_web::model::{DynReadField, DynReadModel, DynReadOrUpdateModel};
use crudkit_web_macros::CkField;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Clone, PartialEq, Eq, Debug, CkId, CkField, Serialize, Deserialize)]
#[ck_field(model = Read)]
pub struct ReadWizard {
    pub id: i64,
    pub house: String,
}

/// Answers each deletion as the wizard's house says, and records the deletions it received.
#[derive(Debug, Default)]
struct Wizards {
    deleted: Mutex<Vec<SerializableId>>,
}

#[async_trait(?Send)]
impl CrudDataSource for Wizards {
    fn resource_name(&self) -> &'static str {
        "wizards"
    }

    fn can_order_by(&self, _field: &DynReadField) -> bool {
        false
    }

    async fn list(
        &self,
        _scope: &CrudScope,
        _request: CrudListRequest,
    ) -> Result<CrudPage, RequestError> {
        unreachable!("the tests only delete")
    }

    async fn read(
        &self,
        _scope: &CrudScope,
        _request: CrudReadRequest,
    ) -> Result<Option<DynReadModel>, RequestError> {
        unreachable!("the tests only delete")
    }

    async fn create(
        &self,
        _scope: &CrudScope,
        _request: CrudCreateRequest,
    ) -> Result<CrudSaved, RequestError> {
        unreachable!("the tests only delete")
    }

    async fn update(
        &self,
        _scope: &CrudScope,
        _request: CrudUpdateRequest,
    ) -> Result<CrudSaved, RequestError> {
        unreachable!("the tests only delete")
    }

    async fn delete(
        &self,
        _scope: &CrudScope,
        request: CrudDeleteRequest,
    ) -> Result<CrudDeleted, RequestError> {
        self.deleted.lock().expect("lock").push(request.id.clone());
        let DynReadOrUpdateModel::Read(wizard) = &request.entity else {
            unreachable!("the tests delete read models");
        };
        match wizard.downcast_ref::<ReadWizard>().house.as_str() {
            "refusing" => Err(RequestError::Forbidden("captain".to_owned())),
            "invalid" => Err(RequestError::CriticalValidationErrors {
                message: "invalid".to_owned(),
                violations: PartialSerializableAggregateViolations::default(),
            }),
            "gone" => Ok(CrudDeleted::new(0)),
            "offline" => Err(RequestError::Request("offline".to_owned())),
            "outdated" => Err(RequestError::Outdated("changed meanwhile".to_owned())),
            "slow" => Ok(CrudDeleted::accepted(wizard.clone())),
            _ => Ok(CrudDeleted::new(1)),
        }
    }
}

fn wizard(id: i64, house: &str) -> DynReadOrUpdateModel {
    DynReadOrUpdateModel::Read(DynReadModel::from(ReadWizard {
        id,
        house: house.to_owned(),
    }))
}

fn id(value: i64) -> SerializableId {
    SerializableId(vec![SerializableIdEntry {
        field_name: "id".to_owned(),
        value: IdValue::I64(value),
    }])
}

fn delete_many(wizards: &Wizards, entities: Vec<DynReadOrUpdateModel>) -> CrudDeletedOutcome {
    let outcome = futures::executor::block_on(
        wizards.delete_many(&CrudScope::default(), CrudDeleteManyRequest::new(entities)),
    )
    .expect("the default never fails as a whole");
    CrudDeletedOutcome {
        deleted: outcome.deleted,
        accepted: outcome.accepted,
        failed: outcome
            .failed
            .into_iter()
            .map(|(id, error)| (id, error.to_string()))
            .collect(),
        skipped: outcome.skipped,
    }
}

/// The outcome of a mass deletion, with the errors as text, so that it can be compared.
#[derive(Debug, PartialEq)]
struct CrudDeletedOutcome {
    deleted: Vec<SerializableId>,
    accepted: Vec<SerializableId>,
    failed: Vec<(SerializableId, String)>,
    skipped: Vec<SerializableId>,
}

#[test]
fn failures_of_one_entity_do_not_stop_the_deletion_of_the_others() {
    let wizards = Wizards::default();

    let outcome = delete_many(
        &wizards,
        vec![
            wizard(1, "gryffindor"),
            wizard(2, "refusing"),
            wizard(3, "invalid"),
            wizard(4, "gone"),
            wizard(5, "ravenclaw"),
        ],
    );

    assert_that!(outcome.deleted).is_equal_to(vec![id(1), id(5)]);
    // An entity that no deletion affected is not counted as deleted.
    assert_that!(outcome.failed).is_equal_to(vec![
        (id(2), "Permission denied: captain".to_owned()),
        (id(3), "Validation failed: invalid".to_owned()),
        (id(4), "Not found: the entity was not deleted".to_owned()),
    ]);
    assert_that!(outcome.skipped).is_empty();
}

#[test]
fn accepted_deletions_are_reported_apart_and_outdated_ones_do_not_stop_the_others() {
    let wizards = Wizards::default();

    let outcome = delete_many(
        &wizards,
        vec![
            wizard(1, "slow"),
            wizard(2, "outdated"),
            wizard(3, "hufflepuff"),
        ],
    );

    assert_that!(outcome).is_equal_to(CrudDeletedOutcome {
        deleted: vec![id(3)],
        accepted: vec![id(1)],
        failed: vec![(id(2), "Outdated: changed meanwhile".to_owned())],
        skipped: Vec::new(),
    });
}

#[test]
fn failures_beyond_one_entity_skip_the_remaining_entities() {
    let wizards = Wizards::default();

    let outcome = delete_many(
        &wizards,
        vec![
            wizard(1, "gryffindor"),
            wizard(2, "offline"),
            wizard(3, "ravenclaw"),
            wizard(4, "slytherin"),
        ],
    );

    assert_that!(outcome.deleted).is_equal_to(vec![id(1)]);
    assert_that!(outcome.failed).is_equal_to(vec![(id(2), "Network error: offline".to_owned())]);
    assert_that!(outcome.skipped).is_equal_to(vec![id(3), id(4)]);
    // The skipped entities were not tried.
    assert_that!(wizards.deleted.lock().expect("lock").clone()).is_equal_to(vec![id(1), id(2)]);
}
