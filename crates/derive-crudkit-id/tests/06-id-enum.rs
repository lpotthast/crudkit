#![allow(dead_code)]
#![allow(unused_variables)]

use derive_crud_id::CrudId;

#[derive(CrudId)]
pub struct Foo {
    #[crud_id(id)]
    id_a: i32,

    #[crud_id(id)]
    id_b: i32,
}

fn main() {
    let field_a = FooIdField::IdA(-1337);
    let field_b = FooIdField::IdB(1337);

    use crud_id::IdField;
    assert_eq!("id_a", field_a.name());
    assert_eq!("id_b", field_b.name());

    use crud_id::IdValue;
    assert_eq!(IdValue::I32(-1337), field_a.to_value());
    assert_eq!(IdValue::I32(1337), field_b.to_value());
}
