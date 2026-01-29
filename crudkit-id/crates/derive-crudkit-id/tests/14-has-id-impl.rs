#![allow(dead_code)]
#![allow(unused_variables)]

use crudkit_id::HasId;
use derive_crudkit_id::CkId;

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
    assert_eq!(user_id.user_id, 42);

    // Test composite ID fields.
    let entity = CompositeEntity {
        org_id: 100,
        user_id: 42,
        data: "test".to_string(),
    };
    let entity_id: CompositeEntityId = entity.id();
    assert_eq!(entity_id.org_id, 100);
    assert_eq!(entity_id.user_id, 42);
}
