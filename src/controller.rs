use axum_websockets::WebsocketController;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct CrudController {
    db: Arc<DatabaseConnection>,
    ws: Arc<WebsocketController>,
}

impl CrudController {
    pub fn new(db: Arc<DatabaseConnection>, ws: Arc<WebsocketController>) -> Self {
        Self { db, ws }
    }

    pub fn get_database_connection(&self) -> &DatabaseConnection {
        self.db.as_ref()
    }

    pub fn get_websocket_controller(&self) -> &WebsocketController {
        &self.ws.as_ref()
    }
}
