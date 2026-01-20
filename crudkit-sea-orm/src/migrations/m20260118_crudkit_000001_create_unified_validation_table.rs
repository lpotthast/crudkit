use sea_orm_migration::prelude::*;

#[derive(Iden)]
enum CrudkitValidation {
    #[iden = "CrudkitValidation"]
    Table,
    Id,
    ResourceName,
    EntityId,
    ValidatorName,
    ValidatorVersion,
    ViolationSeverity,
    ViolationMessage,
    CreatedAt,
}

/// Migration for creating the unified `CrudkitValidation` table, allowing crudkit to store
/// validation violations of any resource.
///
/// Already part of our main [`crate::migrations::Migrator`].
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CrudkitValidation::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CrudkitValidation::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(CrudkitValidation::ResourceName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CrudkitValidation::EntityId)
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CrudkitValidation::ValidatorName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CrudkitValidation::ValidatorVersion)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CrudkitValidation::ViolationSeverity)
                            .string_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CrudkitValidation::ViolationMessage)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CrudkitValidation::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on `resource_type` for faster unified mode queries.
        // Note: We don't index `entity_id` because JSON columns can't use btree indexes.
        // Queries filter by resource_type first, then scan for matching entity_id.
        manager
            .create_index(
                Index::create()
                    .name("idx_crudkit_validation_resource_type")
                    .table(CrudkitValidation::Table)
                    .col(CrudkitValidation::ResourceName)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CrudkitValidation::Table).to_owned())
            .await
    }
}
