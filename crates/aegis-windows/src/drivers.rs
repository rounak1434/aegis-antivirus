//! Driver scanner (parses `driverquery /v /fo csv`).

use crate::model::{PersistenceEntry, PersistenceKind};
use crate::util::{header_index, parse_csv};

/// Parse `driverquery /v /fo csv` output into driver entries.
pub fn parse_driverquery_csv(text: &str) -> Vec<PersistenceEntry> {
    let rows = parse_csv(text);
    let Some(header) = rows.first() else {
        return Vec::new();
    };
    let name_col = header_index(header, "Module Name");
    let path_col = header_index(header, "Path");
    let start_col = header_index(header, "Start Mode");
    let (Some(name_col), Some(path_col)) = (name_col, path_col) else {
        return Vec::new();
    };

    let mut entries = Vec::new();
    for row in rows.iter().skip(1) {
        let name = row.get(name_col).map(String::as_str).unwrap_or("");
        if name.is_empty() {
            continue;
        }
        let path = row.get(path_col).map(String::as_str).unwrap_or("");
        let start = start_col
            .and_then(|i| row.get(i))
            .map(String::as_str)
            .unwrap_or("");
        entries.push(
            PersistenceEntry::new(PersistenceKind::DriverPersistence, name, path, "drivers")
                .with_detail(format!("start: {start}")),
        );
    }
    entries
}

#[cfg(windows)]
pub fn collect() -> Vec<PersistenceEntry> {
    use std::process::Command;
    let out = Command::new("driverquery")
        .args(["/v", "/fo", "csv"])
        .output();
    match out {
        Ok(o) if o.status.success() => parse_driverquery_csv(&String::from_utf8_lossy(&o.stdout)),
        _ => Vec::new(),
    }
}

#[cfg(not(windows))]
pub fn collect() -> Vec<PersistenceEntry> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_drivers() {
        let csv = "\"Module Name\",\"Display Name\",\"Path\",\"Start Mode\"\n\
                   \"evilsys\",\"Evil\",\"C:\\Users\\a\\AppData\\Local\\Temp\\evil.sys\",\"Auto\"\n";
        let e = parse_driverquery_csv(csv);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].name, "evilsys");
        assert!(e[0].command.contains("Temp"));
    }
}
