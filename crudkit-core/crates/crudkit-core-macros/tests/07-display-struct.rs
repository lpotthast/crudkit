#![allow(dead_code)]
#![allow(unused_variables)]

use assertr::prelude::*;
use crudkit_core_macros::CkId;

#[derive(CkId)]
pub struct Foo {
    #[ck_id(id)]
    id_a: i32,

    #[ck_id(id)]
    id_b: i32,
}

fn main() {
    let id = FooId { id_a: 1, id_b: 2 };

    assert_that(id.to_string()).is_equal_to("(id_a: 1, id_b: 2)".to_string());
}
