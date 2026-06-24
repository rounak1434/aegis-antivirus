//! Service-health reporting.

use serde::{Deserialize, Serialize};

/// Health of a single subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentStatus {
    Ok,
    Degraded,
    Unavailable,
}

/// Aggregate service health across all engines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub scanner: ComponentStatus,
    pub database: ComponentStatus,
    pub rules: ComponentStatus,
    pub quarantine: ComponentStatus,
    pub active_jobs: usize,
    pub overall: ComponentStatus,
}

impl ServiceHealth {
    /// Overall = worst of the components.
    pub fn compute(
        scanner: ComponentStatus,
        database: ComponentStatus,
        rules: ComponentStatus,
        quarantine: ComponentStatus,
        active_jobs: usize,
    ) -> Self {
        let overall = [scanner, database, rules, quarantine]
            .into_iter()
            .max_by_key(severity)
            .unwrap_or(ComponentStatus::Ok);
        Self { scanner, database, rules, quarantine, active_jobs, overall }
    }
}

fn severity(s: &ComponentStatus) -> u8 {
    match s {
        ComponentStatus::Ok => 0,
        ComponentStatus::Degraded => 1,
        ComponentStatus::Unavailable => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overall_is_worst_component() {
        let h = ServiceHealth::compute(
            ComponentStatus::Ok,
            ComponentStatus::Ok,
            ComponentStatus::Degraded,
            ComponentStatus::Ok,
            0,
        );
        assert_eq!(h.overall, ComponentStatus::Degraded);

        let h2 = ServiceHealth::compute(
            ComponentStatus::Ok,
            ComponentStatus::Unavailable,
            ComponentStatus::Degraded,
            ComponentStatus::Ok,
            1,
        );
        assert_eq!(h2.overall, ComponentStatus::Unavailable);
    }
}
