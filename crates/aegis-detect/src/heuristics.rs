//! Stateless heuristic detectors over a file's name and content bytes.

use std::path::Path;

/// Dangerous extensions (lowercase, no dot).
pub const SUSPICIOUS_EXTS: &[&str] = &[
    "scr", "pif", "com", "hta", "js", "jse", "vbs", "vbe", "wsf", "ps1",
];

/// Extensions treated as "executable / runnable" for double-extension and
/// packed-executable heuristics.
pub const EXECUTABLE_EXTS: &[&str] = &[
    "exe", "dll", "scr", "com", "pif", "sys", "bat", "cmd", "hta", "msi", "ps1",
    "vbs", "js", "jse", "vbe", "wsf",
];

/// Common decoy extensions used to disguise the real one.
pub const DECOY_EXTS: &[&str] = &[
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "jpg", "jpeg", "png",
    "gif", "txt", "zip", "rar", "mp3", "mp4", "csv",
];

/// Script-host indicators searched for in content.
pub const SCRIPT_INDICATORS: &[&str] = &["powershell", "cmd.exe", "wscript", "cscript"];

/// PowerShell abuse indicators searched for in content (matched lowercase).
pub const POWERSHELL_INDICATORS: &[&str] =
    &["encodedcommand", "downloadstring", "invoke-expression", "iex", "bypass"];

/// Max bytes of content inspected by the string/entropy heuristics.
pub const CONTENT_SCAN_LIMIT: usize = 1024 * 1024;

fn file_name_lower(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn final_ext(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
}

/// Detect a decoy double extension like `invoice.pdf.exe` → `(decoy, real)`.
pub fn double_extension(path: &Path) -> Option<(String, String, String)> {
    let name = file_name_lower(path);
    let parts: Vec<&str> = name.split('.').collect();
    if parts.len() < 3 {
        return None;
    }
    let real = parts[parts.len() - 1];
    let decoy = parts[parts.len() - 2];
    if EXECUTABLE_EXTS.contains(&real) && DECOY_EXTS.contains(&decoy) {
        Some((name.clone(), decoy.to_string(), real.to_string()))
    } else {
        None
    }
}

/// Return the suspicious extension if the file's final extension is dangerous.
pub fn suspicious_extension(path: &Path) -> Option<String> {
    final_ext(path).filter(|e| SUSPICIOUS_EXTS.contains(&e.as_str()))
}

/// True if the file looks executable: PE `MZ` magic or an executable extension.
pub fn is_executable(path: &Path, content: &[u8]) -> bool {
    if content.starts_with(b"MZ") {
        return true;
    }
    final_ext(path)
        .map(|e| EXECUTABLE_EXTS.contains(&e.as_str()))
        .unwrap_or(false)
}

/// Shannon entropy (0.0–8.0) of `data`. Empty input → 0.0.
pub fn shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut counts = [0u64; 256];
    for &b in data {
        counts[b as usize] += 1;
    }
    let len = data.len() as f64;
    counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum()
}

/// Find script-host indicator strings present in content (case-insensitive).
pub fn script_indicators(content: &[u8]) -> Vec<String> {
    let hay = lowered(content);
    SCRIPT_INDICATORS
        .iter()
        .filter(|ind| hay.contains(*ind))
        .map(|s| s.to_string())
        .collect()
}

/// Find PowerShell abuse indicator strings present in content (case-insensitive).
pub fn powershell_indicators(content: &[u8]) -> Vec<String> {
    let hay = lowered(content);
    POWERSHELL_INDICATORS
        .iter()
        .filter(|ind| hay.contains(*ind))
        .map(|s| s.to_string())
        .collect()
}

fn lowered(content: &[u8]) -> String {
    let end = content.len().min(CONTENT_SCAN_LIMIT);
    String::from_utf8_lossy(&content[..end]).to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn double_ext_detected() {
        let d = double_extension(&PathBuf::from("invoice.pdf.exe")).unwrap();
        assert_eq!(d.1, "pdf");
        assert_eq!(d.2, "exe");
        assert!(double_extension(&PathBuf::from("photo.jpg.scr")).is_some());
        assert!(double_extension(&PathBuf::from("report.docx.bat")).is_some());
        assert!(double_extension(&PathBuf::from("normal.exe")).is_none());
        assert!(double_extension(&PathBuf::from("archive.tar.gz")).is_none());
    }

    #[test]
    fn suspicious_ext_detected() {
        assert_eq!(suspicious_extension(&PathBuf::from("x.scr")).as_deref(), Some("scr"));
        assert_eq!(suspicious_extension(&PathBuf::from("x.PS1")).as_deref(), Some("ps1"));
        assert!(suspicious_extension(&PathBuf::from("x.txt")).is_none());
    }

    #[test]
    fn entropy_bounds() {
        assert_eq!(shannon_entropy(&[]), 0.0);
        assert!(shannon_entropy(&[0u8; 1000]) < 0.001); // all same → ~0
        let varied: Vec<u8> = (0..=255).cycle().take(4096).collect();
        assert!(shannon_entropy(&varied) > 7.9); // uniform → ~8
    }

    #[test]
    fn executable_detection() {
        assert!(is_executable(&PathBuf::from("a.txt"), b"MZ\x90\x00"));
        assert!(is_executable(&PathBuf::from("a.exe"), b"plain"));
        assert!(!is_executable(&PathBuf::from("a.txt"), b"plain"));
    }

    #[test]
    fn indicators_found() {
        let ps = b"powershell -EncodedCommand ABC; IEX (New-Object Net.WebClient).DownloadString('x')";
        let s = script_indicators(ps);
        assert!(s.contains(&"powershell".to_string()));
        let p = powershell_indicators(ps);
        assert!(p.contains(&"encodedcommand".to_string()));
        assert!(p.contains(&"downloadstring".to_string()));
        assert!(p.contains(&"iex".to_string()));
        assert!(powershell_indicators(b"clean text").is_empty());
    }
}
