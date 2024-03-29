#![allow(dead_code)]
#![allow(unused_variables)]

use derive_crudkit_id::CkId;

#[derive(CkId)]
pub struct Foo {
    #[ck_id(id)]
    id_a: i32,

    #[ck_id(id)]
    id_b: i32,
}

fn main() {
    let id = FooId {
        id_a: 1,
        id_b: 2,
    };

    assert_eq!("(id_a: 1, id_b: 2)", id.to_string());
}
