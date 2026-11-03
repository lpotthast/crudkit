//! Deleting by ID refuses an ID that is not an ID of the resource before the repository sees it,
//! because a weaker condition than the ID could select and delete other entities.

mod support;

use std::sync::Arc;

use assertr::prelude::*;
use axum::http::StatusCode;
use crudkit_core::condition::{IdConditionError, TryIntoAllEqualCondition, condition_matching_id};
use crudkit_core::id::{HasId, Id, IdValue, SerializableId, SerializableIdEntry};
use crudkit_rs::crudkit_wire_format::v1::DeleteByIdV1;
use crudkit_rs::prelude::*;
use futures::executor::block_on;
use support::{Accounts, Clients, MemoryRepository, account, context, post};

#[test]
fn delete_by_id_refuses_ids_that_are_not_ids_of_the_resource() {
    let cases = [
        ("an ID without fields", id(&[])),
        ("a field without a name", id(&[("", IdValue::I64(1))])),
        (
            "a missing field",
            id(&[("name", IdValue::String("Account 1".to_owned()))]),
        ),
        ("a misnamed field", id(&[("ID", IdValue::I64(1))])),
        (
            "a mistyped field",
            id(&[("id", IdValue::String("1".to_owned()))]),
        ),
        (
            "an extra field",
            id(&[
                ("id", IdValue::I64(1)),
                ("name", IdValue::String("Account 2".to_owned())),
            ]),
        ),
        (
            "a repeated field",
            id(&[("id", IdValue::I64(1)), ("id", IdValue::I64(2))]),
        ),
    ];
    for (case, id) in cases {
        // Arrange
        let context = context::<RestrictedAuthPolicy>(
            MemoryRepository::with((1..=3).map(account).collect()),
            Arc::new(Clients::default()),
        );

        // Act
        let (status, body) = block_on(delete_by_id(&context, id));

        // Assert
        assert_that!(status)
            .with_detail_message(case)
            .is_equal_to(StatusCode::BAD_REQUEST);
        assert_that!(body["error"]["kind"].clone())
            .with_detail_message(case)
            .is_equal_to(serde_json::json!("bad_request"));
        assert_that!(context.repository.calls())
            .with_detail_message(case)
            .is_equal_to(0);
        assert_that!(context.repository.accounts())
            .with_detail_message(case)
            .has_length(3);
    }
}

#[test]
fn delete_by_id_deletes_the_entity_of_its_id() {
    // Arrange
    let context = context::<RestrictedAuthPolicy>(
        MemoryRepository::with((1..=3).map(account).collect()),
        Arc::new(Clients::default()),
    );

    // Act
    let (status, body) = block_on(delete_by_id(&context, id(&[("id", IdValue::I64(2))])));

    // Assert
    assert_that!(status).is_equal_to(StatusCode::OK);
    assert_that!(body["entities_affected"].clone()).is_equal_to(serde_json::json!(1));
    assert_that!(context.repository.accounts()).is_equal_to(vec![account(1), account(3)]);
}

/// A membership in a club for a season, which the club and the season identify.
#[derive(Clone, Debug, PartialEq, Eq, CkId)]
struct Membership {
    #[ck_id(id)]
    club_id: i64,
    #[ck_id(id)]
    season: String,
}

#[test]
fn composite_ids_build_a_condition_only_with_exactly_their_fields() {
    // Arrange
    let membership = Membership {
        club_id: 4,
        season: "2024".to_owned(),
    };
    let club = || ("club_id", IdValue::I64(4));
    let season = || ("season", IdValue::String("2024".to_owned()));
    let expected = id(&[club(), season()])
        .into_entries()
        .try_into_all_equal_condition();

    // Act
    let complete = condition_matching_id::<MembershipId>(&membership.id().to_serializable_id());
    let reordered = condition_matching_id::<MembershipId>(&id(&[season(), club()]));
    let refused = [
        id(&[club()]),
        id(&[club(), ("season", IdValue::I64(2024))]),
        id(&[
            club(),
            season(),
            ("role", IdValue::String("coach".to_owned())),
        ]),
        id(&[club(), season(), club()]),
    ]
    .map(|id| condition_matching_id::<MembershipId>(&id));

    // Assert
    assert_that!(expected.clone()).is_ok();
    assert_that!(complete).is_equal_to(expected.clone());
    assert_that!(reordered).is_equal_to(expected);
    for result in refused {
        assert_that!(result).is_equal_to(Err(IdConditionError::MismatchedIdFields));
    }
}

fn id(fields: &[(&str, IdValue)]) -> SerializableId {
    SerializableId(
        fields
            .iter()
            .map(|(field_name, value)| SerializableIdEntry {
                field_name: (*field_name).to_owned(),
                value: value.clone(),
            })
            .collect(),
    )
}

/// Requests the deletion of the account `id` through the generated route.
async fn delete_by_id(
    context: &Arc<CrudContext<Accounts<RestrictedAuthPolicy>>>,
    id: SerializableId,
) -> (StatusCode, serde_json::Value) {
    let body = DeleteByIdV1::from(DeleteById {
        id,
        condition: None,
    });
    post(context, "delete-by-id", &body).await
}
