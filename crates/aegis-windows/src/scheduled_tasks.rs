//! Scheduled-tasks scanner (parses `schtasks /query /fo CSV /v`).

use crate::model::{PersistenceEntry, PersistenceKind};
use crate::util::{header_index, parse_csv};

/// Parse `schtasks /query /fo CSV /v` output into scheduled-task entries.
pub fn parse_schtasks_csv(text: &str) -> Vec<PersistenceEntry> {
    let rows = parse_csv(text);
    let Some(header) = rows.first() else {
        return Vec::new();
    };
    let name_col = header_index(header, "TaskName");
    let run_col = header_index(header, "Task To Run");
    let author_col = header_index(header, "Author");
    let (Some(name_col), Some(run_col)) = (name_col, run_col) else {
        return Vec::new();
    };

    let mut entries = Vec::new();
    for row in rows.iter().skip(1) {
        // schtasks repeats the header between folders; skip those + blanks.
        let name = row.get(name_col).map(String::as_str).unwrap_or("");
        if name.is_empty() || name.eq_ignore_ascii_case("TaskName") {
            continue;
        }
        let run = row.get(run_col).map(String::as_str).unwrap_or("");
        if run.eq_ignore_ascii_case("N/A") || run.is_empty() {
            continue;
        }
        let author = author_col.and_then(|i| row.get(i)).map(String::as_str).unwrap_or("");
        entries.push(
            PersistenceEntry::new(PersistenceKind::ScheduledTask, name, run, "Task Scheduler")
                .with_detail(format!("author: {author}")),
        );
    }
    entries
}

#[cfg(windows)]
pub fn collect() -> Vec<PersistenceEntry> {
    use std::process::Command;
    let out = Command::new("schtasks")
        .args(["/query", "/fo", "CSV", "/v"])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            parse_schtasks_csv(&String::from_utf8_lossy(&o.stdout))
        }
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
    fn parses_tasks() {
        let csv = "\"HostName\",\"TaskName\",\"Task To Run\",\"Author\"\n\
                   \"PC\",\"\\Evil\",\"powershell -enc ABC\",\"hacker\"\n\
                   \"PC\",\"\\Legit\",\"N/A\",\"MS\"\n";
        let e = parse_schtasks_csv(csv);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].name, "\\Evil");
        assert!(e[0].command.contains("-enc"));
    }
}
