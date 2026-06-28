//! Windows services scanner (registry-backed).

use crate::model::{PersistenceEntry, PersistenceKind};

/// Classify a service by its registry `Type` value:
/// kernel/file-system drivers (0x1/0x2) → driver; win32 services (0x10/0x20) →
/// service. Returns `None` for other types (adapters, recognizers).
pub fn classify(service_type: u32) -> Option<PersistenceKind> {
    match service_type {
        0x1 | 0x2 => Some(PersistenceKind::DriverPersistence),
        0x10 | 0x20 => Some(PersistenceKind::ServicePersistence),
        _ => None,
    }
}

/// Build a service/driver entry from its registry fields.
pub fn entry_from_service(
    name: &str,
    image_path: &str,
    service_type: u32,
) -> Option<PersistenceEntry> {
    let kind = classify(service_type)?;
    Some(PersistenceEntry::new(
        kind,
        name,
        image_path,
        "HKLM\\SYSTEM\\CurrentControlSet\\Services",
    ))
}

/// Enumerate `HKLM\SYSTEM\CurrentControlSet\Services` (best-effort).
#[cfg(windows)]
pub fn collect() -> Vec<PersistenceEntry> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let root = RegKey::predef(HKEY_LOCAL_MACHINE);
    let Ok(services) = root.open_subkey("SYSTEM\\CurrentControlSet\\Services") else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for name in services.enum_keys().filter_map(Result::ok) {
        let Ok(svc) = services.open_subkey(&name) else {
            continue;
        };
        let service_type: u32 = svc.get_value("Type").unwrap_or(0);
        let image: String = svc.get_value("ImagePath").unwrap_or_default();
        if image.is_empty() {
            continue;
        }
        if let Some(entry) = entry_from_service(&name, &image, service_type) {
            out.push(entry);
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
    fn classify_types() {
        assert_eq!(classify(0x1), Some(PersistenceKind::DriverPersistence));
        assert_eq!(classify(0x20), Some(PersistenceKind::ServicePersistence));
        assert_eq!(classify(0x100), None);
    }

    #[test]
    fn service_entry_built() {
        let e = entry_from_service("evil", "C:\\Temp\\evil.exe", 0x10).unwrap();
        assert_eq!(e.kind, PersistenceKind::ServicePersistence);
        assert!(e.command.contains("Temp"));
    }
}
