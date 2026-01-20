//! Pre-built sea-orm migrations and migration helper functions for crudkit.

use sea_orm_migration::{MigrationTrait, MigratorTrait};

pub mod crud_read_view;
pub mod m20260118_crudkit_000001_create_unified_validation_table;

/// Provides all migrations required for crudkit to operate. This currently includes:
///
/// - [`m20260118_crudkit_000001_create_unified_validation_table`] - For storing all validations in one unified database table.
///
/// # Usage
///
/// ```ignore
/// info!("Performing pending crudkit database migrations...");
/// crudkit_sea_orm::migrations::Migrator::up(&db, None)
///     .await
///     .unwrap();
///
/// info!("Performing pending own database migrations...");
/// your_own::Migrator::up(&db, None)
///     .await
///     .unwrap();
/// ```
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            m20260118_crudkit_000001_create_unified_validation_table::Migration,
        )]
    }
}
