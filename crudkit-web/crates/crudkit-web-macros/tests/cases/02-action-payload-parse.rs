use crudkit_web_macros::CkActionPayload;

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, CkActionPayload)]
pub struct Foo {}

fn main() {}
