#![allow(dead_code)]
#![allow(unused_variables)]

use assertr::prelude::*;
use crudkit_core::id::HasId;
use crudkit_core_macros::CkId;

#[derive(CkId)]
pub struct User {
    #[ck_id(id)]
    user_id: i32,

    name: String,
}

#[derive(CkId)]
pub struct CompositeEntity {
    #[ck_id(id)]
    org_id: i64,

    #[ck_id(id)]
    user_id: i32,

    data: String,
}

fn main() {
    // Test single ID field.
    let user = User {
        user_id: 42,
        name: "Alice".to_string(),
    };
    let user_id: UserId = user.id();
    assert_that(user_id.user_id).is_equal_to(42);

    // Test composite ID fields.
    let entity = CompositeEntity {
        org_id: 100,
        user_id: 42,
        data: "test".to_string(),
    };
    let entity_id: CompositeEntityId = entity.id();
    assert_that(entity_id.org_id).is_equal_to(100);
    assert_that(entity_id.user_id).is_equal_to(42);
}
