//! `AegisOrchestrator` — the single entry point every engine is reached through.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use aegis_detect::{HashAlgo, SignatureDatabase, ThreatDetection};
use aegis_quarantine::{QuarantineRecord, Vault};
use aegis_scan::ScanOptions;
use aegis_yara::RuleManager;
use rusqlite::Connection;

use crate::db;
use crate::jobs::{JobManager, JobState, JobType};
use crate::service::detection_service::DetectionService;
use crate::service::quarantine_service::QuarantineService;
use crate::service::scan_service::ScanService;
use crate::service::status_service::{ComponentStatus, ServiceHealth};
use crate::service::windows_service::WindowsSecurityService;
use crate::{Result, ServiceError};

pub use aegis_common::ScanMode;

/// Where the service keeps its data.
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub data_dir: PathBuf,
}

impl ServiceConfig {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }
}

pub use aegis_realtime::{ProtectionMode, RealtimeStatus};

struct RealtimeState {
    engine: aegis_realtime::RealtimeEngine,
    monitor: aegis_realtime::RealtimeMonitor,
}

/// Central orchestrator. Cheap to clone-share via the inner `Arc`s.
pub struct AegisOrchestrator {
    db_path: PathBuf,
    signatures: Arc<Mutex<SignatureDatabase>>,
    yara: Arc<Mutex<RuleManager>>,
    vault: Arc<Mutex<Vault>>,
    jobs: JobManager,
    realtime: Mutex<Option<RealtimeState>>,
    update: Mutex<Option<aegis_update::UpdateEngine>>,
}

impl AegisOrchestrator {
    /// Initialize storage (DB + migrations + vault) under `config.data_dir`.
    pub fn new(config: &ServiceConfig) -> Result<Self> {
        std::fs::create_dir_all(&config.data_dir)?;
        let db_path = config.data_dir.join("aegis.db");

        let mut conn = aegis_db::open_database(&db_path)?;
        aegis_db::apply_migrations(&mut conn)?;
        db::record_event(&conn, "service_started", &serde_json::json!({}))?;
        db::set_state(
            &conn,
            "service",
            &serde_json::json!({ "version": env!("CARGO_PKG_VERSION"), "data_dir": config.data_dir.display().to_string() }),
        )?;

        let vault_conn = aegis_db::open_database(&db_path)?;
        let vault = Vault::open(config.data_dir.join("quarantine"), vault_conn)?;

        Ok(Self {
            db_path,
            signatures: Arc::new(Mutex::new(SignatureDatabase::new())),
            yara: Arc::new(Mutex::new(RuleManager::new())),
            vault: Arc::new(Mutex::new(vault)),
            jobs: JobManager::new(),
            realtime: Mutex::new(None),
            update: Mutex::new(None),
        })
    }

    fn conn(&self) -> Result<Connection> {
        Ok(aegis_db::open_database(&self.db_path)?)
    }

    // ---- signature / rule management ------------------------------------

    pub fn add_signature_sha256(&self, hex: &str) {
        self.signatures
            .lock()
            .unwrap()
            .insert(HashAlgo::Sha256, hex);
    }

    pub fn load_signature_file(&self, path: impl AsRef<Path>) -> Result<usize> {
        self.signatures
            .lock()
            .unwrap()
            .load_file(path)
            .map_err(|e| ServiceError::Other(e.to_string()))
    }

    pub fn add_yara_rule(&self, origin: &str, source: &str) -> Result<()> {
        let mut mgr = self.yara.lock().unwrap();
        mgr.add_source(origin, source);
        mgr.compile_rules()?;
        Ok(())
    }

    // ---- IPC contract: scanning -----------------------------------------

    /// Queue a file scan; returns the job id immediately. The scan runs on a
    /// background thread and feeds detection + persistence on completion.
    pub fn start_scan(&self, roots: Vec<PathBuf>, mode: ScanMode) -> Result<String> {
        let root_strs: Vec<String> = roots.iter().map(|p| p.display().to_string()).collect();
        let (id, cancel) = self.jobs.create(JobType::FileScan, root_strs);

        if let Ok(conn) = self.conn() {
            let _ = db::upsert_job(&conn, &self.jobs.get(&id).unwrap());
            let _ = db::record_event(&conn, "scan_queued", &serde_json::json!({ "job": id }));
        }

        let jobs = self.jobs.clone();
        let signatures = self.signatures.clone();
        let yara = self.yara.clone();
        let db_path = self.db_path.clone();
        let job_id = id.clone();

        std::thread::spawn(move || {
            jobs.mark_running(&job_id);
            let opts = ScanOptions::for_mode(mode, roots);
            let progress_jobs = jobs.clone();
            let progress_id = job_id.clone();
            let result = ScanService::new().run(&opts, cancel, move |p| {
                progress_jobs.update_progress(&progress_id, p);
            });

            match result {
                Ok(report) => {
                    let detections = {
                        let sig = signatures.lock().unwrap();
                        let yara = yara.lock().unwrap();
                        let yara_ref = if yara.is_compiled() {
                            Some(&*yara)
                        } else {
                            None
                        };
                        DetectionService::new().analyze(&report, &sig, yara_ref)
                    };
                    if let Ok(conn) = aegis_db::open_database(&db_path) {
                        let _ = DetectionService::new().persist(&conn, &detections);
                        jobs.complete(&job_id, report.files_scanned, detections.len() as u32);
                        let _ = db::upsert_job(&conn, &jobs.get(&job_id).unwrap());
                        let _ = db::record_event(
                            &conn,
                            "scan_completed",
                            &serde_json::json!({ "job": job_id, "threats": detections.len() }),
                        );
                    } else {
                        jobs.complete(&job_id, report.files_scanned, detections.len() as u32);
                    }
                }
                Err(aegis_scan::ScanError::Cancelled) => {
                    // Status already 'cancelled' via stop_scan; just stamp history.
                    if let Ok(conn) = aegis_db::open_database(&db_path) {
                        let _ = db::upsert_job(&conn, &jobs.get(&job_id).unwrap());
                    }
                }
                Err(e) => {
                    jobs.fail(&job_id, e.to_string());
                    if let Ok(conn) = aegis_db::open_database(&db_path) {
                        let _ = db::upsert_job(&conn, &jobs.get(&job_id).unwrap());
                    }
                }
            }
        });

        Ok(id)
    }

    /// Request cancellation of a running/queued scan.
    pub fn stop_scan(&self, job_id: &str) -> Result<bool> {
        let cancelled = self.jobs.cancel(job_id);
        if cancelled {
            if let Ok(conn) = self.conn() {
                let _ = db::record_event(
                    &conn,
                    "scan_cancelled",
                    &serde_json::json!({ "job": job_id }),
                );
            }
        }
        Ok(cancelled)
    }

    pub fn get_scan_status(&self, job_id: &str) -> Option<JobState> {
        self.jobs.get(job_id)
    }

    pub fn list_jobs(&self) -> Vec<JobState> {
        self.jobs.list()
    }

    // ---- IPC contract: threats ------------------------------------------

    pub fn get_threats(&self) -> Result<Vec<ThreatDetection>> {
        let conn = self.conn()?;
        db::list_detections(&conn, 500)
    }

    // ---- IPC contract: quarantine ---------------------------------------

    pub fn quarantine_detection(
        &self,
        detection: &ThreatDetection,
        actor: &str,
    ) -> Result<QuarantineRecord> {
        let vault = self.vault.lock().unwrap();
        QuarantineService::quarantine(&vault, detection, actor)
    }

    pub fn restore_file(&self, id: &str, dest: Option<&Path>, actor: &str) -> Result<String> {
        let vault = self.vault.lock().unwrap();
        QuarantineService::restore(&vault, id, dest, actor)
    }

    pub fn delete_quarantine_item(&self, id: &str, actor: &str) -> Result<()> {
        let vault = self.vault.lock().unwrap();
        QuarantineService::delete(&vault, id, actor)
    }

    pub fn list_quarantine(&self) -> Result<Vec<QuarantineRecord>> {
        let vault = self.vault.lock().unwrap();
        QuarantineService::list(&vault)
    }

    // ---- IPC contract: windows scan -------------------------------------

    /// Run a Windows persistence sweep synchronously; persist + return findings.
    pub fn run_windows_scan(&self) -> Result<Vec<ThreatDetection>> {
        let (id, _cancel) = self.jobs.create(JobType::WindowsScan, vec![]);
        self.jobs.mark_running(&id);
        let detections = WindowsSecurityService::new().run();
        let conn = self.conn()?;
        DetectionService::new().persist(&conn, &detections)?;
        self.jobs.complete(&id, 0, detections.len() as u32);
        db::upsert_job(&conn, &self.jobs.get(&id).unwrap())?;
        db::record_event(
            &conn,
            "windows_scan_completed",
            &serde_json::json!({ "threats": detections.len() }),
        )?;
        Ok(detections)
    }

    /// Analyze caller-supplied persistence entries (test seam / future RTP feed).
    pub fn analyze_windows_entries(
        &self,
        entries: &[aegis_windows::PersistenceEntry],
    ) -> Result<Vec<ThreatDetection>> {
        let detections = WindowsSecurityService::new().analyze(entries);
        let conn = self.conn()?;
        DetectionService::new().persist(&conn, &detections)?;
        Ok(detections)
    }

    // ---- IPC contract: real-time protection -----------------------------

    /// Start RTP watching the default user folders.
    pub fn start_realtime(&self, mode: ProtectionMode) -> Result<()> {
        self.start_realtime_with_paths(mode, aegis_realtime::default_watched_paths())
    }

    /// Start RTP watching an explicit set of folders (test seam / custom config).
    pub fn start_realtime_with_paths(
        &self,
        mode: ProtectionMode,
        paths: Vec<String>,
    ) -> Result<()> {
        let mut guard = self.realtime.lock().unwrap();
        if guard
            .as_ref()
            .map(|r| r.monitor.is_running())
            .unwrap_or(false)
        {
            return Ok(()); // already running
        }
        let engine = aegis_realtime::RealtimeEngine::new(
            self.signatures.clone(),
            self.yara.clone(),
            self.vault.clone(),
            self.db_path.clone(),
            mode,
        );
        let mut monitor = aegis_realtime::RealtimeMonitor::new(engine.clone(), paths);
        monitor.start();
        *guard = Some(RealtimeState { engine, monitor });
        if let Ok(conn) = self.conn() {
            let _ = db::record_event(
                &conn,
                "realtime_started",
                &serde_json::json!({ "mode": mode }),
            );
        }
        Ok(())
    }

    /// Stop RTP if running.
    pub fn stop_realtime(&self) -> Result<()> {
        if let Some(mut state) = self.realtime.lock().unwrap().take() {
            state.monitor.stop();
            if let Ok(conn) = self.conn() {
                let _ = db::record_event(&conn, "realtime_stopped", &serde_json::json!({}));
            }
        }
        Ok(())
    }

    pub fn get_realtime_status(&self) -> RealtimeStatus {
        match &*self.realtime.lock().unwrap() {
            Some(state) => RealtimeStatus {
                running: state.monitor.is_running(),
                mode: state.engine.mode(),
                watched_paths: state.monitor.watched_paths().to_vec(),
                events_processed: state.engine.events_processed(),
                alerts_raised: state.engine.alerts_raised(),
            },
            None => RealtimeStatus {
                running: false,
                mode: ProtectionMode::default(),
                watched_paths: vec![],
                events_processed: 0,
                alerts_raised: 0,
            },
        }
    }

    // ---- IPC contract: secure updates -----------------------------------

    /// Configure the update subsystem with a pinned Ed25519 public key (hex) and
    /// a fetcher. Storage lives under `<data_dir>/update`.
    pub fn init_updates(
        &self,
        pubkey_hex: &str,
        fetcher: Box<dyn aegis_update::Fetcher>,
        app_version: &str,
    ) -> Result<()> {
        let verifier = aegis_update::UpdateVerifier::from_hex(pubkey_hex)
            .map_err(aegis_update::UpdateError::from)?;
        let root = self
            .db_path
            .parent()
            .unwrap_or(Path::new("."))
            .join("update");
        let engine = aegis_update::UpdateEngine::new(
            root,
            verifier,
            fetcher,
            self.db_path.clone(),
            app_version,
        )?;
        *self.update.lock().unwrap() = Some(engine);
        Ok(())
    }

    pub fn check_updates(
        &self,
        available: &[aegis_update::UpdateManifest],
    ) -> Result<Vec<aegis_update::UpdateManifest>> {
        let guard = self.update.lock().unwrap();
        let engine = guard.as_ref().ok_or(ServiceError::UpdatesNotConfigured)?;
        Ok(engine.check(available)?)
    }

    pub fn download_updates(&self, manifest: &aegis_update::UpdateManifest) -> Result<()> {
        let guard = self.update.lock().unwrap();
        let engine = guard.as_ref().ok_or(ServiceError::UpdatesNotConfigured)?;
        engine.download(manifest)?;
        Ok(())
    }

    /// Install a downloaded update and hot-reload the affected engine.
    pub fn install_updates(
        &self,
        manifest: &aegis_update::UpdateManifest,
    ) -> Result<aegis_update::InstallOutcome> {
        let outcome = {
            let guard = self.update.lock().unwrap();
            let engine = guard.as_ref().ok_or(ServiceError::UpdatesNotConfigured)?;
            engine.install(manifest)?
        };
        // Reload the running engine from the freshly installed file.
        match manifest.component {
            aegis_update::UpdateComponent::SignatureDatabase => {
                let _ = self
                    .signatures
                    .lock()
                    .unwrap()
                    .load_file(&outcome.installed_path);
            }
            aegis_update::UpdateComponent::YaraRules => {
                if let Ok(src) = std::fs::read_to_string(&outcome.installed_path) {
                    let mut mgr = self.yara.lock().unwrap();
                    mgr.add_source(outcome.installed_path.display().to_string(), src);
                    let _ = mgr.compile_rules();
                }
            }
            _ => {}
        }
        Ok(outcome)
    }

    pub fn rollback_updates(
        &self,
        component: aegis_update::UpdateComponent,
    ) -> Result<aegis_update::InstallOutcome> {
        let guard = self.update.lock().unwrap();
        let engine = guard.as_ref().ok_or(ServiceError::UpdatesNotConfigured)?;
        Ok(engine.rollback(component)?)
    }

    /// Installed components (component, version), read straight from the DB so
    /// the UI's update page works even before the verifier key is configured.
    pub fn get_update_status(&self) -> Result<Vec<(String, String)>> {
        let conn = self.conn()?;
        let mut stmt =
            conn.prepare("SELECT component, version FROM installed_components ORDER BY component")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    // ---- IPC contract: settings -----------------------------------------

    /// Load the persisted UI settings blob (JSON object string; `{}` if unset).
    pub fn get_settings(&self) -> Result<String> {
        let conn = self.conn()?;
        let value: Option<String> = conn
            .query_row(
                "SELECT value_json FROM settings WHERE key = 'app'",
                [],
                |row| row.get(0),
            )
            .ok();
        Ok(value.unwrap_or_else(|| "{}".to_string()))
    }

    /// Persist the UI settings blob (validated as JSON) via the service.
    pub fn save_settings(&self, settings_json: &str) -> Result<()> {
        // Reject non-JSON so the UI can't store garbage.
        let _: serde_json::Value = serde_json::from_str(settings_json)?;
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO settings (key, value_json, updated_at_utc) VALUES ('app', ?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at_utc = excluded.updated_at_utc",
            rusqlite::params![settings_json, chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    // ---- IPC contract: health -------------------------------------------

    pub fn get_service_health(&self) -> ServiceHealth {
        let database = match self.conn().and_then(|c| db::count_detections(&c)) {
            Ok(_) => ComponentStatus::Ok,
            Err(_) => ComponentStatus::Unavailable,
        };
        let rules = {
            let mgr = self.yara.lock().unwrap();
            if mgr.is_compiled() && mgr.source_count() > 0 {
                ComponentStatus::Ok
            } else {
                ComponentStatus::Degraded // hash + heuristics still operate
            }
        };
        let quarantine = match self.vault.lock().unwrap().list_records() {
            Ok(_) => ComponentStatus::Ok,
            Err(_) => ComponentStatus::Degraded,
        };
        let active_jobs = self
            .jobs
            .list()
            .iter()
            .filter(|j| !j.status.is_terminal())
            .count();
        ServiceHealth::compute(
            ComponentStatus::Ok,
            database,
            rules,
            quarantine,
            active_jobs,
        )
    }
}
