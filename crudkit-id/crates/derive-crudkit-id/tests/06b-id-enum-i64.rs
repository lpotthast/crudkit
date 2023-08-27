#![allow(dead_code)]
#![allow(unused_variables)]

use derive_crudkit_id::CkId;

#[derive(CkId)]
pub struct Foo {
    #[ck_id(id)]
    id_a: i64,

    #[ck_id(id)]
    id_b: i64,
}

fn main() {
    let field_a = FooIdField::IdA(-1337);
    let field_b = FooIdField::IdB(1337);

    use crudkit_id::IdField;
    assert_eq!("id_a", field_a.name());
    assert_eq!("id_b", field_b.name());

    use crudkit_id::IdValue;
    assert_eq!(IdValue::I64(-1337), field_a.to_value());
    assert_eq!(IdValue::I64(1337), field_b.to_value());
}
