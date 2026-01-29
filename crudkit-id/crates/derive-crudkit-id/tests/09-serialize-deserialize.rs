#![allow(dead_code)]
#![allow(unused_variables)]

use assertr::prelude::*;
use derive_crudkit_id::CkId;

#[derive(CkId)]
pub struct Foo {
    #[ck_id(id)]
    id_a: i32,

    #[ck_id(id)]
    id_b: i32,
}

fn main() {
    let id = FooId { id_a: 1, id_b: 2 };
    let serialized = "{\"id_a\":1,\"id_b\":2}";
    assert_that(serde_json::to_string(&id).unwrap()).is_equal_to(serialized);
    assert_that(serde_json::from_str::<FooId>(serialized).unwrap()).is_equal_to(id);

    let id_a = FooIdField::IdA(12);
    let serialized = "{\"IdA\":12}";
    assert_that(serde_json::to_string(&id_a).unwrap()).is_equal_to(serialized);
    assert_that(serde_json::from_str::<FooIdField>(serialized).unwrap()).is_equal_to(id_a);
}
