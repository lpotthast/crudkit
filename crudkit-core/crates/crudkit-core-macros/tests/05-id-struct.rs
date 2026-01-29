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

    use crudkit_core::id::Id;

    let expected = vec![FooIdField::IdA(1), FooIdField::IdB(2)];
    assert_that(id.fields()).is_equal_to(expected.clone());
    assert_that(id.fields_iter().collect::<Vec<_>>()).is_equal_to(expected);

    use crudkit_core::id::IdField;

    let expected = crudkit_core::id::SerializableId(
        id.fields_iter()
            .map(|field| (field.name().to_owned(), field.to_value()).into())
            .collect(),
    );
    assert_that(id.to_serializable_id()).is_equal_to(expected);
}
