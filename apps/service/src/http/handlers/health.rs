use std::collections::BTreeMap;

use actix_web::{HttpResponse, get, web};
use sea_orm::DatabaseConnection;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    application::health::check_health,
    domain::health::{HealthIndicator, HealthIndicatorStatus, HealthReport, HealthStatus},
    infra::database::health as database_health,
};

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    status: String,
    info: BTreeMap<String, HealthIndicatorResponse>,
    error: BTreeMap<String, HealthIndicatorResponse>,
    details: BTreeMap<String, HealthIndicatorResponse>,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct HealthIndicatorResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

impl From<&HealthIndicator> for HealthIndicatorResponse {
    fn from(indicator: &HealthIndicator) -> Self {
        let message = match indicator.status() {
            HealthIndicatorStatus::Up => None,
            HealthIndicatorStatus::Down { message } => Some(message.clone()),
        };

        Self {
            status: indicator.status().as_str().to_owned(),
            message,
        }
    }
}

impl From<HealthReport> for HealthResponse {
    fn from(report: HealthReport) -> Self {
        let mut info = BTreeMap::new();
        let mut error = BTreeMap::new();
        let mut details = BTreeMap::new();

        for indicator in report.indicators() {
            let name = indicator.name().to_owned();
            let response = HealthIndicatorResponse::from(indicator);

            if indicator.is_up() {
                info.insert(name.clone(), response.clone());
            } else {
                error.insert(name.clone(), response.clone());
            }

            details.insert(name, response);
        }

        Self {
            status: report.status().as_str().to_owned(),
            info,
            error,
            details,
        }
    }
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service dependency is unhealthy", body = HealthResponse)
    )
)]
#[get("/health")]
pub async fn health(db: web::Data<DatabaseConnection>) -> HttpResponse {
    let database = database_health::check(db.get_ref()).await;
    let report = check_health(vec![database]);
    let status = report.status();
    let response = HealthResponse::from(report);

    match status {
        HealthStatus::Ok => HttpResponse::Ok().json(response),
        HealthStatus::Error => HttpResponse::ServiceUnavailable().json(response),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_healthy_report_to_terminus_shape() {
        let response =
            HealthResponse::from(HealthReport::from_indicators(vec![HealthIndicator::up(
                "database",
            )]));

        assert_eq!(response.status, "ok");
        assert_eq!(response.info["database"].status, "up");
        assert!(response.error.is_empty());
        assert_eq!(response.details["database"].status, "up");
    }

    #[test]
    fn maps_unhealthy_report_to_terminus_shape() {
        let response =
            HealthResponse::from(HealthReport::from_indicators(vec![HealthIndicator::down(
                "database",
                "connection failed",
            )]));

        assert_eq!(response.status, "error");
        assert!(response.info.is_empty());
        assert_eq!(response.error["database"].status, "down");
        assert_eq!(
            response.details["database"].message.as_deref(),
            Some("connection failed")
        );
    }
}
