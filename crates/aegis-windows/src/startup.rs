//! Startup-folder scanner.

use std::path::Path;

use crate::model::{PersistenceEntry, PersistenceKind};

/// Enumerate files in a startup directory as persistence entries.
pub fn scan_dir(dir: &Path) -> Vec<PersistenceEntry> {
    let mut entries = Vec::new();
    let Ok(read) = std::fs::read_dir(dir) else {
        return entries;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        // Skip the inert placeholder Windows ships in the Startup folder.
        if name.eq_ignore_ascii_case("desktop.ini") {
            continue;
        }
        entries.push(PersistenceEntry::new(
            PersistenceKind::StartupEntry,
            name,
            path.display().to_string(),
            dir.display().to_string(),
        ));
    }
    entries
}

/// Scan the per-user and all-users Startup folders (best-effort).
#[cfg(windows)]
pub fn collect() -> Vec<PersistenceEntry> {
    let mut out = Vec::new();
    let suffix = "Microsoft\\Windows\\Start Menu\\Programs\\Startup";
    if let Ok(appdata) = std::env::var("APPDATA") {
        out.extend(scan_dir(Path::new(&format!("{appdata}\\{suffix}"))));
    }
    if let Ok(programdata) = std::env::var("ProgramData") {
        out.extend(scan_dir(Path::new(&format!("{programdata}\\{suffix}"))));
    }
    out
}

#[cfg(not(windows))]
pub fn collect() -> Vec<PersistenceEntry> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_files_skips_desktop_ini() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("launcher.lnk"), b"x").unwrap();
        std::fs::write(dir.path().join("evil.vbs"), b"x").unwrap();
        std::fs::write(dir.path().join("desktop.ini"), b"x").unwrap();
        let e = scan_dir(dir.path());
        assert_eq!(e.len(), 2);
        assert!(e.iter().all(|x| x.kind == PersistenceKind::StartupEntry));
    }
}
