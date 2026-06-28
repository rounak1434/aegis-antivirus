//! Aegis file scanner engine.
//!
//! Phase 2 capability: recursive directory traversal with hidden-file and
//! symlink handling, streaming SHA-256 + MD5 hashing, file metadata collection,
//! multi-threaded hashing (rayon), and live progress + cooperative cancellation.
//!
//! The engine is filesystem-only and platform-aware (Windows hidden-attribute
//! detection behind `cfg(windows)`, dotfile fallback elsewhere) so it can be
//! reused under other operating systems later. It performs no detection — that
//! is layered on top by the YARA and heuristic engines in later phases.

use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

use aegis_common::ScanMode;
use chrono::{DateTime, Utc};
use md5::{Digest as _, Md5};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;
use walkdir::WalkDir;

/// Read buffer for streaming hashes (64 KiB).
const HASH_BUF: usize = 64 * 1024;

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("no scan roots were provided")]
    NoRoots,
    #[error("scan root does not exist: {0}")]
    MissingRoot(PathBuf),
    #[error("scan was cancelled")]
    Cancelled,
}

/// Options controlling a scan job. Build with [`ScanOptions::for_mode`] for the
/// standard Quick/Full/Deep presets, or construct directly for Custom scans.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanOptions {
    pub roots: Vec<PathBuf>,
    /// Follow symbolic links / reparse points during traversal.
    pub follow_symlinks: bool,
    /// Include hidden and system files.
    pub include_hidden: bool,
    /// Maximum directory depth (None = unlimited).
    pub max_depth: Option<usize>,
    /// Compute SHA-256 + MD5 for each file. Disable for a metadata-only pass.
    pub hash_files: bool,
}

impl ScanOptions {
    pub fn for_mode(mode: ScanMode, roots: Vec<PathBuf>) -> Self {
        match mode {
            // Quick: shallow, skip hidden/system, follow nothing — hot paths only.
            ScanMode::Quick => Self {
                roots,
                follow_symlinks: false,
                include_hidden: false,
                max_depth: Some(6),
                hash_files: true,
            },
            // Full: everything reachable, hidden included, no symlink loops.
            ScanMode::Full => Self {
                roots,
                follow_symlinks: false,
                include_hidden: true,
                max_depth: None,
                hash_files: true,
            },
            // Deep: as Full but also follows symlinks/reparse points.
            ScanMode::Deep => Self {
                roots,
                follow_symlinks: true,
                include_hidden: true,
                max_depth: None,
                hash_files: true,
            },
            // Custom: sensible defaults the caller can override field-by-field.
            ScanMode::Custom => Self {
                roots,
                follow_symlinks: false,
                include_hidden: true,
                max_depth: None,
                hash_files: true,
            },
        }
    }
}

/// SHA-256 and MD5 digests of a file, lowercase hex.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileHashes {
    pub sha256: String,
    pub md5: String,
}

/// Collected metadata for one filesystem entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub modified_utc: Option<DateTime<Utc>>,
    pub is_hidden: bool,
    pub is_symlink: bool,
}

/// A scanned file: its metadata, optional hashes, and any per-file error text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScannedFile {
    pub metadata: FileMetadata,
    pub hashes: Option<FileHashes>,
    pub error: Option<String>,
}

/// A point-in-time progress snapshot passed to the progress callback.
///
/// Carries live throughput and ETA metrics so the UI's scan screen can render
/// files/sec, bytes/sec, percent complete, and time remaining directly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScanProgress {
    pub files_scanned: u64,
    pub total_files: u64,
    pub bytes_scanned: u64,
    pub errors: u64,
    pub percent: f64,
    pub elapsed_ms: u64,
    pub files_per_sec: f64,
    pub bytes_per_sec: f64,
    pub eta_ms: u64,
    pub current_path: PathBuf,
}

/// Final result of a scan, including aggregate throughput metrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScanReport {
    pub mode: ScanMode,
    pub files_scanned: u64,
    pub bytes_scanned: u64,
    pub errors: u64,
    pub started_utc: DateTime<Utc>,
    pub finished_utc: DateTime<Utc>,
    pub duration_ms: u64,
    pub files_per_sec: f64,
    pub bytes_per_sec: f64,
    pub files: Vec<ScannedFile>,
}

/// Stream SHA-256 + MD5 over a file without loading it fully into memory.
pub fn compute_hashes(path: &Path) -> io::Result<FileHashes> {
    let mut file = std::fs::File::open(path)?;
    let mut sha = Sha256::new();
    let mut md5 = Md5::new();
    let mut buf = vec![0u8; HASH_BUF];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        sha.update(&buf[..n]);
        md5.update(&buf[..n]);
    }
    Ok(FileHashes {
        sha256: hex(&sha.finalize()),
        md5: hex(&md5.finalize()),
    })
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

#[cfg(windows)]
fn is_hidden(_path: &Path, meta: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
    meta.file_attributes() & (FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM) != 0
}

#[cfg(not(windows))]
fn is_hidden(path: &Path, _meta: &std::fs::Metadata) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}

fn system_time_to_utc(t: SystemTime) -> DateTime<Utc> {
    DateTime::<Utc>::from(t)
}

/// Run a scan. `cancel` is polled cooperatively; `on_progress` is invoked as
/// files are hashed (it must be `Sync` — it is called from worker threads).
pub fn scan<F>(
    opts: &ScanOptions,
    cancel: Arc<AtomicBool>,
    on_progress: F,
) -> Result<ScanReport, ScanError>
where
    F: Fn(ScanProgress) + Sync + Send,
{
    if opts.roots.is_empty() {
        return Err(ScanError::NoRoots);
    }
    for root in &opts.roots {
        if !root.exists() {
            return Err(ScanError::MissingRoot(root.clone()));
        }
    }

    let started = Utc::now();
    let started_instant = std::time::Instant::now();

    // 1. Traverse: collect file entries (single-threaded; directory walk is IO-bound).
    let mut entries: Vec<FileMetadata> = Vec::new();
    for root in &opts.roots {
        if cancel.load(Ordering::Relaxed) {
            return Err(ScanError::Cancelled);
        }
        let mut walker = WalkDir::new(root).follow_links(opts.follow_symlinks);
        if let Some(depth) = opts.max_depth {
            walker = walker.max_depth(depth);
        }
        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let file_type = entry.file_type();
            if !file_type.is_file() && !file_type.is_symlink() {
                continue;
            }
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if meta.is_dir() {
                continue;
            }
            let hidden = is_hidden(entry.path(), &meta);
            if hidden && !opts.include_hidden {
                continue;
            }
            entries.push(FileMetadata {
                path: entry.path().to_path_buf(),
                size_bytes: meta.len(),
                modified_utc: meta.modified().ok().map(system_time_to_utc),
                is_hidden: hidden,
                is_symlink: file_type.is_symlink(),
            });
        }
    }

    // 2. Hash in parallel (CPU/IO bound) with shared atomic progress counters.
    let total_files = entries.len() as u64;
    let files_counter = AtomicU64::new(0);
    let bytes_counter = AtomicU64::new(0);
    let errors_counter = AtomicU64::new(0);

    let cancelled = AtomicBool::new(false);

    let scanned: Vec<ScannedFile> = entries
        .par_iter()
        .map(|meta| {
            if cancel.load(Ordering::Relaxed) {
                cancelled.store(true, Ordering::Relaxed);
            }
            if cancelled.load(Ordering::Relaxed) {
                return ScannedFile {
                    metadata: meta.clone(),
                    hashes: None,
                    error: Some("cancelled".into()),
                };
            }

            let (hashes, error) = if opts.hash_files && !meta.is_symlink {
                match compute_hashes(&meta.path) {
                    Ok(h) => (Some(h), None),
                    Err(e) => {
                        errors_counter.fetch_add(1, Ordering::Relaxed);
                        (None, Some(e.to_string()))
                    }
                }
            } else {
                (None, None)
            };

            let files = files_counter.fetch_add(1, Ordering::Relaxed) + 1;
            let bytes =
                bytes_counter.fetch_add(meta.size_bytes, Ordering::Relaxed) + meta.size_bytes;
            let elapsed = started_instant.elapsed().as_secs_f64().max(1e-6);
            let fps = files as f64 / elapsed;
            let percent = if total_files > 0 {
                files as f64 / total_files as f64 * 100.0
            } else {
                100.0
            };
            let remaining = total_files.saturating_sub(files) as f64;
            let eta_ms = if fps > 0.0 {
                (remaining / fps * 1000.0) as u64
            } else {
                0
            };
            on_progress(ScanProgress {
                files_scanned: files,
                total_files,
                bytes_scanned: bytes,
                errors: errors_counter.load(Ordering::Relaxed),
                percent,
                elapsed_ms: (elapsed * 1000.0) as u64,
                files_per_sec: fps,
                bytes_per_sec: bytes as f64 / elapsed,
                eta_ms,
                current_path: meta.path.clone(),
            });

            ScannedFile {
                metadata: meta.clone(),
                hashes,
                error,
            }
        })
        .collect();

    if cancelled.load(Ordering::Relaxed) {
        return Err(ScanError::Cancelled);
    }

    let finished = Utc::now();
    let total_secs = started_instant.elapsed().as_secs_f64().max(1e-6);
    let files_scanned = files_counter.load(Ordering::Relaxed);
    let bytes_scanned = bytes_counter.load(Ordering::Relaxed);
    Ok(ScanReport {
        mode: opts_mode(opts),
        files_scanned,
        bytes_scanned,
        errors: errors_counter.load(Ordering::Relaxed),
        started_utc: started,
        finished_utc: finished,
        duration_ms: started_instant.elapsed().as_millis() as u64,
        files_per_sec: files_scanned as f64 / total_secs,
        bytes_per_sec: bytes_scanned as f64 / total_secs,
        files: scanned,
    })
}

/// Best-effort recovery of the scan mode from options (Deep follows symlinks).
fn opts_mode(opts: &ScanOptions) -> ScanMode {
    if opts.follow_symlinks {
        ScanMode::Deep
    } else if opts.include_hidden && opts.max_depth.is_none() {
        ScanMode::Full
    } else if !opts.include_hidden {
        ScanMode::Quick
    } else {
        ScanMode::Custom
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn write(path: &Path, contents: &[u8]) {
        let mut f = fs::File::create(path).unwrap();
        f.write_all(contents).unwrap();
    }

    #[test]
    fn known_hashes_for_abc() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("abc.txt");
        write(&p, b"abc");
        let h = compute_hashes(&p).unwrap();
        // Well-known digests of "abc".
        assert_eq!(
            h.sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(h.md5, "900150983cd24fb0d6963f7d28e17f72");
    }

    #[test]
    fn scans_files_and_counts() {
        let dir = tempfile::tempdir().unwrap();
        write(&dir.path().join("a.txt"), b"hello");
        write(&dir.path().join("b.bin"), b"world!!");
        let sub = dir.path().join("nested");
        fs::create_dir(&sub).unwrap();
        write(&sub.join("c.dat"), b"deep");

        let opts = ScanOptions::for_mode(ScanMode::Full, vec![dir.path().to_path_buf()]);
        let report = scan(&opts, Arc::new(AtomicBool::new(false)), |_p| {}).unwrap();

        assert_eq!(report.files_scanned, 3);
        assert_eq!(report.bytes_scanned, 5 + 7 + 4);
        assert_eq!(report.errors, 0);
        assert!(report.files.iter().all(|f| f.hashes.is_some()));
    }

    #[test]
    fn respects_max_depth() {
        let dir = tempfile::tempdir().unwrap();
        write(&dir.path().join("top.txt"), b"x");
        let sub = dir.path().join("a").join("b");
        fs::create_dir_all(&sub).unwrap();
        write(&sub.join("buried.txt"), b"y");

        let mut opts = ScanOptions::for_mode(ScanMode::Custom, vec![dir.path().to_path_buf()]);
        opts.max_depth = Some(1);
        let report = scan(&opts, Arc::new(AtomicBool::new(false)), |_p| {}).unwrap();
        assert_eq!(report.files_scanned, 1);
    }

    #[test]
    fn cancellation_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        write(&dir.path().join("a.txt"), b"hello");
        let opts = ScanOptions::for_mode(ScanMode::Full, vec![dir.path().to_path_buf()]);
        let err = scan(&opts, Arc::new(AtomicBool::new(true)), |_p| {}).unwrap_err();
        assert!(matches!(err, ScanError::Cancelled));
    }

    #[test]
    fn missing_root_errors() {
        let opts = ScanOptions::for_mode(
            ScanMode::Full,
            vec![PathBuf::from("Z:/definitely/not/here/aegis")],
        );
        let err = scan(&opts, Arc::new(AtomicBool::new(false)), |_p| {}).unwrap_err();
        assert!(matches!(err, ScanError::MissingRoot(_)));
    }
}
