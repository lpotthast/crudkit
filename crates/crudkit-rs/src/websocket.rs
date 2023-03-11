use crudkit_websocket::CkWsMessage;

pub trait CrudWebsocketController {
    fn broadcast_json(&self, json: &CkWsMessage);
}
