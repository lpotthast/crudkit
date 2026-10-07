//! SQLite schema of this example.

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // CrudKit's own migration and ours share one migration table, so they must be applied by one migrator.
            Box::new(
                crudkit_sea_orm::migrations::m20260118_crudkit_000001_create_unified_validation_table::Migration,
            ),
            Box::new(CreateClubTable),
            Box::new(ReadViewMigration {
                name: "m20260101_000002_create_club_read_view",
                table_name: "Club",
                resource_name: "clubs",
            }),
            Box::new(CreatePersonTable),
            Box::new(ReadViewMigration {
                name: "m20260101_000004_create_person_read_view",
                table_name: "Person",
                resource_name: "people",
            }),
        ]
    }
}

#[derive(Iden)]
enum Club {
    #[iden = "Club"]
    Table,
    Id,
    Name,
    CreatedAt,
}

#[derive(Iden)]
enum Person {
    #[iden = "Person"]
    Table,
    Id,
    FirstName,
    LastName,
    Gender,
    YearOfBirth,
    ClubId,
    CreatedAt,
}

struct CreateClubTable;

impl MigrationName for CreateClubTable {
    fn name(&self) -> &str {
        "m20260101_000001_create_club_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for CreateClubTable {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Club::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Club::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Club::Name).string().not_null())
                    .col(ColumnDef::new(Club::CreatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Club::Table).to_owned())
            .await
    }
}

struct CreatePersonTable;

impl MigrationName for CreatePersonTable {
    fn name(&self) -> &str {
        "m20260101_000003_create_person_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for CreatePersonTable {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Person::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Person::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Person::FirstName).string().not_null())
                    .col(ColumnDef::new(Person::LastName).string().not_null())
                    .col(ColumnDef::new(Person::Gender).string().not_null())
                    .col(ColumnDef::new(Person::YearOfBirth).integer().not_null())
                    .col(ColumnDef::new(Person::ClubId).big_integer().not_null())
                    .col(ColumnDef::new(Person::CreatedAt).date_time().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Person::Table, Person::ClubId)
                            .to(Club::Table, Club::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Person::Table).to_owned())
            .await
    }
}

/// Creates the `{table_name}ReadView` view backing a resource's read model.
///
/// The view adds the `has_validation_errors` column expected by `crudkit_sea_orm::ReadView`. CrudKit's own
/// `crud_read_view` helpers emit PostgreSQL-only SQL (`jsonb_build_array`), so this example builds the equivalent
/// SQLite statement. `json_array`/`json_object` produce the same compact JSON that `UnifiedValidationRepository`
/// stores for a single `i64` id named `id`, e.g. `[["id",{"I64":1}]]`.
struct ReadViewMigration {
    name: &'static str,
    table_name: &'static str,
    resource_name: &'static str,
}

impl ReadViewMigration {
    fn drop_stmt(&self) -> String {
        format!(r#"DROP VIEW IF EXISTS "{}ReadView";"#, self.table_name)
    }

    fn create_stmt(&self) -> String {
        let Self {
            table_name,
            resource_name,
            ..
        } = self;
        format!(
            r#"
            CREATE VIEW "{table_name}ReadView" AS
            SELECT N.*,
                   EXISTS (
                       SELECT 1
                       FROM "CrudkitValidation" V
                       WHERE V.resource_name = '{resource_name}'
                         AND V.entity_id = json_array(json_array('id', json_object('I64', N.id)))
                   ) AS has_validation_errors
            FROM "{table_name}" AS N;
            "#
        )
    }
}

impl MigrationName for ReadViewMigration {
    fn name(&self) -> &str {
        self.name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for ReadViewMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(&self.drop_stmt()).await?;
        db.execute_unprepared(&self.create_stmt()).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(&self.drop_stmt())
            .await?;
        Ok(())
    }
}
