#![allow(dead_code)]

use derive_crudkit_id::CrudId;

#[derive(CrudId)]
pub struct Foo {
    foo: i32,
}

fn main() {}
