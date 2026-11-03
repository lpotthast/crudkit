//! A change of an entity reaches only the clients that may read the entity's resource.

mod support;

use std::sync::Arc;

use assertr::prelude::*;
use crudkit_core::id::{IdValue, SerializableId, SerializableIdEntry};
use crudkit_rs::crudkit_wire_format::v1::{CollabEventV1, CollabMessageV1};
use crudkit_rs::prelude::*;
use futures::executor::block_on;
use support::{
    AccountUpdate, Accounts, Clients, MemoryRepository, NewAccount, Token, account, context,
};

#[test]
fn changes_of_a_resource_that_requires_authentication_reach_only_authenticated_clients() {
    // Arrange
    let clients = Arc::new(Clients::default());
    let reader = clients.connect(true);
    let anonymous = clients.connect(false);
    let context =
        context::<RestrictedAuthPolicy>(MemoryRepository::with(vec![account(1)]), clients.clone());

    // Act
    block_on(create_update_and_delete(&context));

    // Assert
    assert_that!(kinds(&clients.received(reader))).is_equal_to(vec![
        "entity_created",
        "partial_validation_result",
        "entity_updated",
        "entity_deleted",
    ]);
    assert_that!(clients.received(anonymous)).is_empty();
}

#[test]
fn changes_of_a_public_resource_reach_every_client() {
    // Arrange
    let clients = Arc::new(Clients::default());
    let reader = clients.connect(true);
    let anonymous = clients.connect(false);
    let context =
        context::<DefaultAuthPolicy>(MemoryRepository::with(vec![account(1)]), clients.clone());

    // Act
    block_on(create_update_and_delete(&context));

    // Assert
    assert_that!(kinds(&clients.received(anonymous))).is_equal_to(vec![
        "entity_created",
        "partial_validation_result",
        "entity_updated",
        "entity_deleted",
    ]);
    assert_that!(clients.received(anonymous)).is_equal_to(clients.received(reader));
}

/// Creates an account, then updates and deletes the existing account 1.
async fn create_update_and_delete<P: CrudAuthPolicy>(context: &Arc<CrudContext<Accounts<P>>>) {
    let request = || RequestContext::authenticated(Token);
    create_one(
        request(),
        context.clone(),
        CreateOne {
            entity: NewAccount {
                name: "Created".to_owned(),
                secret: "created".to_owned(),
            },
        },
    )
    .await
    .expect("create should succeed");
    update_one(
        request(),
        context.clone(),
        UpdateOne {
            condition: Some(support::equals(
                "id",
                crudkit_core::condition::ConditionClauseValue::I64(1),
            )),
            entity: AccountUpdate {
                name: "Updated".to_owned(),
                secret: "updated".to_owned(),
            },
        },
    )
    .await
    .expect("update should succeed");
    delete_by_id(
        request(),
        context.clone(),
        DeleteById {
            id: SerializableId(vec![SerializableIdEntry {
                field_name: "id".to_owned(),
                value: IdValue::I64(1),
            }]),
            condition: None,
        },
    )
    .await
    .expect("delete should succeed");
}

/// The kind of each message, as its wire format names it.
fn kinds(messages: &[CollabMessageV1]) -> Vec<&'static str> {
    messages
        .iter()
        .map(|message| match message.event {
            CollabEventV1::EntityCreated { .. } => "entity_created",
            CollabEventV1::EntityUpdated { .. } => "entity_updated",
            CollabEventV1::EntityDeleted { .. } => "entity_deleted",
            CollabEventV1::PartialValidationResult { .. } => "partial_validation_result",
            CollabEventV1::FullValidationResult { .. } => "full_validation_result",
        })
        .collect()
}
