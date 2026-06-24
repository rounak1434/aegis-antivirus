//! Browser-extension scanner (Chrome, Edge, Firefox).

use std::path::Path;

use crate::model::{PersistenceEntry, PersistenceKind};

/// Scan a Chromium `Extensions` directory (`<id>/<version>/manifest.json`).
/// An extension whose manifest has no `update_url` is treated as
/// unpacked/sideloaded (store extensions carry an update URL).
pub fn scan_chromium_extensions(extensions_dir: &Path, browser: &str) -> Vec<PersistenceEntry> {
    let mut out = Vec::new();
    let Ok(ids) = std::fs::read_dir(extensions_dir) else {
        return out;
    };
    for id_entry in ids.flatten() {
        if !id_entry.path().is_dir() {
            continue;
        }
        let id = id_entry.file_name().to_string_lossy().to_string();
        let Ok(versions) = std::fs::read_dir(id_entry.path()) else {
            continue;
        };
        for ver in versions.flatten() {
            let manifest = ver.path().join("manifest.json");
            if !manifest.is_file() {
                continue;
            }
            let text = std::fs::read_to_string(&manifest).unwrap_or_default();
            let json: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
            let name = json
                .get("name")
                .and_then(|v| v.as_str())
                .filter(|s| !s.starts_with("__MSG_"))
                .unwrap_or(&id)
                .to_string();
            let sideloaded = json.get("update_url").is_none();
            out.push(
                PersistenceEntry::new(
                    PersistenceKind::BrowserExtension,
                    name,
                    id.clone(),
                    manifest.display().to_string(),
                )
                .with_detail(if sideloaded {
                    format!("{browser}: unpacked (no update_url)")
                } else {
                    format!("{browser}: store")
                }),
            );
        }
    }
    out
}

/// Scan a Firefox `extensions` directory for `.xpi` add-ons.
pub fn scan_firefox_extensions(extensions_dir: &Path) -> Vec<PersistenceEntry> {
    let mut out = Vec::new();
    let Ok(items) = std::fs::read_dir(extensions_dir) else {
        return out;
    };
    for item in items.flatten() {
        let path = item.path();
        if path.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("xpi")) != Some(true) {
            continue;
        }
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("addon").to_string();
        out.push(
            PersistenceEntry::new(
                PersistenceKind::BrowserExtension,
                name,
                path.display().to_string(),
                extensions_dir.display().to_string(),
            )
            .with_detail("firefox: xpi"),
        );
    }
    out
}

/// Best-effort scan of Chrome, Edge, and Firefox profiles.
#[cfg(windows)]
pub fn collect() -> Vec<PersistenceEntry> {
    let mut out = Vec::new();
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let chromium = [
            ("Chrome", format!("{local}\\Google\\Chrome\\User Data\\Default\\Extensions")),
            ("Edge", format!("{local}\\Microsoft\\Edge\\User Data\\Default\\Extensions")),
        ];
        for (browser, dir) in chromium {
            out.extend(scan_chromium_extensions(Path::new(&dir), browser));
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let profiles = format!("{appdata}\\Mozilla\\Firefox\\Profiles");
        if let Ok(dirs) = std::fs::read_dir(&profiles) {
            for d in dirs.flatten() {
                out.extend(scan_firefox_extensions(&d.path().join("extensions")));
            }
        }
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
    fn detects_unpacked_extension() {
        let dir = tempfile::tempdir().unwrap();
        let ext = dir.path().join("abcdefghijklmnop").join("1.0");
        std::fs::create_dir_all(&ext).unwrap();
        std::fs::write(ext.join("manifest.json"), r#"{"name":"Sneaky"}"#).unwrap();
        let e = scan_chromium_extensions(dir.path(), "Chrome");
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].name, "Sneaky");
        assert!(e[0].detail.contains("unpacked"));
    }

    #[test]
    fn store_extension_has_update_url() {
        let dir = tempfile::tempdir().unwrap();
        let ext = dir.path().join("store-id").join("2.0");
        std::fs::create_dir_all(&ext).unwrap();
        std::fs::write(ext.join("manifest.json"), r#"{"name":"Good","update_url":"https://clients2.google.com/service/update2/crx"}"#).unwrap();
        let e = scan_chromium_extensions(dir.path(), "Chrome");
        assert!(e[0].detail.contains("store"));
    }
}
