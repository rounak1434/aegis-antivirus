//! Hash signature database for Aegis.
//!
//! Holds known-bad SHA-256 and MD5 digests in an in-memory cache for O(1)
//! lookups during a scan, with two backing sources that can be (re)loaded:
//! local signature files and a SQLite `signatures` table. The cache is the hot
//! path; sources exist so `reload()` can rebuild it without a restart.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SignatureError {
    #[error("io error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("invalid signature line {line}: {reason}")]
    InvalidLine { line: usize, reason: String },
}

/// Which digest a signature matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgo {
    Sha256,
    Md5,
}

/// A reloadable source of signatures.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Source {
    File(PathBuf),
    SqliteFile(PathBuf),
}

/// In-memory set of known-bad hashes, rebuilt from its sources on `reload()`.
#[derive(Debug, Default)]
pub struct SignatureDatabase {
    sha256: HashSet<String>,
    md5: HashSet<String>,
    sources: Vec<Source>,
}

impl SignatureDatabase {
    /// Empty database with no sources.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of loaded signatures (sha256 + md5).
    pub fn len(&self) -> usize {
        self.sha256.len() + self.md5.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sha256.is_empty() && self.md5.is_empty()
    }

    /// Insert one digest directly into the cache (does not register a source).
    pub fn insert(&mut self, algo: HashAlgo, hex: &str) {
        let norm = hex.trim().to_ascii_lowercase();
        if norm.is_empty() {
            return;
        }
        match algo {
            HashAlgo::Sha256 => self.sha256.insert(norm),
            HashAlgo::Md5 => self.md5.insert(norm),
        };
    }

    /// True if `hex` (any case) is a known-bad SHA-256.
    pub fn contains_sha256(&self, hex: &str) -> bool {
        self.sha256.contains(&hex.trim().to_ascii_lowercase())
    }

    /// True if `hex` (any case) is a known-bad MD5.
    pub fn contains_md5(&self, hex: &str) -> bool {
        self.md5.contains(&hex.trim().to_ascii_lowercase())
    }

    /// Register and load a local signature file. Each non-empty, non-`#` line
    /// is `sha256:<hex>`, `md5:<hex>`, or a bare hex digest (64 → sha256,
    /// 32 → md5).
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<usize, SignatureError> {
        let path = path.as_ref().to_path_buf();
        let added = self.read_file(&path)?;
        self.sources.push(Source::File(path));
        Ok(added)
    }

    /// Register and load signatures from a SQLite database file.
    pub fn load_sqlite_file(&mut self, path: impl AsRef<Path>) -> Result<usize, SignatureError> {
        let path = path.as_ref().to_path_buf();
        let conn = Connection::open(&path)?;
        let added = self.read_sqlite(&conn)?;
        self.sources.push(Source::SqliteFile(path));
        Ok(added)
    }

    /// Load signatures from an already-open connection (not registered as a
    /// reload source — use `load_sqlite_file` for that).
    pub fn load_sqlite(&mut self, conn: &Connection) -> Result<usize, SignatureError> {
        self.read_sqlite(conn)
    }

    /// Clear the cache and rebuild it from every registered source.
    pub fn reload(&mut self) -> Result<usize, SignatureError> {
        self.sha256.clear();
        self.md5.clear();
        let sources = self.sources.clone();
        let mut total = 0;
        for src in &sources {
            total += match src {
                Source::File(p) => self.read_file(p)?,
                Source::SqliteFile(p) => {
                    let conn = Connection::open(p)?;
                    self.read_sqlite(&conn)?
                }
            };
        }
        Ok(total)
    }

    fn read_file(&mut self, path: &Path) -> Result<usize, SignatureError> {
        let text = fs::read_to_string(path).map_err(|source| SignatureError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let mut added = 0;
        for (i, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (algo, hex) = parse_line(line, i + 1)?;
            self.insert(algo, hex);
            added += 1;
        }
        Ok(added)
    }

    fn read_sqlite(&mut self, conn: &Connection) -> Result<usize, SignatureError> {
        let mut stmt = conn.prepare("SELECT algo, hex FROM signatures")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut added = 0;
        for row in rows {
            let (algo, hex) = row?;
            match algo.to_ascii_lowercase().as_str() {
                "sha256" => self.insert(HashAlgo::Sha256, &hex),
                "md5" => self.insert(HashAlgo::Md5, &hex),
                _ => continue,
            }
            added += 1;
        }
        Ok(added)
    }
}

fn parse_line(line: &str, lineno: usize) -> Result<(HashAlgo, &str), SignatureError> {
    if let Some(rest) = line.strip_prefix("sha256:") {
        return Ok((HashAlgo::Sha256, rest.trim()));
    }
    if let Some(rest) = line.strip_prefix("md5:") {
        return Ok((HashAlgo::Md5, rest.trim()));
    }
    let is_hex = line.bytes().all(|b| b.is_ascii_hexdigit());
    match (is_hex, line.len()) {
        (true, 64) => Ok((HashAlgo::Sha256, line)),
        (true, 32) => Ok((HashAlgo::Md5, line)),
        _ => Err(SignatureError::InvalidLine {
            line: lineno,
            reason: "expected 'sha256:'/'md5:' prefix or a 64/32-char hex digest".into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_lookup_case_insensitive() {
        let mut db = SignatureDatabase::new();
        db.insert(HashAlgo::Sha256, "ABCDEF");
        assert!(db.contains_sha256("abcdef"));
        assert!(db.contains_sha256("ABCDEF"));
        assert!(!db.contains_md5("abcdef"));
        assert_eq!(db.len(), 1);
    }

    #[test]
    fn load_file_parses_all_forms() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("sigs.txt");
        let sha = "a".repeat(64);
        let md5 = "b".repeat(32);
        std::fs::write(
            &p,
            format!("# header\nsha256:{sha}\nmd5:{md5}\n{}\n", "c".repeat(64)),
        )
        .unwrap();
        let mut db = SignatureDatabase::new();
        let n = db.load_file(&p).unwrap();
        assert_eq!(n, 3);
        assert!(db.contains_sha256(&sha));
        assert!(db.contains_md5(&md5));
        assert!(db.contains_sha256(&"c".repeat(64)));
    }

    #[test]
    fn reload_rebuilds_from_sources() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("s.txt");
        std::fs::write(&p, format!("sha256:{}\n", "d".repeat(64))).unwrap();
        let mut db = SignatureDatabase::new();
        db.load_file(&p).unwrap();
        assert_eq!(db.len(), 1);
        std::fs::write(
            &p,
            format!("sha256:{}\nmd5:{}\n", "d".repeat(64), "e".repeat(32)),
        )
        .unwrap();
        let n = db.reload().unwrap();
        assert_eq!(n, 2);
        assert!(db.contains_md5(&"e".repeat(32)));
    }

    #[test]
    fn bad_line_errors() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("bad.txt");
        std::fs::write(&p, "not-a-hash\n").unwrap();
        let mut db = SignatureDatabase::new();
        assert!(db.load_file(&p).is_err());
    }
}
