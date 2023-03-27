#![allow(dead_code)]

use derive_crudkit_id::CrudId;

#[derive(CrudId)]
pub struct Foo {
    #[ck_id(id)]
    foo: i32,
}

fn main() {}
