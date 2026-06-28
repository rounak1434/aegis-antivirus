---
title: "Reporting: export scan/detection reports (JSON, HTML, PDF)"
labels: ["enhancement", "reporting"]
---

## Objective

Generate shareable reports for a scan or a time range from the persisted
detections, jobs, and quarantine records.

## Requirements

- New `aegis-reporting` crate that reads from the existing SQLite tables
  (`detection_results`, `job_history`, `quarantine_records`, `scan_events`).
- Formats: **JSON** (machine-readable), **HTML** (styled, matches the app
  theme), **PDF** (print/export).
- Reports include scan summary, detections with explainable evidence, actions
  taken, and timing/throughput.
- Service exposes a `generate_report(scope, format)` command.

## Acceptance Criteria

- [ ] JSON/HTML/PDF generated from fixture data in tests.
- [ ] Evidence + actions appear and are explainable (no bare scores).
- [ ] `cargo test` + `cargo clippy … -D warnings` pass; docs updated.
