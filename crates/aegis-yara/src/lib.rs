//! YARA-X rule management for Aegis.
//!
//! Wraps the pure-Rust `yara-x` engine: loads `.yar`/`.yara` sources from a
//! directory or strings, validates and compiles them, caches the compiled
//! `Rules`, and scans file bytes — returning structured matches.

use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;
use yara_x::{Compiler, Rules, Scanner};

#[derive(Debug, Error)]
pub enum YaraError {
    #[error("io error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("rule compilation failed: {0}")]
    Compile(String),
    #[error("no rules compiled; call load_rules() + compile_rules() first")]
    NotCompiled,
    #[error("scan error: {0}")]
    Scan(String),
}

/// One YARA rule match against a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YaraMatch {
    pub rule: String,
    pub namespace: String,
    /// Identifiers of the patterns ($a, $b, …) that matched.
    pub patterns: Vec<String>,
}

/// Loads, compiles, caches, and runs YARA-X rules.
#[derive(Default)]
pub struct RuleManager {
    sources: Vec<(String, String)>, // (origin label, source text)
    dirs: Vec<PathBuf>,
    rules: Option<Rules>,
}

impl RuleManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of source units staged for compilation.
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// True once rules have been compiled and cached.
    pub fn is_compiled(&self) -> bool {
        self.rules.is_some()
    }

    /// Stage a single rule source string (origin is a label for diagnostics).
    pub fn add_source(&mut self, origin: impl Into<String>, source: impl Into<String>) {
        self.sources.push((origin.into(), source.into()));
    }

    /// Stage every `.yar`/`.yara` file under `dir` (non-recursive) and remember
    /// the directory for `reload_rules`.
    pub fn load_rules(&mut self, dir: impl AsRef<Path>) -> Result<usize, YaraError> {
        let dir = dir.as_ref().to_path_buf();
        let added = self.read_dir(&dir)?;
        self.dirs.push(dir);
        Ok(added)
    }

    fn read_dir(&mut self, dir: &Path) -> Result<usize, YaraError> {
        let entries = fs::read_dir(dir).map_err(|source| YaraError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let mut added = 0;
        for entry in entries {
            let entry = entry.map_err(|source| YaraError::Io {
                path: dir.to_path_buf(),
                source,
            })?;
            let path = entry.path();
            let is_rule = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("yar") || e.eq_ignore_ascii_case("yara"))
                .unwrap_or(false);
            if !is_rule {
                continue;
            }
            let text = fs::read_to_string(&path).map_err(|source| YaraError::Io {
                path: path.clone(),
                source,
            })?;
            self.sources.push((path.display().to_string(), text));
            added += 1;
        }
        Ok(added)
    }

    /// Validate a rule source without staging it. `Ok(())` means it compiles.
    pub fn validate(source: &str) -> Result<(), YaraError> {
        let mut compiler = Compiler::new();
        compiler
            .add_source(source)
            .map_err(|e| YaraError::Compile(e.to_string()))?;
        Ok(())
    }

    /// Compile all staged sources into a cached `Rules` set.
    pub fn compile_rules(&mut self) -> Result<(), YaraError> {
        let mut compiler = Compiler::new();
        for (origin, source) in &self.sources {
            compiler
                .add_source(source.as_str())
                .map_err(|e| YaraError::Compile(format!("{origin}: {e}")))?;
        }
        self.rules = Some(compiler.build());
        Ok(())
    }

    /// Re-read every directory registered via `load_rules`, then recompile.
    /// String sources added via `add_source` are preserved.
    pub fn reload_rules(&mut self) -> Result<(), YaraError> {
        let dirs = self.dirs.clone();
        // Keep only string-origin sources (those not from a tracked dir file).
        self.sources.retain(|(origin, _)| {
            !dirs
                .iter()
                .any(|d| origin.starts_with(&d.display().to_string()))
        });
        for dir in &dirs {
            self.read_dir(dir)?;
        }
        self.compile_rules()
    }

    /// Scan a file's bytes against the compiled rules.
    pub fn scan_file(&self, path: impl AsRef<Path>) -> Result<Vec<YaraMatch>, YaraError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| YaraError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        self.scan_bytes(&bytes)
    }

    /// Scan raw bytes against the compiled rules.
    pub fn scan_bytes(&self, data: &[u8]) -> Result<Vec<YaraMatch>, YaraError> {
        let rules = self.rules.as_ref().ok_or(YaraError::NotCompiled)?;
        let mut scanner = Scanner::new(rules);
        let results = scanner
            .scan(data)
            .map_err(|e| YaraError::Scan(e.to_string()))?;
        let matches = results
            .matching_rules()
            .map(|rule| YaraMatch {
                rule: rule.identifier().to_string(),
                namespace: rule.namespace().to_string(),
                patterns: rule
                    .patterns()
                    .filter(|p| p.matches().count() > 0)
                    .map(|p| p.identifier().to_string())
                    .collect(),
            })
            .collect();
        Ok(matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULE: &str = r#"
rule eicar_like {
  strings:
    $a = "X5O!P%@AP"
  condition:
    $a
}
"#;

    #[test]
    fn validate_good_and_bad() {
        assert!(RuleManager::validate(RULE).is_ok());
        assert!(RuleManager::validate("rule broken { condition: }").is_err());
    }

    #[test]
    fn compile_and_match_bytes() {
        let mut mgr = RuleManager::new();
        mgr.add_source("inline", RULE);
        mgr.compile_rules().unwrap();
        assert!(mgr.is_compiled());
        let hits = mgr.scan_bytes(b"prefix X5O!P%@AP suffix").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule, "eicar_like");
        assert!(hits[0].patterns.contains(&"$a".to_string()));
    }

    #[test]
    fn no_match_returns_empty() {
        let mut mgr = RuleManager::new();
        mgr.add_source("inline", RULE);
        mgr.compile_rules().unwrap();
        assert!(mgr.scan_bytes(b"clean content").unwrap().is_empty());
    }

    #[test]
    fn scan_before_compile_errors() {
        let mgr = RuleManager::new();
        assert!(matches!(mgr.scan_bytes(b"x"), Err(YaraError::NotCompiled)));
    }

    #[test]
    fn load_rules_from_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("r.yar"), RULE).unwrap();
        std::fs::write(dir.path().join("ignore.txt"), "not a rule").unwrap();
        let mut mgr = RuleManager::new();
        let n = mgr.load_rules(dir.path()).unwrap();
        assert_eq!(n, 1);
        mgr.compile_rules().unwrap();
        assert_eq!(mgr.scan_bytes(b"X5O!P%@AP").unwrap().len(), 1);
    }
}
