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
    let field_a = FooIdField::IdA(-1337);
    let field_b = FooIdField::IdB(1337);

    use crudkit_core::id::IdField;
    assert_that(field_a.name()).is_equal_to("id_a");
    assert_that(field_b.name()).is_equal_to("id_b");

    use crudkit_core::id::IdValue;
    assert_that(field_a.to_value()).is_equal_to(IdValue::I32(-1337));
    assert_that(field_b.to_value()).is_equal_to(IdValue::I32(1337));
}
