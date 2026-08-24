use sea_orm::{Database, DatabaseConnection};

pub async fn connect(database_url: &str) -> std::io::Result<DatabaseConnection> {
    Database::connect(database_url)
        .await
        .map_err(|error| std::io::Error::other(format!("database connection failed: {error}")))
}
