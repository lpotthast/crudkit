use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct CrudController {
    db: Arc<DatabaseConnection>,
}

impl CrudController {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub fn get_database_connection(&self) -> &DatabaseConnection {
        self.db.as_ref()
    }
}
