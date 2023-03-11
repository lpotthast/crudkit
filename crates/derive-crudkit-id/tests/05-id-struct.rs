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
    let id = FooId { id_a: 1, id_b: 2 };

    use crud_id::Id;

    let expected = vec![FooIdField::IdA(1), FooIdField::IdB(2)];
    assert_eq!(expected, id.fields());
    assert_eq!(expected, id.fields_iter().collect::<Vec<_>>());

    use crud_id::IdField;

    assert_eq!(
        crud_id::SerializableId(
            id.fields_iter()
                .map(|field| (field.name().to_owned(), field.to_value(),))
                .collect()
        ),
        id.into_serializable_id()
    );
}
