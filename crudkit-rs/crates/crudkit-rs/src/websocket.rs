use crudkit_websocket::CkWsMessage;

pub trait CrudWebsocketController {
    type Error;

    fn broadcast_json(&self, json: CkWsMessage) -> Result<(), Self::Error>;
}
