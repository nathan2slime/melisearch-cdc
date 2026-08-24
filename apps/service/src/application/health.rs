use crate::domain::health::{HealthIndicator, HealthReport};

pub fn check_health(indicators: Vec<HealthIndicator>) -> HealthReport {
    HealthReport::from_indicators(indicators)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::health::HealthStatus;

    #[test]
    fn check_health_reports_ok_when_all_indicators_are_up() {
        let report = check_health(vec![HealthIndicator::up("database")]);

        assert_eq!(report.status(), HealthStatus::Ok);
    }

    #[test]
    fn check_health_reports_error_when_any_indicator_is_down() {
        let report = check_health(vec![
            HealthIndicator::up("api"),
            HealthIndicator::down("database", "connection failed"),
        ]);

        assert_eq!(report.status(), HealthStatus::Error);
    }
}
