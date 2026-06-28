//! Policy: maps a detection's threat level + the active mode to an action.

use crate::model::{ProtectionMode, RealtimeAction, ThreatLevel};

/// Decide the action for a detection under `mode`.
///
/// - `MonitorOnly` → log only (`Monitored`).
/// - `NotifyOnly` → alert, no action (`Notified`).
/// - `AutoQuarantine` → quarantine High/Critical, otherwise notify.
pub fn decide(mode: ProtectionMode, level: ThreatLevel) -> RealtimeAction {
    match mode {
        ProtectionMode::MonitorOnly => RealtimeAction::Monitored,
        ProtectionMode::NotifyOnly => RealtimeAction::Notified,
        ProtectionMode::AutoQuarantine => match level {
            ThreatLevel::High | ThreatLevel::Critical => RealtimeAction::Quarantined,
            _ => RealtimeAction::Notified,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor_only_never_acts() {
        assert_eq!(
            decide(ProtectionMode::MonitorOnly, ThreatLevel::Critical),
            RealtimeAction::Monitored
        );
    }

    #[test]
    fn notify_only_always_notifies() {
        assert_eq!(
            decide(ProtectionMode::NotifyOnly, ThreatLevel::Critical),
            RealtimeAction::Notified
        );
        assert_eq!(
            decide(ProtectionMode::NotifyOnly, ThreatLevel::Low),
            RealtimeAction::Notified
        );
    }

    #[test]
    fn auto_quarantine_thresholds() {
        assert_eq!(
            decide(ProtectionMode::AutoQuarantine, ThreatLevel::Critical),
            RealtimeAction::Quarantined
        );
        assert_eq!(
            decide(ProtectionMode::AutoQuarantine, ThreatLevel::High),
            RealtimeAction::Quarantined
        );
        assert_eq!(
            decide(ProtectionMode::AutoQuarantine, ThreatLevel::Medium),
            RealtimeAction::Notified
        );
        assert_eq!(
            decide(ProtectionMode::AutoQuarantine, ThreatLevel::Low),
            RealtimeAction::Notified
        );
    }

    #[test]
    fn default_mode_is_notify_only() {
        assert_eq!(ProtectionMode::default(), ProtectionMode::NotifyOnly);
    }
}
