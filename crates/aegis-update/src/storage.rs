//! On-disk layout: `updates/` (downloads), `installed/` (live), `backup/`
//! (previous versions for rollback), and `manifest.json` (installed registry).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::manifest::{UpdateComponent, UpdateManifest};

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("no backup to roll back for {0}")]
    NoBackup(String),
}

pub struct UpdateStorage {
    root: PathBuf,
}

impl UpdateStorage {
    /// Create the layout under `root`.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let root = root.into();
        for sub in ["updates", "installed", "backup"] {
            std::fs::create_dir_all(root.join(sub))?;
        }
        Ok(Self { root })
    }

    pub fn download_path(&self, manifest: &UpdateManifest) -> PathBuf {
        self.root
            .join("updates")
            .join(format!("{}-{}.bin", manifest.component.as_str(), manifest.version))
    }

    pub fn installed_path(&self, component: UpdateComponent) -> PathBuf {
        self.root.join("installed").join(format!("{}.bin", component.as_str()))
    }

    /// Install a verified payload: back up the current installed file (if any),
    /// then atomically move the new payload into place. Returns the live path.
    pub fn install(&self, manifest: &UpdateManifest, payload: &Path) -> Result<PathBuf, StorageError> {
        let live = self.installed_path(manifest.component);
        if live.exists() {
            let backup = self
                .root
                .join("backup")
                .join(format!("{}.bin", manifest.component.as_str()));
            std::fs::copy(&live, &backup)?;
        }
        // Copy then rename for atomicity within the installed dir.
        let tmp = live.with_extension("tmp");
        std::fs::copy(payload, &tmp)?;
        std::fs::rename(&tmp, &live)?;
        Ok(live)
    }

    /// Restore the most recent backup for a component over the live file.
    pub fn rollback(&self, component: UpdateComponent) -> Result<PathBuf, StorageError> {
        let backup = self.root.join("backup").join(format!("{}.bin", component.as_str()));
        if !backup.exists() {
            return Err(StorageError::NoBackup(component.as_str().to_string()));
        }
        let live = self.installed_path(component);
        std::fs::copy(&backup, &live)?;
        Ok(live)
    }

    fn manifest_file(&self) -> PathBuf {
        self.root.join("manifest.json")
    }

    /// Read the installed-manifest registry (component → manifest).
    pub fn read_manifest(&self) -> Result<HashMap<String, UpdateManifest>, StorageError> {
        let path = self.manifest_file();
        if !path.exists() {
            return Ok(HashMap::new());
        }
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    }

    /// Record a component's installed manifest in the registry.
    pub fn write_installed(&self, manifest: &UpdateManifest) -> Result<(), StorageError> {
        let mut map = self.read_manifest()?;
        map.insert(manifest.component.as_str().to_string(), manifest.clone());
        std::fs::write(self.manifest_file(), serde_json::to_string_pretty(&map)?)?;
        Ok(())
    }

    fn prev_file(&self) -> PathBuf {
        self.root.join("backup-manifest.json")
    }

    /// Snapshot the prior installed manifest so a rollback can restore it.
    pub fn write_prev(&self, manifest: &UpdateManifest) -> Result<(), StorageError> {
        let mut map: HashMap<String, UpdateManifest> = if self.prev_file().exists() {
            serde_json::from_str(&std::fs::read_to_string(self.prev_file())?)?
        } else {
            HashMap::new()
        };
        map.insert(manifest.component.as_str().to_string(), manifest.clone());
        std::fs::write(self.prev_file(), serde_json::to_string_pretty(&map)?)?;
        Ok(())
    }

    /// The previous installed manifest for a component, if a backup exists.
    pub fn read_prev(&self, component: UpdateComponent) -> Result<Option<UpdateManifest>, StorageError> {
        if !self.prev_file().exists() {
            return Ok(None);
        }
        let map: HashMap<String, UpdateManifest> =
            serde_json::from_str(&std::fs::read_to_string(self.prev_file())?)?;
        Ok(map.get(component.as_str()).cloned())
    }
}
