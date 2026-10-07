//! Database connection, migrations and demo data.

use crate::server::migrations::Migrator;
use crate::server::{club, person};
use sea_orm::{
    ActiveModelTrait, ConnectOptions, Database, DatabaseConnection, DbErr, EntityTrait,
    PaginatorTrait, Set,
};
use sea_orm_migration::MigratorTrait;

/// Used when the `DATABASE_URL` environment variable is not set. Relative to the working directory.
const DEFAULT_DATABASE_URL: &str = "sqlite://data.sqlite?mode=rwc";

/// Connects to the database, applies all pending migrations and seeds demo data into an empty database.
pub async fn connect_and_prepare() -> Result<DatabaseConnection, DbErr> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_owned());
    tracing::info!(%url, "Connecting to database.");

    let mut options = ConnectOptions::new(url);
    options.sqlx_logging(false);
    let db = Database::connect(options).await?;

    tracing::info!("Applying pending database migrations.");
    Migrator::up(&db, None).await?;

    seed(&db).await?;
    Ok(db)
}

async fn seed(db: &DatabaseConnection) -> Result<(), DbErr> {
    if club::Entity::find().count(db).await? > 0 {
        return Ok(());
    }
    tracing::info!("Seeding demo data.");

    let now = crate::models::now_primitive();
    // Dummy data: Quidditch teams and characters from the Harry Potter books, with made-up rosters. The clubs were
    // registered over the past months, newest last.
    let clubs = [
        (
            "Holyhead Harpies",
            160,
            vec![
                ("Ginny", "Weasley", "female", 1981),
                ("Gwenog", "Jones", "female", 1968),
                ("Katie", "Bell", "female", 1979),
                ("Angelina", "Johnson", "female", 1977),
                ("Alicia", "Spinnet", "female", 1978),
            ],
        ),
        (
            "Puddlemere United",
            131,
            vec![
                ("Oliver", "Wood", "male", 1975),
                ("Cormac", "McLaggen", "male", 1979),
                ("Cho", "Chang", "female", 1979),
                ("Roger", "Davies", "male", 1978),
            ],
        ),
        (
            "Chudley Cannons",
            97,
            vec![
                ("Ron", "Weasley", "male", 1980),
                ("Seamus", "Finnigan", "male", 1980),
                ("Dean", "Thomas", "male", 1980),
                ("Lee", "Jordan", "male", 1977),
            ],
        ),
        (
            "Wimbourne Wasps",
            64,
            vec![
                ("Fred", "Weasley", "male", 1978),
                ("George", "Weasley", "male", 1978),
                ("Ernie", "Macmillan", "male", 1980),
                ("Susan", "Bones", "female", 1979),
                ("Padma", "Patil", "female", 1979),
            ],
        ),
        (
            "Tutshill Tornados",
            38,
            vec![
                ("Harry", "Potter", "male", 1980),
                ("Draco", "Malfoy", "male", 1980),
                ("Marcus", "Flint", "male", 1976),
                ("Blaise", "Zabini", "male", 1979),
            ],
        ),
        (
            "Ballycastle Bats",
            12,
            vec![
                ("Luna", "Lovegood", "female", 1981),
                ("Neville", "Longbottom", "male", 1980),
                ("Hermione", "Granger", "female", 1979),
                ("Hannah", "Abbott", "female", 1980),
                ("Zacharias", "Smith", "male", 1980),
            ],
        ),
    ];

    for (club_name, registered_days_ago, people) in clubs {
        let registered_at = now - time::Duration::days(registered_days_ago);
        let club = club::ActiveModel {
            name: Set(club_name.to_owned()),
            created_at: Set(registered_at),
            ..Default::default()
        }
        .insert(db)
        .await?;

        for (index, (first_name, last_name, gender, year_of_birth)) in (0_i64..).zip(people) {
            person::ActiveModel {
                first_name: Set(first_name.to_owned()),
                last_name: Set(last_name.to_owned()),
                gender: Set(gender.to_owned()),
                year_of_birth: Set(year_of_birth),
                club_id: Set(club.id),
                // Members joined in the days after their club was registered.
                created_at: Set(registered_at + time::Duration::days(index * 2 + 1)),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }
    }
    Ok(())
}
