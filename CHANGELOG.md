# Changelog

All notable changes to Aegis Antivirus will be documented in this file.

## Unreleased - Phase 10: CI/CD & Quality Gates — VERIFIED

### Added
- GitHub Actions pipeline (`.github/workflows/`):
  - `ci.yml` — entry workflow (push to main + PRs) fanning out to reusable
    workflows with a `gate` job + concurrency cancellation;
  - `rust.yml` — `fmt --check`, `clippy -D warnings`, `test`, `build`
    (Windows/MSVC, `aegis-tauri` excluded), with `Swatinem/rust-cache`;
  - `frontend.yml` — `npm ci` + `npm run build` + `npm test`, npm cache;
  - `security.yml` — `cargo-deny`, `cargo-audit`, dependency review (PRs),
    gitleaks secret scan;
  - `release.yml` — **draft-only** release on `v*` tags (no packaging/signing).
- `deny.toml` (cargo-deny: advisories/licenses/bans/sources).
- `.github/dependabot.yml` (cargo + npm + github-actions, weekly).
- `.github/CODEOWNERS`; improved `pull_request_template.md` (required gate
  commands + tests + docs checklist).
- `CI_CD.md`; README CI/tests/release/license badges.

### Changed
- `cargo fmt --all` applied across the workspace so `fmt --check` passes in CI
  (formatting only — no logic changes).

### Verified
- All 9 workflow/template YAML files validated with `npx js-yaml`.
- Gate commands pass locally: `cargo fmt --check` clean, `clippy -D warnings`
  clean, `cargo test --workspace --exclude aegis-tauri` 118 pass, `npm run build`
  + `npm test` (10) green.

## Unreleased - Phase 9: UI ↔ Service Integration — VERIFIED

### Added
- **Tauri command bridge** (`src-tauri/src/commands`): 21 typed
  `#[tauri::command]`s over a managed `AegisOrchestrator` (built at
  `%LOCALAPPDATA%\Aegis`). Covers scan, threats, quarantine, windows scan,
  real-time, updates, settings, and health.
- **Typed IPC layer**: `src/lib/ipc.ts` (single `invoke` site, `ServiceError`
  normalization, `inTauri()`), `src/lib/api.ts` (domain wrappers), and snake_case
  DTO types in `src/types/ipc.ts`.
- **Zustand stores**: `scanStore`, `threatStore`, `quarantineStore`,
  `updateStore`, `realtimeStore`, `settingsStore` (+ `healthStore`).
- **Live screens**: Dashboard, Scan Center (Quick/Full/Deep/Custom + Cancel +
  live progress/ETA), Threat Center (filter/search/sort + evidence drawer),
  Quarantine (restore/delete/metadata), Real-time (state/policy/counters), new
  **Updates** page (installed/check/install/rollback), Settings (persisted).
- Error/loading/empty components (`src/components/States.tsx`).
- Orchestrator `get_settings`/`save_settings` (settings table); `InstallOutcome`
  is now serializable.
- App icon set (`src-tauri/icons/*`) so `aegis-tauri` compiles + joins the gate.
- Frontend tests (vitest + Testing Library): IPC wrappers, stores, Dashboard.

### Removed
- All mock/simulated UI data: `useScanSimulation`, threat/dashboard seed arrays,
  the old `securityStore`.

### Verified
- `npm run build` (tsc + vite): clean. `npm test` (vitest): 10/10.
- `cargo build -p aegis-tauri`: compiles (247 crates).
- `cargo test --workspace --exclude aegis-tauri`: 118 pass; clippy `-D warnings` clean.

### Notes
- Pause/Resume not added (would require modifying the scanner engine); Cancel is
  real. The UI hosts the orchestrator in-process; a separate service process over
  named-pipe IPC is the next step.

## Unreleased - Phase 8: Secure Update System — VERIFIED

### Added
- `aegis-update` crate — secure updates for signatures, YARA rules, threat
  metadata, and engine config:
  - `UpdateManifest` signed over a canonical `component|version|sha256|size`
    message; `UpdateComponent` model;
  - cryptographic gate: SHA-256 integrity + Ed25519 signature (pinned key),
    anti-rollback (strictly-newer), and minimum-app-version — each a typed
    `VerifyError`;
  - download engine (`reqwest`, blocking): byte-range resume, timeout, bounded
    retries, gzip; `Fetcher` trait with an offline `LocalFetcher` for tests;
  - storage layout (`updates/`, `installed/`, `backup/`, `manifest.json`) with
    atomic swap and backup-based rollback;
  - `UpdateSchedule` (Manual / Daily / Weekly / Startup);
  - `UpdateEngine`: check → download → install → rollback → status.
- DB migration `006_update.sql`: `installed_components` (reuses `update_history`
  from 001 for the action log).
- Orchestrator update API: `init_updates`, `check_updates`, `download_updates`,
  `install_updates` (with engine hot-reload), `rollback_updates`,
  `get_update_status`.
- `aegis-update` benchmark (`benches/update_bench.rs`); `UPDATE_SYSTEM.md`.

### Changed
- `aegis-db::apply_migrations` now applies 001 → 006.
- `aegis-update` `build.rs` embeds an `asInvoker` manifest into its test/bench
  binaries to bypass Windows UAC installer-detection (name contains "update").

### Verified
- `cargo test -p aegis-update`: 17/17 pass; `cargo test -p aegis-service`: 12/12.
- `cargo clippy --workspace --all-targets --all-features --exclude aegis-tauri
  -- -D warnings`: clean.
- Benchmark (release, 8 MiB): download+verify 17 ms (~470 MiB/s), install 17.6 ms,
  SHA-256 ~1,725 MiB/s.

## Unreleased - Phase 7: Real-Time Protection — VERIFIED

### Added
- `aegis-realtime` crate — monitors filesystem + process activity and feeds
  events into the verified engines (no engine rewritten):
  - file monitoring via `notify` (create/modify/rename, default user folders,
    500 ms debounce);
  - process monitoring via `sysinfo` (new-process diff: name, exe, command line);
  - event pipeline: scan (`aegis-scan`) → detect (`aegis-detect`) →
    policy → quarantine (`aegis-quarantine`);
  - policies `MonitorOnly` / `NotifyOnly` (default) / `AutoQuarantine`;
  - `RealtimeAlert` (timestamp, path, process, threat_level, score, action,
    reason); `RealtimeEngine`, `RealtimeMonitor`, `Debouncer`.
- DB migration `005_realtime.sql`: `realtime_events`, `realtime_alerts`.
- Orchestrator RTP API: `start_realtime`, `start_realtime_with_paths`,
  `stop_realtime`, `get_realtime_status` — runs in a background thread sharing
  the orchestrator's signature/YARA/vault engines.
- `aegis-realtime` benchmark (`benches/realtime_bench.rs`).
- `REALTIME_PROTECTION.md` — pipeline, policies, performance, limitations.

### Changed
- `aegis-db::apply_migrations` now applies 001 → 005.

### Verified
- `cargo test -p aegis-realtime`: 14/14 pass (4 policy + 3 monitor unit +
  7 integration). `cargo test -p aegis-service`: 11/11 (incl. RTP lifecycle).
- `cargo clippy --workspace --exclude aegis-tauri --all-targets --all-features
  -- -D warnings`: clean.
- Benchmark (release, 2,000 file events): 864 events/s, 1,157 µs/event,
  single-file scan 2,233 µs.

## Unreleased - Phase 6: Service Integration — VERIFIED

### Added
- `aegis-service` is now the **central orchestrator**. Split into a library
  (`AegisOrchestrator`) plus the existing Windows-service binary.
- Per-engine adapters under `service/`: `scan_service`, `detection_service`,
  `quarantine_service`, `windows_service`, `status_service` — thin wrappers over
  the verified engine crates (no engine logic changed).
- Orchestrator IPC contract: `start_scan`, `stop_scan`, `get_scan_status`,
  `list_jobs`, `get_threats`, `quarantine_detection`, `restore_file`,
  `delete_quarantine_item`, `list_quarantine`, `run_windows_scan`,
  `get_service_health`.
- `JobManager` — thread-safe background jobs (queued → running →
  completed/cancelled/failed) with live `ScanProgress`, cooperative
  cancellation, and `job_history` persistence.
- `ServiceHealth` / `ComponentStatus` — scanner / database / rules / quarantine
  status with worst-component `overall`.
- DB migration `004_service.sql`: `service_events`, `job_history`,
  `service_state`.
- `SERVICE_INTEGRATION.md` — orchestrator design, data flow, contract, jobs.

### Changed
- `aegis-db::apply_migrations` now applies 001 → 004.
- `aegis-service` gained a `[lib]` + `[[bin]]`; `runtime` moved into the library;
  the binary host now drives the library.

### Verified
- `cargo test -p aegis-service`: 10/10 pass (4 job-manager + 1 health unit +
  5 integration/lifecycle). Full workspace suite green.
- `cargo clippy --workspace --exclude aegis-tauri --all-targets --all-features
  -- -D warnings`: clean.

## Unreleased - Phase 5: Windows Security Scanner — VERIFIED

### Added
- `aegis-windows` crate — Windows persistence scanner producing
  `aegis_detect::ThreatDetection`s:
  - collectors + pure parsers for startup folders, registry Run/RunOnce
    (`winreg`), scheduled tasks (`schtasks` CSV), services (registry), drivers
    (`driverquery` CSV), browser extensions (Chrome/Edge/Firefox), hosts file;
  - `PersistenceEntry` / `PersistenceKind` model;
  - heuristics: temp-dir executables, scripts in startup, unsigned binaries,
    LOLBin command lines, encoded PowerShell, browser-extension sideloading,
    hosts-file redirects (suspicious-only → no benign flooding);
  - `WindowsScanner::scan_all()` / `analyze_entries()`.
- `WINDOWS_SCANNER.md` — design, surfaces, heuristics, scoring, limitations.

### Changed
- `aegis-detect::ThreatEvidence` gained two additive variants (integration
  requirement for the Windows scanner): `PersistenceMechanism` (+15) and
  `SuspiciousLocation` (+20), with `weight`/`reason`/`label` arms.

### Verified
- `cargo test -p aegis-windows -p aegis-detect`: 39/39 pass (windows 20 unit +
  3 integration; detect 16, unaffected by the new variants).
- `cargo clippy --workspace --exclude aegis-tauri --all-targets --all-features
  -- -D warnings`: clean.

## Unreleased - Phase 4: Quarantine System — VERIFIED

### Added
- `aegis-quarantine` built out into a secure vault (replacing the Phase-1
  boundary skeleton):
  - AES-256-GCM encryption at rest (`crypto::VaultKey`); on-disk format
    `nonce || ciphertext+tag`; per-file random nonce; OS-CSPRNG 256-bit key
    persisted as `vault.key`.
  - `Vault` actions: `quarantine_file`, `quarantine_detection` (consumes
    `aegis_detect::ThreatDetection`), `restore_file`, `delete_file`,
    `get_record`, `list_records`.
  - `QuarantineRecord` (id, original_path, quarantine_path, sha256, threat_level,
    reason, timestamp, size, encrypted, status); randomized `<uuid>.qbin` blobs.
  - Safety: SHA-256 integrity-checked restore, GCM tamper detection,
    path-traversal + relative-path + overwrite guards, no double-restore.
  - Plaintext original shredded (zero-overwrite) after isolation.
  - Audit trail to `audit_log` (actor, action, subject, result).
- DB migration `003_quarantine.sql`: `quarantine_records` (+ reuses `audit_log`).
- Quarantine/restore/encryption benchmark (`benches/quarantine_bench.rs`).
- `QUARANTINE_SYSTEM.md` — vault layout, crypto, safety, performance, limits.

### Changed
- `aegis-db::apply_migrations` now applies 001 → 002 → 003 (idempotent list).

### Verified
- `cargo test -p aegis-quarantine`: 14/14 pass.
- `cargo clippy --workspace --exclude aegis-tauri --all-targets --all-features
  -- -D warnings`: clean.
- Benchmark (release, 1,000 × 64 KiB = 62.5 MiB): quarantine 505 files/s
  (31.5 MiB/s), restore 541 files/s (33.8 MiB/s), AES-256-GCM 1,323.8 MiB/s
  (encryption ≈ 2% of cost; disk I/O dominates).

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
