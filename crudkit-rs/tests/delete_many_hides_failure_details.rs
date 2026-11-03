//! A batch deletion reports each entity that it could not delete with a generic reason, because
//! the details of a failure can contain the values of the entity.

mod support;

use std::sync::Arc;

use assertr::prelude::*;
use axum::http::StatusCode;
use crudkit_core::condition::ConditionClauseValue;
use crudkit_core::request::DeleteMany;
use crudkit_rs::crudkit_wire_format::v1::DeleteManyV1;
use crudkit_rs::prelude::*;
use futures::executor::block_on;
use support::{Clients, MemoryRepository, SECRET, account, context, equals, post};

#[test]
fn failed_deletions_name_no_values_of_the_entity() {
    // Arrange
    let repository = MemoryRepository::with((1..=3).map(account).collect());
    repository.reference(2);
    let context = context::<RestrictedAuthPolicy>(repository, Arc::new(Clients::default()));
    let body = DeleteManyV1::from(DeleteMany {
        condition: Some(equals(
            "secret",
            ConditionClauseValue::String(SECRET.to_owned()),
        )),
    });

    // Act
    let (status, body) = block_on(post(&context, "delete-many", &body));

    // Assert
    assert_that!(status).is_equal_to(StatusCode::OK);
    assert_that!(body["deleted_count"].clone()).is_equal_to(serde_json::json!(2));
    assert_that!(body["errors"][0]["reason"].clone())
        .is_equal_to(serde_json::json!("Repository error."));
    assert_that!(body.to_string()).does_not_contain(SECRET);
    assert_that!(context.repository.accounts()).is_equal_to(vec![account(2)]);
}
