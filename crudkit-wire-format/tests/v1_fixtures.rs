//! Pins the JSON of every version 1 type to a hand-written fixture.
//!
//! A failing test here means the serialized form of version 1 changed. Version 1 is released: never
//! update a fixture to make a test pass; add a new version instead.

use assertr::prelude::*;
use crudkit_wire_format::v1::{
    CollabEventV1, CollabMessageV1, FullViolationsV1, ResourceFullViolationsV1,
    ResourcePartialViolationsV1,
};
use crudkit_wire_format::v1::{
    ConditionClauseV1, ConditionClauseValueV1, ConditionElementV1, ConditionV1, CreateOneV1,
    DeleteByIdV1, DeleteManyV1, DeleteOneV1, IdValueV1, OperatorV1, OrderV1, ReadCountResponseV1,
    ReadCountV1, ReadManyResponseV1, ReadManyV1, ReadOneResponseV1, ReadOneV1,
    SerializableIdEntryV1, SerializableIdV1, UpdateOneV1, WireFormatVersionV1,
};
use crudkit_wire_format::v1::{
    DeletedManyV1, DeletedV1, EntityViolationsV1, ErrorResponseV1, ErrorV1, FailedDeletionV1,
    PartialViolationsV1, SavedV1, SeverityV1, ViolationV1,
};
use indexmap::IndexMap;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use time::macros::datetime;

fn fixture(name: &str) -> serde_json::Value {
    let path = format!("{}/tests/fixtures/v1/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).expect("fixture should be readable");
    serde_json::from_str(&text).expect("fixture should be valid JSON")
}

/// Asserts that `value` serializes to the fixture `name` and that the fixture deserializes to it.
fn assert_matches_fixture<T>(name: &str, value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let expected = fixture(name);
    let serialized = serde_json::to_value(value).expect("value should serialize");
    assert_that!(serialized).is_equal_to(expected.clone());
    let deserialized: T = serde_json::from_value(expected).expect("fixture should deserialize");
    assert_that!(deserialized == *value).is_true();
}

fn clause(
    column_name: &str,
    operator: OperatorV1,
    value: ConditionClauseValueV1,
) -> ConditionElementV1 {
    ConditionElementV1::Clause(ConditionClauseV1 {
        column_name: column_name.to_owned(),
        operator,
        value,
    })
}

fn house_is_gryffindor() -> ConditionV1 {
    ConditionV1::All(vec![clause(
        "house",
        OperatorV1::Equal,
        ConditionClauseValueV1::String("Gryffindor".to_owned()),
    )])
}

fn uuid() -> uuid::Uuid {
    uuid::Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").expect("valid UUID")
}

fn person(id: i64, first_name: &str) -> serde_json::Value {
    serde_json::json!({ "id": id, "first_name": first_name })
}

#[test]
fn read_count() {
    assert_matches_fixture(
        "read_count.json",
        &ReadCountV1 {
            wire_format_version: WireFormatVersionV1,
            condition: Some(house_is_gryffindor()),
        },
    );
}

#[test]
fn read_one() {
    assert_matches_fixture(
        "read_one.json",
        &ReadOneV1 {
            wire_format_version: WireFormatVersionV1,
            skip: Some(2),
            order_by: Some(IndexMap::from([("last_name".to_owned(), OrderV1::Asc)])),
            condition: Some(ConditionV1::Any(vec![
                clause(
                    "year_of_birth",
                    OperatorV1::GreaterOrEqual,
                    ConditionClauseValueV1::I32(1980),
                ),
                ConditionElementV1::Condition(Box::new(ConditionV1::All(vec![clause(
                    "house",
                    OperatorV1::NotEqual,
                    ConditionClauseValueV1::String("Slytherin".to_owned()),
                )]))),
            ])),
        },
    );
}

#[test]
fn read_many() {
    assert_matches_fixture(
        "read_many.json",
        &ReadManyV1 {
            wire_format_version: WireFormatVersionV1,
            limit: Some(10),
            skip: Some(20),
            order_by: Some(IndexMap::from([
                ("last_name".to_owned(), OrderV1::Asc),
                ("first_name".to_owned(), OrderV1::Desc),
            ])),
            condition: None,
        },
    );
}

#[test]
fn create_one() {
    assert_matches_fixture(
        "create_one.json",
        &CreateOneV1 {
            wire_format_version: WireFormatVersionV1,
            entity: serde_json::json!({ "name": "Hufflepuff" }),
        },
    );
}

#[test]
fn update_one() {
    assert_matches_fixture(
        "update_one.json",
        &UpdateOneV1 {
            wire_format_version: WireFormatVersionV1,
            condition: Some(ConditionV1::All(vec![clause(
                "id",
                OperatorV1::Equal,
                ConditionClauseValueV1::I64(4),
            )])),
            entity: serde_json::json!({ "id": 4, "name": "Ravenclaw" }),
        },
    );
}

#[test]
fn delete_by_id() {
    let entry = |field_name: &str, value: i64| SerializableIdEntryV1 {
        field_name: field_name.to_owned(),
        value: IdValueV1::I64(value),
    };
    assert_matches_fixture(
        "delete_by_id.json",
        &DeleteByIdV1 {
            wire_format_version: WireFormatVersionV1,
            id: SerializableIdV1(vec![entry("club_id", 1), entry("person_id", 7)]),
            condition: Some(house_is_gryffindor()),
        },
    );
    assert_matches_fixture(
        "delete_by_id_without_condition.json",
        &DeleteByIdV1 {
            wire_format_version: WireFormatVersionV1,
            id: SerializableIdV1(vec![entry("id", 42)]),
            condition: None,
        },
    );
}

#[test]
fn delete_one() {
    assert_matches_fixture(
        "delete_one.json",
        &DeleteOneV1 {
            wire_format_version: WireFormatVersionV1,
            skip: None,
            order_by: Some(IndexMap::from([("created_at".to_owned(), OrderV1::Desc)])),
            condition: Some(ConditionV1::All(Vec::new())),
        },
    );
}

#[test]
fn delete_many() {
    assert_matches_fixture(
        "delete_many.json",
        &DeleteManyV1 {
            wire_format_version: WireFormatVersionV1,
            condition: Some(ConditionV1::Any(Vec::new())),
        },
    );
}

#[test]
fn read_responses() {
    assert_matches_fixture(
        "read_count_response.json",
        &ReadCountResponseV1 {
            wire_format_version: WireFormatVersionV1,
            count: 27,
        },
    );
    assert_matches_fixture(
        "read_one_response.json",
        &ReadOneResponseV1 {
            wire_format_version: WireFormatVersionV1,
            entity: Some(person(1, "Ron")),
        },
    );
    assert_matches_fixture(
        "read_one_response_without_entity.json",
        &ReadOneResponseV1::<serde_json::Value> {
            wire_format_version: WireFormatVersionV1,
            entity: None,
        },
    );
    assert_matches_fixture(
        "read_many_response.json",
        &ReadManyResponseV1 {
            wire_format_version: WireFormatVersionV1,
            entities: vec![person(1, "Ron"), person(2, "Seamus")],
        },
    );
}

#[test]
fn every_operator() {
    assert_matches_fixture(
        "operators.json",
        &vec![
            OperatorV1::Equal,
            OperatorV1::NotEqual,
            OperatorV1::Less,
            OperatorV1::LessOrEqual,
            OperatorV1::Greater,
            OperatorV1::GreaterOrEqual,
            OperatorV1::IsIn,
        ],
    );
}

#[test]
fn every_order_and_its_aliases() {
    assert_matches_fixture("orders.json", &vec![OrderV1::Asc, OrderV1::Desc]);
    let aliases: Vec<OrderV1> = serde_json::from_value(serde_json::json!([
        "ascending",
        "Asc",
        "descending",
        "Desc"
    ]))
    .expect("aliases should deserialize");
    assert_that!(aliases).is_equal_to(vec![
        OrderV1::Asc,
        OrderV1::Asc,
        OrderV1::Desc,
        OrderV1::Desc,
    ]);
}

#[test]
fn every_condition_clause_value() {
    assert_matches_fixture(
        "condition_clause_values.json",
        &vec![
            ConditionClauseValueV1::Bool(true),
            ConditionClauseValueV1::U8(8),
            ConditionClauseValueV1::U16(16),
            ConditionClauseValueV1::U32(32),
            ConditionClauseValueV1::U64(64),
            ConditionClauseValueV1::U128(128),
            ConditionClauseValueV1::I8(-8),
            ConditionClauseValueV1::I16(-16),
            ConditionClauseValueV1::I32(-32),
            ConditionClauseValueV1::I64(-64),
            ConditionClauseValueV1::I128(-128),
            ConditionClauseValueV1::F32(1.5),
            ConditionClauseValueV1::F64(2.25),
            ConditionClauseValueV1::String("Quaffle".to_owned()),
            ConditionClauseValueV1::Json(serde_json::json!({ "positions": ["Chaser", "Keeper"] })),
            ConditionClauseValueV1::Uuid(uuid()),
            ConditionClauseValueV1::U8Vec(vec![1, 2]),
            ConditionClauseValueV1::I32Vec(vec![-1, 3]),
            ConditionClauseValueV1::I64Vec(vec![7]),
        ],
    );
}

#[test]
fn every_id_value() {
    assert_matches_fixture(
        "id_values.json",
        &vec![
            IdValueV1::I8(-8),
            IdValueV1::I16(-16),
            IdValueV1::I32(-32),
            IdValueV1::I64(-64),
            IdValueV1::I128(-128),
            IdValueV1::U8(8),
            IdValueV1::U16(16),
            IdValueV1::U32(32),
            IdValueV1::U64(64),
            IdValueV1::U128(128),
            IdValueV1::Bool(false),
            IdValueV1::String("Snitch".to_owned()),
            IdValueV1::Uuid(uuid()),
            IdValueV1::PrimitiveDateTime(datetime!(2026-10-05 13:07:09)),
            IdValueV1::PrimitiveDateTime(datetime!(2026-10-05 13:07:09.25)),
            IdValueV1::OffsetDateTime(datetime!(2026-10-05 13:07:09 +02:00)),
            IdValueV1::OffsetDateTime(datetime!(2026-10-05 13:07:09.5 UTC)),
        ],
    );
}

#[test]
fn bodies_of_other_versions_are_rejected() {
    let mut body = fixture("read_count_response.json");
    body["wire_format_version"] = serde_json::json!(2);
    assert_that!(serde_json::from_value::<ReadCountResponseV1>(body).is_err()).is_true();

    let mut body = fixture("read_count_response.json");
    body.as_object_mut()
        .expect("fixture is an object")
        .remove("wire_format_version");
    assert_that!(serde_json::from_value::<ReadCountResponseV1>(body).is_err()).is_true();
}

fn id(value: i64) -> SerializableIdV1 {
    SerializableIdV1(vec![SerializableIdEntryV1 {
        field_name: "id".to_owned(),
        value: IdValueV1::I64(value),
    }])
}

fn violation(severity: SeverityV1, message: &str) -> ViolationV1 {
    ViolationV1 {
        severity,
        message: message.to_owned(),
    }
}

#[test]
fn saved() {
    assert_matches_fixture(
        "saved.json",
        &SavedV1 {
            wire_format_version: WireFormatVersionV1,
            entity: person(7, "Ginny"),
            violations: PartialViolationsV1 {
                general: None,
                create: Some(Vec::new()),
                by_entity: vec![EntityViolationsV1 {
                    id: id(7),
                    violations: vec![
                        violation(SeverityV1::Major, "No position assigned."),
                        violation(SeverityV1::Critical, "Year of birth is in the future."),
                    ],
                }],
            },
        },
    );
}

#[test]
fn deleted() {
    assert_matches_fixture(
        "deleted.json",
        &DeletedV1 {
            wire_format_version: WireFormatVersionV1,
            entities_affected: 1,
        },
    );
    assert_matches_fixture(
        "deleted_many.json",
        &DeletedManyV1 {
            wire_format_version: WireFormatVersionV1,
            deleted_count: 1,
            deleted_ids: vec![id(1)],
            aborted: vec![FailedDeletionV1 {
                id: id(2),
                reason: "Team captains cannot be deleted.".to_owned(),
            }],
            validation_failed: vec![id(3)],
            errors: vec![FailedDeletionV1 {
                id: id(4),
                reason: "Repository error.".to_owned(),
            }],
        },
    );
}

#[test]
fn every_error_kind() {
    let error = |error: ErrorV1| ErrorResponseV1 {
        wire_format_version: WireFormatVersionV1,
        error,
    };
    let message = |text: &str| text.to_owned();
    assert_matches_fixture(
        "errors.json",
        &vec![
            error(ErrorV1::BadRequest {
                message: message("unknown field 'hat'"),
            }),
            error(ErrorV1::Unauthorized {
                message: message("Authentication required"),
            }),
            error(ErrorV1::Forbidden {
                message: message("Only the captain may do this."),
            }),
            error(ErrorV1::NotFound {
                message: message("Not found"),
            }),
            error(ErrorV1::UnprocessableEntity {
                message: message("The season has ended."),
            }),
            error(ErrorV1::CriticalValidationErrors {
                message: message("Critical validation errors prevent the operation."),
                violations: PartialViolationsV1 {
                    general: Some(vec![violation(SeverityV1::Critical, "Too many Seekers.")]),
                    create: None,
                    by_entity: Vec::new(),
                },
            }),
            error(ErrorV1::InternalServerError {
                message: message("Repository error."),
            }),
        ],
    );
}

#[test]
fn every_collaboration_event() {
    let message = |event: CollabEventV1| CollabMessageV1 {
        wire_format_version: WireFormatVersionV1,
        event,
    };
    let people = || "people".to_owned();
    assert_matches_fixture(
        "collab_messages.json",
        &vec![
            message(CollabEventV1::EntityCreated {
                resource_name: people(),
                entity_id: id(28),
                with_validation_errors: true,
            }),
            message(CollabEventV1::EntityUpdated {
                resource_name: people(),
                entity_id: id(28),
                with_validation_errors: false,
            }),
            message(CollabEventV1::EntityDeleted {
                resource_name: people(),
                entity_id: id(28),
            }),
            message(CollabEventV1::PartialValidationResult {
                resources: vec![ResourcePartialViolationsV1 {
                    resource_name: people(),
                    violations: PartialViolationsV1 {
                        general: None,
                        create: None,
                        by_entity: vec![EntityViolationsV1 {
                            id: id(28),
                            violations: vec![violation(
                                SeverityV1::Major,
                                "Gender 'nargle' is unknown.",
                            )],
                        }],
                    },
                }],
            }),
            message(CollabEventV1::FullValidationResult {
                resources: vec![
                    ResourceFullViolationsV1 {
                        resource_name: "clubs".to_owned(),
                        violations: FullViolationsV1::default(),
                    },
                    ResourceFullViolationsV1 {
                        resource_name: people(),
                        violations: FullViolationsV1 {
                            general: vec![violation(SeverityV1::Critical, "Too many Seekers.")],
                            by_entity: vec![EntityViolationsV1 {
                                id: id(28),
                                violations: Vec::new(),
                            }],
                        },
                    },
                ],
            }),
        ],
    );
}
