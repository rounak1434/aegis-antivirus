//! Registry Run / RunOnce scanner.

use crate::model::{PersistenceEntry, PersistenceKind};

/// Build entries from (value-name, command) pairs found under a Run-style key.
pub fn entries_from_pairs<I, S>(kind: PersistenceKind, location: &str, pairs: I) -> Vec<PersistenceEntry>
where
    I: IntoIterator<Item = (S, S)>,
    S: Into<String>,
{
    pairs
        .into_iter()
        .map(|(name, cmd)| PersistenceEntry::new(kind, name.into(), cmd.into(), location.to_string()))
        .collect()
}

/// Read HKCU + HKLM Run and RunOnce keys (best-effort).
#[cfg(windows)]
pub fn collect() -> Vec<PersistenceEntry> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    let targets = [
        (HKEY_CURRENT_USER, "HKCU", "Software\\Microsoft\\Windows\\CurrentVersion\\Run", PersistenceKind::RegistryRunKey),
        (HKEY_LOCAL_MACHINE, "HKLM", "Software\\Microsoft\\Windows\\CurrentVersion\\Run", PersistenceKind::RegistryRunKey),
        (HKEY_CURRENT_USER, "HKCU", "Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce", PersistenceKind::RegistryRunOnce),
        (HKEY_LOCAL_MACHINE, "HKLM", "Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce", PersistenceKind::RegistryRunOnce),
    ];

    let mut out = Vec::new();
    for (hive, hive_label, subkey, kind) in targets {
        let root = RegKey::predef(hive);
        let Ok(key) = root.open_subkey(subkey) else {
            continue;
        };
        let location = format!("{hive_label}\\{subkey}");
        let pairs: Vec<(String, String)> = key
            .enum_values()
            .filter_map(Result::ok)
            .map(|(name, value)| (name, value.to_string()))
            .collect();
        out.extend(entries_from_pairs(kind, &location, pairs));
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
    fn builds_entries() {
        let e = entries_from_pairs(
            PersistenceKind::RegistryRunKey,
            "HKCU\\...\\Run",
            vec![("Updater", "C:\\Temp\\u.exe"), ("App", "C:\\Program Files\\app.exe")],
        );
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].kind, PersistenceKind::RegistryRunKey);
        assert_eq!(e[0].name, "Updater");
    }
}
