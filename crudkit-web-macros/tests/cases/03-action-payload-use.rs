use crudkit_web_macros::CkActionPayload;

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, CkActionPayload)]
pub struct Foo {}

fn send_payload<P: crudkit_web::action::CrudActionPayload>(payload: P) {
    println!("{payload:?}")
}

fn main() {
    let foo = Foo {};
    send_payload(foo);
}
