use sea_orm::DatabaseConnection;

use crate::domain::health::HealthIndicator;

pub async fn check(db: &DatabaseConnection) -> HealthIndicator {
    match db.ping().await {
        Ok(()) => HealthIndicator::up("database"),
        Err(error) => HealthIndicator::down("database", error.to_string()),
    }
}
