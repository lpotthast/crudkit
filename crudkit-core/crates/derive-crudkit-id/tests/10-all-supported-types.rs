#![allow(dead_code)]
#![allow(unused_variables)]

use derive_crudkit_id::CkId;

#[derive(CkId)]
pub struct Foo {
    #[ck_id(id)]
    id_i8: i8,

    #[ck_id(id)]
    id_i16: i16,

    #[ck_id(id)]
    id_i32: i32,

    #[ck_id(id)]
    id_i64: i64,

    #[ck_id(id)]
    id_i128: i128,

    #[ck_id(id)]
    id_u8: u8,

    #[ck_id(id)]
    id_u16: u16,

    #[ck_id(id)]
    id_u32: u32,

    #[ck_id(id)]
    id_u64: u64,

    #[ck_id(id)]
    id_u128: u128,

    #[ck_id(id)]
    id_bool: bool,

    #[ck_id(id)]
    id_string: String,

    #[ck_id(id)]
    id_uuid: uuid::Uuid,

    #[ck_id(id)]
    id_primitive_date_time: time::PrimitiveDateTime,

    #[ck_id(id)]
    id_offset_date_time: time::OffsetDateTime,
}

fn main() {
    let now = time::OffsetDateTime::now_utc();
    let _id = FooId {
        id_i8: 42,
        id_i16: 42,
        id_i32: 42,
        id_i64: 42,
        id_i128: 42,
        id_u8: 42,
        id_u16: 42,
        id_u32: 42,
        id_u64: 42,
        id_u128: 42,
        id_bool: true,
        id_string: "some_string".to_owned(),
        id_uuid: uuid::Uuid::new_v4(),
        id_primitive_date_time: time::PrimitiveDateTime::new(now.date(), now.time()),
        id_offset_date_time: now,
    };
}
