use crudkit_web::data_source::CrudVersion;
use crudkit_web_macros::CkField;

#[derive(Clone, PartialEq, Eq, Debug, CkField, serde::Serialize, serde::Deserialize)]
#[ck_field(model = Read)]
pub struct Foo {
    pub id: i64,
    #[ck_field(version)]
    pub version: CrudVersion,
    #[ck_field(version)]
    pub etag: CrudVersion,
}

fn main() {}
