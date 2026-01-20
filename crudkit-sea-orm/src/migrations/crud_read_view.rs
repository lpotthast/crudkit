//! Helpers for creating SQL read views that include validation error status.
//!
//! These functions generate PostgreSQL views that join entity tables with the
//! unified `CrudkitValidation` table to provide a `has_validation_errors` column.

use sea_orm_migration::{
    sea_orm::{ConnectionTrait, DbBackend, Statement}, DbErr,
    SchemaManager,
};

/// Describes a primary key field for SQL generation.
pub struct IdFieldDef {
    /// The column/field name (e.g., "id", "user_id").
    pub name: &'static str,
    /// The `IdValue` type variant (e.g., "I64", "I32", "String", "Uuid").
    pub type_variant: &'static str,
}

impl IdFieldDef {
    #[must_use]
    pub const fn new(name: &'static str, type_variant: &'static str) -> Self {
        Self { name, type_variant }
    }
}

#[macro_export]
macro_rules! impl_read_view_migration {
    ($table_name: expr, $resource_name: expr, $id_fields: expr, $migration_name: ident) => {
        use sea_orm_migration::prelude::*;

        pub struct $migration_name;

        impl sea_orm_migration::MigrationName for $migration_name {
            fn name(&self) -> &str {
                // Hack to not run into the "cannot return reference to owned content" error.
                static NAME: std::sync::OnceLock<String> = std::sync::OnceLock::new();
                NAME.get_or_init(|| {
                    std::path::Path::new(file!())
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| format!("{s}_read_view"))
                        .unwrap()
                })
            }
        }

        #[async_trait::async_trait]
        impl MigrationTrait for $migration_name {
            async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                crudkit_sea_orm::migrations::crud_read_view::drop_read_view(manager, $table_name)
                    .await?;

                crudkit_sea_orm::migrations::crud_read_view::create_read_view(
                    manager,
                    $table_name,
                    $resource_name,
                    $id_fields,
                )
                .await
            }
            async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                crudkit_sea_orm::migrations::crud_read_view::drop_read_view(manager, $table_name)
                    .await
            }
        }
    };
}

/// # Usage
///
/// ```
/// use sea_orm_migration::prelude::*;
/// use crate::migrations::crud_read_view::{create_read_view, drop_read_view, IdFieldDef};
///
/// #[derive(DeriveMigrationName)]
/// pub struct Migration;
///
/// const TABLE_NAME: &str = "User";
/// const RESOURCE_NAME: &str = "user";
/// const ID_FIELDS: &[IdFieldDef] = &[IdFieldDef::new("id", "I64")];
///
/// #[async_trait::async_trait]
/// impl MigrationTrait for Migration {
///     async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
///         drop_read_view(manager, TABLE_NAME).await?;
///         create_read_view(manager, TABLE_NAME, RESOURCE_NAME, ID_FIELDS).await
///     }
///
///     async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
///         drop_read_view(manager, TABLE_NAME).await
///     }
/// }
/// ```
///
/// # Errors
///
/// When the database statement fails to execute.
pub async fn create_read_view(
    manager: &SchemaManager<'_>,
    table_name: &'static str,
    resource_name: &'static str,
    id_fields: &[IdFieldDef],
) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute(build_create_read_view_stmt(
            table_name,
            resource_name,
            id_fields,
        ))
        .await
        .map(|_exec_result| ())
}

#[must_use]
pub fn build_create_read_view_stmt(
    table_name: &'static str,
    resource_name: &'static str,
    id_fields: &[IdFieldDef],
) -> Statement {
    let id_json_sql = build_id_json_sql(id_fields);

    Statement::from_sql_and_values(
        DbBackend::Postgres,
        format!(
            r#"
            CREATE VIEW "{table_name}ReadView" AS
            SELECT N.*,
                   EXISTS (
                       SELECT 1
                       FROM "CrudkitValidation" V
                       WHERE V.resource_name = '{resource_name}'
                         AND V.entity_id = {id_json_sql}
                   ) AS has_validation_errors
            FROM "{table_name}" AS N;
            "#
        ),
        vec![],
    )
}

/// Build the JSON construction SQL for comparing entity IDs.
/// Constructs a JSON array matching the `SerializableId` format: `[["field_name", {"TypeVariant": value}], ...]`
fn build_id_json_sql(id_fields: &[IdFieldDef]) -> String {
    let field_expressions: Vec<String> = id_fields
        .iter()
        .map(|f| {
            format!(
                "jsonb_build_array('{}', jsonb_build_object('{}', N.{}))",
                f.name, f.type_variant, f.name
            )
        })
        .collect();

    format!("jsonb_build_array({})", field_expressions.join(", "))
}

/// # Errors
///
/// When the database statement fails to execute.
pub async fn drop_read_view(
    manager: &SchemaManager<'_>,
    table_name: &'static str,
) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute(build_drop_read_view_stmt(table_name))
        .await
        .map(|_exec_result| ())
}

#[must_use]
pub fn build_drop_read_view_stmt(table_name: &'static str) -> Statement {
    Statement::from_sql_and_values(
        DbBackend::Postgres,
        format!(
            r#"
            DROP VIEW IF EXISTS "{table_name}ReadView";
            "#
        ),
        vec![],
    )
}
