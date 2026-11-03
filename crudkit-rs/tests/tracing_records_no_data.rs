//! The operations trace which resource and entity they act on, but never the data of a request or
//! of an entity, because it may hold secrets or personal data.

mod support;

use std::io;
use std::sync::{Arc, Mutex};

use assertr::prelude::*;
use crudkit_core::condition::{Condition, ConditionClauseValue};
use crudkit_core::id::{IdValue, SerializableId, SerializableIdEntry};
use crudkit_rs::prelude::*;
use futures::executor::block_on;
use support::{
    AccountUpdate, Accounts, Clients, MemoryRepository, NewAccount, SECRET, Token, account,
    context, equals,
};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::util::SubscriberInitExt;

/// The text that a subscriber writes, kept in memory.
#[derive(Clone, Default)]
struct Logs(Arc<Mutex<Vec<u8>>>);

impl Logs {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().expect("logs should not be poisoned")).into_owned()
    }
}

impl io::Write for Logs {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .expect("logs should not be poisoned")
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn operations_trace_the_resource_and_ids_but_no_data() {
    // Arrange
    let logs = Logs::default();
    let writer = logs.clone();
    let _subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_writer(move || writer.clone())
        .finish()
        .set_default();
    let context = context::<RestrictedAuthPolicy>(
        MemoryRepository::with((1..=3).map(account).collect()),
        Arc::new(Clients::default()),
    );

    // Act
    block_on(read_by_secret(&context));
    block_on(write_secrets(&context));

    // Assert
    let text = logs.text();
    assert_that!(context.repository.accounts()).is_empty();
    assert_that!(text.as_str()).does_not_contain(SECRET);
    assert_that!(text.as_str()).contains("read_one{resource=\"accounts\" entity_id=id=1}");
    assert_that!(text.as_str()).contains("read_many{resource=\"accounts\"}");
    assert_that!(text.as_str()).contains("create_one{resource=\"accounts\" entity_id=id=4}");
    assert_that!(text.as_str()).contains("update_one{resource=\"accounts\" entity_id=id=1}");
    assert_that!(text.as_str()).contains("delete_by_id{resource=\"accounts\" entity_id=id=1}");
    assert_that!(text.as_str()).contains("delete_one{resource=\"accounts\" entity_id=id=2}");
}

fn request() -> RequestContext<Token> {
    RequestContext::authenticated(Token)
}

/// The condition selecting the accounts whose secret is [`SECRET`].
fn secret() -> Condition {
    equals("secret", ConditionClauseValue::String(SECRET.to_owned()))
}

/// Counts and reads the accounts by their secret.
async fn read_by_secret(context: &Arc<CrudContext<Accounts<RestrictedAuthPolicy>>>) {
    read_count(
        request(),
        context.clone(),
        ReadCount {
            condition: Some(secret()),
        },
    )
    .await
    .expect("count should succeed");
    read_one(
        request(),
        context.clone(),
        ReadOne {
            skip: None,
            order_by: None,
            condition: Some(secret()),
        },
    )
    .await
    .expect("read one should succeed");
    read_many(
        request(),
        context.clone(),
        ReadMany {
            limit: None,
            skip: None,
            order_by: None,
            condition: Some(secret()),
        },
    )
    .await
    .expect("read many should succeed");
    // The repository rejects this value, and its error describes what it could not convert.
    read_many(
        request(),
        context.clone(),
        ReadMany {
            limit: None,
            skip: None,
            order_by: None,
            condition: Some(equals(
                "id",
                ConditionClauseValue::Json(serde_json::json!(SECRET)),
            )),
        },
    )
    .await
    .expect_err("a JSON value should not match an ID");
}

/// Creates and updates accounts with secrets, then deletes every account by its secret.
async fn write_secrets(context: &Arc<CrudContext<Accounts<RestrictedAuthPolicy>>>) {
    create_one(
        request(),
        context.clone(),
        CreateOne {
            entity: NewAccount {
                name: "Created".to_owned(),
                secret: SECRET.to_owned(),
            },
        },
    )
    .await
    .expect("create should succeed");
    update_one(
        request(),
        context.clone(),
        UpdateOne {
            condition: Some(equals("id", ConditionClauseValue::I64(1))),
            entity: AccountUpdate {
                name: "Updated".to_owned(),
                secret: SECRET.to_owned(),
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
            condition: Some(secret()),
        },
    )
    .await
    .expect("delete by ID should succeed");
    delete_one(
        request(),
        context.clone(),
        DeleteOne {
            skip: None,
            order_by: None,
            condition: Some(secret()),
        },
    )
    .await
    .expect("delete one should succeed");
    delete_many(
        request(),
        context.clone(),
        DeleteMany {
            condition: Some(secret()),
        },
    )
    .await
    .expect("delete many should succeed");
}
