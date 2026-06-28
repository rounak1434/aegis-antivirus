//! Hosts-file scanner.

use crate::model::{PersistenceEntry, PersistenceKind};

/// Parse hosts-file text into one entry per (host → IP) mapping.
pub fn parse_hosts(text: &str) -> Vec<PersistenceEntry> {
    let mut entries = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Strip inline comments.
        let line = line.split('#').next().unwrap_or("").trim();
        let mut parts = line.split_whitespace();
        let Some(ip) = parts.next() else { continue };
        for host in parts {
            entries.push(
                PersistenceEntry::new(PersistenceKind::HostsFileModification, host, ip, "hosts")
                    .with_detail(format!("{ip} {host}")),
            );
        }
    }
    entries
}

/// Read and parse the Windows hosts file (best-effort).
#[cfg(windows)]
pub fn collect() -> Vec<PersistenceEntry> {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
    let path = format!("{root}\\System32\\drivers\\etc\\hosts");
    match std::fs::read_to_string(path) {
        Ok(text) => parse_hosts(&text),
        Err(_) => Vec::new(),
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
    fn parses_mappings_and_skips_comments() {
        let text =
            "# comment\n127.0.0.1 localhost\n10.0.0.5 www.microsoft.com evil.test # inline\n\n";
        let e = parse_hosts(text);
        assert_eq!(e.len(), 3);
        assert_eq!(e[0].name, "localhost");
        assert_eq!(e[1].name, "www.microsoft.com");
        assert_eq!(e[1].command, "10.0.0.5");
        assert_eq!(e[2].name, "evil.test");
    }
}
