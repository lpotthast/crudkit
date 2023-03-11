#![allow(dead_code)]

use derive_crud_id::CrudId;

#[derive(CrudId)]
pub struct Foo {
    id: i32,
}

fn main() {}
