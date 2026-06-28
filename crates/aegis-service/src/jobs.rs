//! Background-job manager: queued/running scans with cancellation + status.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use aegis_scan::ScanProgress;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    FileScan,
    WindowsScan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Cancelled,
    Failed,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Queued => "queued",
            JobStatus::Running => "running",
            JobStatus::Completed => "completed",
            JobStatus::Cancelled => "cancelled",
            JobStatus::Failed => "failed",
        }
    }
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            JobStatus::Completed | JobStatus::Cancelled | JobStatus::Failed
        )
    }
}

/// A serializable snapshot of a job's state (returned to the UI).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobState {
    pub id: String,
    pub job_type: JobType,
    pub status: JobStatus,
    pub roots: Vec<String>,
    pub progress: Option<ScanProgress>,
    pub files_scanned: u64,
    pub threats_found: u32,
    pub error: Option<String>,
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

struct JobEntry {
    state: JobState,
    cancel: Arc<AtomicBool>,
}

/// Thread-safe registry of jobs. Cloneable (shares the inner map).
#[derive(Clone, Default)]
pub struct JobManager {
    jobs: Arc<Mutex<HashMap<String, JobEntry>>>,
}

impl JobManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new queued job; returns (id, cancel flag).
    pub fn create(&self, job_type: JobType, roots: Vec<String>) -> (String, Arc<AtomicBool>) {
        let id = uuid::Uuid::new_v4().to_string();
        let cancel = Arc::new(AtomicBool::new(false));
        let state = JobState {
            id: id.clone(),
            job_type,
            status: JobStatus::Queued,
            roots,
            progress: None,
            files_scanned: 0,
            threats_found: 0,
            error: None,
            queued_at: Utc::now(),
            started_at: None,
            finished_at: None,
        };
        self.jobs.lock().unwrap().insert(
            id.clone(),
            JobEntry {
                state,
                cancel: cancel.clone(),
            },
        );
        (id, cancel)
    }

    pub fn mark_running(&self, id: &str) {
        self.mutate(id, |s| {
            // A cancel that arrived while still queued must not be overwritten.
            if !s.status.is_terminal() {
                s.status = JobStatus::Running;
                s.started_at = Some(Utc::now());
            }
        });
    }

    pub fn update_progress(&self, id: &str, progress: ScanProgress) {
        self.mutate(id, |s| {
            s.files_scanned = progress.files_scanned;
            s.progress = Some(progress);
        });
    }

    pub fn complete(&self, id: &str, files_scanned: u64, threats_found: u32) {
        self.mutate(id, |s| {
            // A cancel requested mid-run wins over a natural completion.
            if s.status != JobStatus::Cancelled {
                s.status = JobStatus::Completed;
            }
            s.files_scanned = files_scanned;
            s.threats_found = threats_found;
            s.finished_at = Some(Utc::now());
        });
    }

    pub fn fail(&self, id: &str, error: impl Into<String>) {
        self.mutate(id, |s| {
            s.status = JobStatus::Failed;
            s.error = Some(error.into());
            s.finished_at = Some(Utc::now());
        });
    }

    /// Request cancellation. Returns false if the job is unknown.
    pub fn cancel(&self, id: &str) -> bool {
        let mut jobs = self.jobs.lock().unwrap();
        match jobs.get_mut(id) {
            Some(entry) => {
                entry.cancel.store(true, Ordering::Relaxed);
                if !entry.state.status.is_terminal() {
                    entry.state.status = JobStatus::Cancelled;
                    entry.state.finished_at = Some(Utc::now());
                }
                true
            }
            None => false,
        }
    }

    pub fn get(&self, id: &str) -> Option<JobState> {
        self.jobs.lock().unwrap().get(id).map(|e| e.state.clone())
    }

    pub fn list(&self) -> Vec<JobState> {
        self.jobs
            .lock()
            .unwrap()
            .values()
            .map(|e| e.state.clone())
            .collect()
    }

    fn mutate(&self, id: &str, f: impl FnOnce(&mut JobState)) {
        if let Some(entry) = self.jobs.lock().unwrap().get_mut(id) {
            f(&mut entry.state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_queued_running_completed() {
        let jm = JobManager::new();
        let (id, _cancel) = jm.create(JobType::FileScan, vec!["C:/".into()]);
        assert_eq!(jm.get(&id).unwrap().status, JobStatus::Queued);
        jm.mark_running(&id);
        assert_eq!(jm.get(&id).unwrap().status, JobStatus::Running);
        jm.complete(&id, 100, 2);
        let s = jm.get(&id).unwrap();
        assert_eq!(s.status, JobStatus::Completed);
        assert_eq!(s.files_scanned, 100);
        assert_eq!(s.threats_found, 2);
    }

    #[test]
    fn cancel_sets_flag_and_status() {
        let jm = JobManager::new();
        let (id, cancel) = jm.create(JobType::FileScan, vec![]);
        assert!(jm.cancel(&id));
        assert!(cancel.load(Ordering::Relaxed));
        assert_eq!(jm.get(&id).unwrap().status, JobStatus::Cancelled);
        // completion after cancel does not override cancelled status
        jm.complete(&id, 5, 0);
        assert_eq!(jm.get(&id).unwrap().status, JobStatus::Cancelled);
    }

    #[test]
    fn cancel_unknown_job_is_false() {
        let jm = JobManager::new();
        assert!(!jm.cancel("nope"));
    }

    #[test]
    fn fail_records_error() {
        let jm = JobManager::new();
        let (id, _) = jm.create(JobType::WindowsScan, vec![]);
        jm.fail(&id, "boom");
        let s = jm.get(&id).unwrap();
        assert_eq!(s.status, JobStatus::Failed);
        assert_eq!(s.error.as_deref(), Some("boom"));
    }
}
