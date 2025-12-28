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

    use crudkit_id::Id;

    let expected = vec![FooIdField::IdA(1), FooIdField::IdB(2)];
    assert_eq!(expected, id.fields());
    assert_eq!(expected, id.fields_iter().collect::<Vec<_>>());

    use crudkit_id::IdField;

    assert_eq!(
        crudkit_id::SerializableId(
            id.fields_iter()
                .map(|field| (field.name().to_owned(), field.to_value(),))
                .collect()
        ),
        id.to_serializable_id()
    );
}
