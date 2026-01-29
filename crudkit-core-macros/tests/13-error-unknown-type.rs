#![allow(dead_code)]

use crudkit_core_macros::CkId;

#[derive(CkId)]
pub struct Foo {
    #[ck_id(id)]
    id: Vec<String>,
}

fn main() {}
