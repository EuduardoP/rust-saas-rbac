use sea_orm::{Database, DatabaseConnection, DbErr};

use std::env;

pub async fn connect_db() -> Result<DatabaseConnection, DbErr> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL não definido no .env");

    Database::connect(&database_url).await
}
