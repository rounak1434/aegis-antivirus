# Changelog

All notable changes to Aegis Antivirus will be documented in this file.

## 0.1.0-phase1 - Unreleased

### Added

- Documented service-owned architecture for `AegisService`.
- Defined desktop UI, Windows service, engine crates, database layer, update system, and IPC boundary.
- Added Phase 1 roadmap and task plan.
- Added native Tauri v2 + React + TypeScript + TailwindCSS + Zustand frontend scaffold.
- Added Rust workspace with `aegis-common`, `aegis-ipc`, `aegis-db`, `aegis-service`, `aegis-update`, and `aegis-quarantine` crates.
- Added initial SQLite migration for settings, scan history, threats, detections, quarantine, YARA rules, notifications, update history, and audit logs.
- Added `AegisService` Windows service lifecycle skeleton.
- Added Tauri command bridge for service status and scan requests.
- Added `DEVELOPMENT.md` with local tooling and validation commands.

### Changed

- Reclassified the existing HTML work as design prototype material and preserved it under `design-prototype/`.

### Security

- Established privilege-separation requirement: the Tauri app is UI-only; `AegisService` owns monitoring, scheduled scans, quarantine, and updates.
- Upgraded frontend dev tooling until `npm audit --audit-level=moderate` reports zero vulnerabilities.
