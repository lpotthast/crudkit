use derive_crud_action_payload::CrudActionPayload;

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, CrudActionPayload)]
pub struct Foo {}

fn send_payload<P: crud_yew::CrudActionPayload>(payload: P) {
    println!("{payload:?}")
}

fn main() {
    let foo = Foo {};
    send_payload(foo);
}
