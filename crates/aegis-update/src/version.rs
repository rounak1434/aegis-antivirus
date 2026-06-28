//! Dotted-numeric version comparison (handles `2024.06.22.02`, `1.2.3`, …).

use std::cmp::Ordering;

/// Parse a dotted version into numeric components (non-numeric parts → 0).
fn parts(v: &str) -> Vec<u64> {
    v.split('.')
        .map(|p| p.trim().parse::<u64>().unwrap_or(0))
        .collect()
}

/// Compare two versions component-wise (shorter is zero-padded).
pub fn compare(a: &str, b: &str) -> Ordering {
    let (pa, pb) = (parts(a), parts(b));
    let n = pa.len().max(pb.len());
    for i in 0..n {
        let x = pa.get(i).copied().unwrap_or(0);
        let y = pb.get(i).copied().unwrap_or(0);
        match x.cmp(&y) {
            Ordering::Equal => continue,
            other => return other,
        }
    }
    Ordering::Equal
}

/// True if `candidate` is strictly newer than `current`.
pub fn is_newer(candidate: &str, current: &str) -> bool {
    compare(candidate, current) == Ordering::Greater
}

/// True if `version` is >= `minimum`.
pub fn at_least(version: &str, minimum: &str) -> bool {
    compare(version, minimum) != Ordering::Less
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_detection() {
        assert!(is_newer("2024.06.22.02", "2024.06.22.01"));
        assert!(is_newer("1.2.0", "1.1.9"));
        assert!(!is_newer("1.0.0", "1.0.0"));
        assert!(!is_newer("1.0.0", "1.0.1"));
    }

    #[test]
    fn zero_pad_compare() {
        assert_eq!(compare("1.2", "1.2.0"), Ordering::Equal);
        assert!(is_newer("1.2.1", "1.2"));
    }

    #[test]
    fn min_app_version() {
        assert!(at_least("1.5.0", "1.0.0"));
        assert!(at_least("1.0.0", "1.0.0"));
        assert!(!at_least("0.9.0", "1.0.0"));
    }
}
