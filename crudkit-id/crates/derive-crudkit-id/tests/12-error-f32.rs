#![allow(dead_code)]

use derive_crudkit_id::CkId;

#[derive(CkId)]
pub struct Foo {
    #[ck_id(id)]
    id: f32,
}

fn main() {}
