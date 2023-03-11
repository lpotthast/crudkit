use std::sync::Arc;

pub trait CkWebsocketController {
    type Json;

    fn broadcast_json(Self::Json);
}

// TODO: Get rid of this? Use aggregate specific data all the way. See context.rs and the repository field as an example.
pub struct CrudController {
    ws: Arc<dyn CkWebsocketController>,
}

impl CrudController {
    pub fn new(ws: Arc<dyn CkWebsocketController>) -> Self {
        Self { ws }
    }

    pub fn get_websocket_controller(&self) -> &dyn CkWebsocketController {
        self.ws.as_ref()
    }
}
