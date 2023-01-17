use axum_websockets::WebsocketController;
use std::sync::Arc;

// TODO: Get rid of this? Use aggregate specific data all the way. See context.rs and the repository field as an example.
pub struct CrudController {
    ws: Arc<WebsocketController>,
}

impl CrudController {
    pub fn new(ws: Arc<WebsocketController>) -> Self {
        Self { ws }
    }

    pub fn get_websocket_controller(&self) -> &WebsocketController {
        self.ws.as_ref()
    }
}
