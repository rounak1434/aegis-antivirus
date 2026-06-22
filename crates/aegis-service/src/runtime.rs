use aegis_common::{ProtectionStatus, ServiceHealth};
use tokio::sync::watch;

pub struct AegisServiceRuntime {
    status_tx: watch::Sender<ProtectionStatus>,
}

impl AegisServiceRuntime {
    pub fn new() -> Self {
        let (status_tx, _) = watch::channel(ProtectionStatus {
            health: ServiceHealth::Starting,
            real_time_protection: false,
            file_monitor: false,
            process_monitor: false,
            scheduled_scans: false,
            signature_version: None,
            last_service_heartbeat_utc: None,
        });
        Self { status_tx }
    }

    pub fn subscribe_status(&self) -> watch::Receiver<ProtectionStatus> {
        self.status_tx.subscribe()
    }

    pub async fn mark_running(&self) {
        let mut status = self.status_tx.borrow().clone();
        status.health = ServiceHealth::Running;
        let _ = self.status_tx.send(status);
    }

    pub async fn mark_stopped(&self) {
        let mut status = self.status_tx.borrow().clone();
        status.health = ServiceHealth::Stopped;
        let _ = self.status_tx.send(status);
    }
}

impl Default for AegisServiceRuntime {
    fn default() -> Self {
        Self::new()
    }
}
