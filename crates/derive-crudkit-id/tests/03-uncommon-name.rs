#![allow(dead_code)]

use derive_crudkit_id::CrudId;

#[derive(CrudId)]
pub struct Foo {
    #[crudkit_id(id)]
    foo: i32,
}

fn main() {}
