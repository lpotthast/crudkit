#![allow(dead_code)]

use crudkit_core_macros::CkId;

#[derive(CkId)]
pub struct Foo {
    #[ck_id(id)]
    id: Option<i64>,
}

fn main() {}
