#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HealthStatus {
    Ok,
    Error,
}

impl HealthStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthIndicator {
    name: String,
    status: HealthIndicatorStatus,
}

impl HealthIndicator {
    pub fn up(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthIndicatorStatus::Up,
        }
    }

    pub fn down(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthIndicatorStatus::Down {
                message: message.into(),
            },
        }
    }

    pub fn is_up(&self) -> bool {
        matches!(self.status, HealthIndicatorStatus::Up)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn status(&self) -> &HealthIndicatorStatus {
        &self.status
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HealthIndicatorStatus {
    Up,
    Down { message: String },
}

impl HealthIndicatorStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down { .. } => "down",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthReport {
    status: HealthStatus,
    indicators: Vec<HealthIndicator>,
}

impl HealthReport {
    pub fn from_indicators(indicators: Vec<HealthIndicator>) -> Self {
        let status = if indicators.iter().all(HealthIndicator::is_up) {
            HealthStatus::Ok
        } else {
            HealthStatus::Error
        };

        Self { status, indicators }
    }

    pub fn status(&self) -> HealthStatus {
        self.status
    }

    pub fn indicators(&self) -> &[HealthIndicator] {
        &self.indicators
    }
}
