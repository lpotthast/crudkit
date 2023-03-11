#![allow(dead_code)]
#![allow(unused_variables)]

use derive_crud_id::CrudId;

#[derive(CrudId)]
pub struct Foo {
    #[crud_id(id)]
    id_a: i32,

    #[crud_id(id)]
    id_b: i32,
}

fn main() {
    let id_a = FooIdField::IdA(42);

    assert_eq!("42", id_a.to_string());
}
