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
    let id_a = FooIdField::IdA(42);

    assert_that(id_a.to_string()).is_equal_to("42".to_string());
}
