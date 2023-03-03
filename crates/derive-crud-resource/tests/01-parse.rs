use derive_crud_resource::CrudResource;

// TODO: Work in progress...

#[derive(CrudResource)]
#[crud(resource_name = "foo")]
pub struct Foo {
    id: i32,
    name: String,
}

pub struct CreateFoo {}

pub struct ReadFoo {}
pub struct ReadFooId {}
pub enum ReadFooIdField {}

pub struct FooId {}
pub enum FooIdField {}

fn main() {}
