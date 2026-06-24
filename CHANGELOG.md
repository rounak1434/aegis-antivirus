# Changelog

All notable changes to Aegis Antivirus will be documented in this file.

## Unreleased - Phase 3: Detection Engine — VERIFIED

### Added
- `aegis-signatures` crate — `SignatureDatabase` for SHA-256/MD5 known-bad
  hashes: SQLite + local-file sources + in-memory cache, `load`/`reload`/
  `contains_sha256`/`contains_md5`.
- `aegis-yara` crate — `RuleManager` over YARA-X 1.19: load (dir/strings),
  validate, compile, cache compiled `Rules`, reload, and `scan_file`/`scan_bytes`.
- `aegis-detect` crate — detection engine above scanner output:
  - threat model: `ThreatLevel`, `ThreatEvidence` (8 variants), `ThreatDetection`;
  - heuristics: double extension, suspicious extension, Shannon entropy /
    packed-executable, script indicators, PowerShell-abuse indicators;
  - additive 0–100 risk scoring with per-evidence `reason()` (no black-box scores)
    and level thresholds (Safe/Low/Medium/High/Critical);
  - `DetectionEngine` orchestrating hash + YARA + heuristic layers over
    `ScannedFile` / `ScanReport`;
  - persistence: `persist_detection`, `record_scan_event`, `detection_count`.
- DB migration `002_detection.sql`: `signature_sets`, `signatures`,
  `detection_results`, `scan_events` (history/audit/reporting).
- `aegis-detect` benchmark (`benches/detect_bench.rs`, 10k files, peak-memory
  via `K32GetProcessMemoryInfo` on Windows).
- `DETECTION_ENGINE.md` — architecture, data flow, scoring, performance, limits.

### Changed
- `aegis-db::apply_migrations` now applies an ordered migration list (001 + 002),
  recording each version and skipping already-applied ones (idempotent).

### Verified
- `cargo test -p aegis-signatures -p aegis-yara -p aegis-detect`: 25/25 pass.
- `cargo clippy --workspace --exclude aegis-tauri --all-targets --all-features
  -- -D warnings`: clean.
- Benchmark (release, 10,000 files): scan 171 ms; detect 6,074 ms →
  1,646 files/s, 607 µs/file, 800 detections, 27.5 MiB peak working set.

### Notes
- Detection is single-threaded (parallelization is the next throughput item);
  `aegis-tauri` still excluded from the clippy gate pending the Phase-12 icon set.

## Unreleased - Phase 2: File Scanner (`aegis-scan`) — VERIFIED

### Added
- Live scan metrics on `ScanProgress` (`total_files`, `percent`, `elapsed_ms`,
  `files_per_sec`, `bytes_per_sec`, `eta_ms`) and aggregate throughput on
  `ScanReport` (`files_per_sec`, `bytes_per_sec`).
- Integration test suite (`tests/integration.rs`): nested directories, hidden
  files, symlink handling, permission-denied paths, large-file hashing, scan
  cancellation, and progress-metric reporting.
- Throughput benchmark example (`examples/bench.rs`).
- `SCANNER_VALIDATION.md` — environment, test/clippy/benchmark evidence,
  ETA method, and limitations.

### Changed
- `hex()` digest formatting rewritten without per-byte `format!` (clippy-clean,
  faster).
- `aegis-service::runtime::subscribe_status` annotated `#[allow(dead_code)]`
  (IPC API consumed in a later phase) to keep `clippy -D warnings` clean.

### Verified
- `cargo test -p aegis-scan`: 12/12 pass. `cargo clippy --workspace --exclude
  aegis-tauri --all-targets --all-features -- -D warnings`: clean. Benchmark:
  4,000 files / 62.5 MiB in 133 ms → 29,981 files/s · 468.5 MiB/s (8 threads).

### Known gaps (not scanner)
- `aegis-tauri` build needs `icons/icon.ico` (Phase 12 packaging asset);
  excluded from the workspace clippy gate until icons land.

## Unreleased - Prototype Migration (Phase B: all core screens)

### Added
- Converted **Scan Center** (`features/scan-center`) — posture ring, per-engine
  activity, current path, and a live detections feed driven by a self-contained
  `useScanSimulation` hook (swappable for IPC ScanProgress events in Phase D).
- Converted **Threat Center** (`features/threat-center`) — detections table,
  severity/risk chips, filter chips, and the per-threat evidence drawer (risk
  score, recommended action, file evidence, detection layers, reasons; Esc to close).
- Converted **Quarantine** (`features/quarantine`) — encrypted-vault banner,
  stats, bulk selection bar with select-all, and restore/delete (delete confirm).
- Converted **Real-time** (`features/realtime`) — 6 shield cards with working
  toggles, live event feed, controlled-folder access, allowed apps.
- Converted **Settings** (`features/settings`) — sticky sub-tabs, engine/behaviour/
  privacy toggles, exclusions, and the simulated signed signature-update flow.
- Routed all five screens in the HashRouter shell.

## Unreleased - Prototype Migration (Phase A + B slice)

### Added
- `PROTOTYPE_AUDIT.md` — authoritative inventory of the design prototype:
  design tokens, UI/component inventory, navigation map, and 9-screen map.
- React app shell rebuilt to match the prototype: frameless `WinBar` (real
  Tauri window controls), grouped `Sidebar` (Overview/Protect/System), sticky
  `TopBar` with per-route title/crumb.
- Typed `<Icon>` component ported from the prototype's `shell.js` icon set.
- Fully converted **Dashboard** screen (posture ring, scan-launch with mode
  tiles, stat tiles, persistence surface health, recent activity) with typed
  seed data in `features/dashboard/data.ts`.
- `react-router-dom` (HashRouter) with an `AppShell` layout route; interim
  `SectionComingNext` view for screens queued in later slices.
- `.claude/launch.json` dev-server config for the Aegis UI.

### Changed
- **Replaced the generic emerald/slate React template** that diverged from the
  prototype. `tailwind.config.ts` and `src/styles.css` now encode the
  prototype's warm-dark / terracotta Anthropic design system verbatim (the
  single source of truth for pixel parity).

### Verified
- `npm run build` (`tsc && vite build`) passes; 0 npm vulnerabilities after
  adding the router.

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
