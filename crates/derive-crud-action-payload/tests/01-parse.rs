use derive_crud_action_payload::CrudActionPayload;

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, CrudActionPayload)]
pub struct Foo {}

fn main() {}
