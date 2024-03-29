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
    let id = FooId { id_a: 1, id_b: 2 };
    let serialized = "{\"id_a\":1,\"id_b\":2}";
    assert_eq!(serialized, serde_json::to_string(&id).unwrap().as_str());
    assert_eq!(id, serde_json::from_str(serialized).unwrap());

    let id_a = FooIdField::IdA(12);
    let serialized = "{\"IdA\":12}";
    assert_eq!(serialized, serde_json::to_string(&id_a).unwrap().as_str());
    assert_eq!(id_a, serde_json::from_str(serialized).unwrap());
}
