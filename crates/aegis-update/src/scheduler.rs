//! Update check scheduling policy (pure; the service runs the actual timer).

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateSchedule {
    Manual,
    Daily,
    Weekly,
    Startup,
}

impl UpdateSchedule {
    /// Whether an update check is due now.
    ///
    /// - `Manual` → never automatic.
    /// - `Startup` → due once at startup.
    /// - `Daily` / `Weekly` → due if never checked or the interval elapsed.
    pub fn is_due(
        &self,
        last_check: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
        is_startup: bool,
    ) -> bool {
        match self {
            UpdateSchedule::Manual => false,
            UpdateSchedule::Startup => is_startup,
            UpdateSchedule::Daily => due_after(last_check, now, Duration::hours(24)),
            UpdateSchedule::Weekly => due_after(last_check, now, Duration::days(7)),
        }
    }
}

fn due_after(last: Option<DateTime<Utc>>, now: DateTime<Utc>, interval: Duration) -> bool {
    match last {
        None => true,
        Some(t) => now - t >= interval,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    #[test]
    fn manual_never_due() {
        assert!(!UpdateSchedule::Manual.is_due(None, t("2026-01-01T00:00:00Z"), true));
    }

    #[test]
    fn startup_due_only_at_startup() {
        let now = t("2026-01-01T00:00:00Z");
        assert!(UpdateSchedule::Startup.is_due(None, now, true));
        assert!(!UpdateSchedule::Startup.is_due(None, now, false));
    }

    #[test]
    fn daily_interval() {
        let now = t("2026-01-02T01:00:00Z");
        assert!(UpdateSchedule::Daily.is_due(Some(t("2026-01-01T00:00:00Z")), now, false));
        assert!(!UpdateSchedule::Daily.is_due(Some(t("2026-01-01T02:00:00Z")), now, false));
        assert!(UpdateSchedule::Daily.is_due(None, now, false));
    }

    #[test]
    fn weekly_interval() {
        let now = t("2026-01-09T00:00:00Z");
        assert!(UpdateSchedule::Weekly.is_due(Some(t("2026-01-01T00:00:00Z")), now, false));
        assert!(!UpdateSchedule::Weekly.is_due(Some(t("2026-01-05T00:00:00Z")), now, false));
    }
}
