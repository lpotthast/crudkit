#![allow(dead_code)]
#![allow(unused_variables)]

use derive_crudkit_id::CrudId;

#[derive(CrudId)]
pub struct Foo {
    #[crudkit_id(id)]
    id_a: i32,

    #[crudkit_id(id)]
    id_b: i32,
}

fn main() {
    let id_a = FooIdField::IdA(42);

    assert_eq!("42", id_a.to_string());
}
