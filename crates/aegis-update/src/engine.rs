//! Update engine — ties verification, download, storage, and persistence into
//! the check → download → install → rollback flow.

use std::path::PathBuf;

use thiserror::Error;

use crate::db;
use crate::download::{DownloadError, Fetcher};
use crate::manifest::{UpdateComponent, UpdateManifest};
use crate::storage::{StorageError, UpdateStorage};
use crate::verify::{UpdateVerifier, VerifyError};

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error(transparent)]
    Verify(#[from] VerifyError),
    #[error(transparent)]
    Download(#[from] DownloadError),
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("database open error: {0}")]
    DbOpen(#[from] aegis_db::DbError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("no rollback available for {0}")]
    NoRollback(String),
}

/// Result of a successful install or rollback.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstallOutcome {
    pub component: UpdateComponent,
    pub version: String,
    pub installed_path: PathBuf,
}

pub struct UpdateEngine {
    storage: UpdateStorage,
    verifier: UpdateVerifier,
    fetcher: Box<dyn Fetcher>,
    db_path: PathBuf,
    app_version: String,
}

impl UpdateEngine {
    pub fn new(
        root: impl Into<PathBuf>,
        verifier: UpdateVerifier,
        fetcher: Box<dyn Fetcher>,
        db_path: impl Into<PathBuf>,
        app_version: impl Into<String>,
    ) -> Result<Self, UpdateError> {
        Ok(Self {
            storage: UpdateStorage::open(root)?,
            verifier,
            fetcher,
            db_path: db_path.into(),
            app_version: app_version.into(),
        })
    }

    fn conn(&self) -> Result<rusqlite::Connection, UpdateError> {
        Ok(aegis_db::open_database(&self.db_path)?)
    }

    /// Filter a feed of manifests to those that are signed, app-compatible, and
    /// strictly newer than the installed version. Invalid/old ones are dropped.
    pub fn check(&self, available: &[UpdateManifest]) -> Result<Vec<UpdateManifest>, UpdateError> {
        let conn = self.conn()?;
        let mut applicable = Vec::new();
        for m in available {
            if self.verifier.verify_manifest(m).is_err() {
                continue;
            }
            if self.verifier.check_min_app(m, &self.app_version).is_err() {
                continue;
            }
            let installed = db::installed_version(&conn, m.component)?;
            if self
                .verifier
                .check_rollback(m, installed.as_deref())
                .is_ok()
            {
                applicable.push(m.clone());
            }
        }
        Ok(applicable)
    }

    /// Download a manifest's payload, verifying signature first and the file
    /// hash after. A tampered payload is deleted and rejected.
    pub fn download(&self, manifest: &UpdateManifest) -> Result<PathBuf, UpdateError> {
        self.verifier.verify_manifest(manifest)?; // refuse to fetch unsigned
        let dest = self.storage.download_path(manifest);
        self.fetcher.fetch(&manifest.url, &dest)?;
        let bytes = std::fs::read(&dest)?;
        if let Err(e) = self.verifier.verify_payload(manifest, &bytes) {
            let _ = std::fs::remove_file(&dest);
            return Err(e.into());
        }
        let conn = self.conn()?;
        db::record_history(
            &conn,
            manifest.component,
            &manifest.version,
            &manifest.sha256,
            "downloaded",
        )?;
        Ok(dest)
    }

    /// Install a previously downloaded payload after a full re-verification
    /// (signature + hash + rollback + min-app). Backs up the prior version.
    pub fn install(&self, manifest: &UpdateManifest) -> Result<InstallOutcome, UpdateError> {
        let dest = self.storage.download_path(manifest);
        let bytes = std::fs::read(&dest)?;
        let conn = self.conn()?;
        let installed = db::installed_version(&conn, manifest.component)?;
        self.verifier
            .verify_full(manifest, &bytes, installed.as_deref(), &self.app_version)?;

        let prev = self
            .storage
            .read_manifest()?
            .get(manifest.component.as_str())
            .cloned();
        let live = self.storage.install(manifest, &dest)?;
        if let Some(p) = prev {
            self.storage.write_prev(&p)?;
        }
        self.storage.write_installed(manifest)?;
        db::upsert_installed(&conn, manifest, &live.display().to_string())?;
        db::record_history(
            &conn,
            manifest.component,
            &manifest.version,
            &manifest.sha256,
            "installed",
        )?;
        Ok(InstallOutcome {
            component: manifest.component,
            version: manifest.version.clone(),
            installed_path: live,
        })
    }

    /// Roll a component back to its previous installed version.
    pub fn rollback(&self, component: UpdateComponent) -> Result<InstallOutcome, UpdateError> {
        let prev = self
            .storage
            .read_prev(component)?
            .ok_or_else(|| UpdateError::NoRollback(component.as_str().to_string()))?;
        let live = self.storage.rollback(component)?;
        self.storage.write_installed(&prev)?;
        let conn = self.conn()?;
        db::upsert_installed(&conn, &prev, &live.display().to_string())?;
        db::record_history(&conn, component, &prev.version, &prev.sha256, "rolled_back")?;
        Ok(InstallOutcome {
            component,
            version: prev.version,
            installed_path: live,
        })
    }

    /// (component, version) for every installed component.
    pub fn status(&self) -> Result<Vec<(String, String)>, UpdateError> {
        Ok(db::list_installed(&self.conn()?)?)
    }

    pub fn installed_path(&self, component: UpdateComponent) -> PathBuf {
        self.storage.installed_path(component)
    }
}
