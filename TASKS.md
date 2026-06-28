# Aegis Antivirus Tasks

## Prototype Migration (UI)

### Phase A — Prototype Analysis
- [x] Analyze every prototype page (index, dashboard, scan, threats, quarantine, realtime, settings, widget, architecture).
- [x] Produce design token inventory, UI/component inventory, navigation map, screen map.
- [x] Create `PROTOTYPE_AUDIT.md`.
- [x] Identify that the prior `src/` React scaffold was a generic template diverging from the prototype.

### Phase B — React Conversion
- [x] Install `react-router-dom`.
- [x] Port prototype design tokens into `src/styles.css` + `tailwind.config.ts`.
- [x] Port prototype component classes (.card/.btn/.pill/.stat/.table/.toggle/…) verbatim.
- [x] Create typed `<Icon>` component from `shell.js` icon map.
- [x] Rebuild app shell: `WinBar`, grouped `Sidebar`, `TopBar`, `AppShell` layout route.
- [x] Convert Dashboard screen with full visual parity (first vertical slice).
- [x] Wire `HashRouter` with interim `SectionComingNext` for unconverted routes.
- [x] Verify frontend build (`tsc && vite build`) passes.
- [x] Convert Scan Center screen (`src/features/scan-center`) — live progress simulation hook.
- [x] Convert Threat Center screen + evidence drawer (`src/features/threat-center`).
- [x] Convert Quarantine screen (`src/features/quarantine`) — selection + delete confirm.
- [x] Convert Real-time screen (`src/features/realtime`) — shield toggles + event feed.
- [x] Convert Settings screen (`src/features/settings`) — sub-tabs + signed update flow.
- [ ] Convert Launcher and Architecture pages.

### Phase C — State Layer
- [ ] Zustand stores: app, scan, threat, quarantine, settings, realtime.

### Phase D — Backend Integration
- [ ] Typed Tauri command interfaces: start/stop scan, progress, threats, quarantine, restore, settings.

### Phase E — Production Polish
- [ ] Loading / error / empty states, notifications, keyboard shortcuts, accessibility.

## Phase 1

- [x] Analyze existing project directory.
- [x] Identify current project as static HTML prototype.
- [x] Preserve prototype files under `design-prototype/`.
- [x] Create `ARCHITECTURE.md`.
- [x] Create `ROADMAP.md`.
- [x] Create `TASKS.md`.
- [x] Create `CHANGELOG.md`.
- [x] Add Tauri v2 React TypeScript scaffold.
- [x] Add TailwindCSS configuration.
- [x] Add Zustand state store.
- [x] Add Rust workspace manifest.
- [x] Add `src-tauri` application crate.
- [x] Add `aegis-common` crate.
- [x] Add `aegis-ipc` crate.
- [x] Add `aegis-db` crate.
- [x] Add `aegis-service` crate.
- [x] Add `aegis-update` crate.
- [x] Add `aegis-quarantine` crate.
- [x] Add initial SQLite migration.
- [x] Add UI-to-service IPC bridge contract.
- [x] Document Rust installation requirement if local Cargo is unavailable.

## Phase 2 — File Scanner (`aegis-scan`)  — VERIFIED ✓

> Validated 2026-06-24 on MSVC toolchain: `cargo test -p aegis-scan` 12/12 pass,
> `cargo clippy … -D warnings` clean, benchmark 29,981 files/s · 468.5 MiB/s.
> Evidence in `SCANNER_VALIDATION.md`.

- [x] Add `aegis-scan` crate to the workspace (sha2, md-5, walkdir, rayon).
- [x] Scan mode planner (`ScanOptions::for_mode` — Quick/Full/Deep/Custom presets).
- [x] Recursive filesystem traversal (walkdir) with max-depth.
- [x] Hidden/system file handling (Windows attribute flags, dotfile fallback).
- [x] Symlink handling (follow vs skip; symlinks not hashed).
- [x] Streaming SHA-256 + MD5 hashing (64 KiB buffer).
- [x] File metadata collection (size, modified UTC, hidden, symlink).
- [x] Multi-threaded hashing (rayon) with atomic progress counters.
- [x] Cooperative cancellation + progress callback.
- [x] Scanner unit + integration tests written (known-hash, counts, max-depth,
      nested, hidden, symlink, permission-denied, large-file, cancel, metrics).
- [x] Throughput benchmark example written (`examples/bench.rs`).
- [x] Live metrics: throughput, file/byte counts, ETA, duration tracking.
- [x] **VERIFY**: `cargo test -p aegis-scan` (12/12), `cargo clippy -D warnings`
      (clean), benchmark (29,981 files/s · 468.5 MiB/s) — see SCANNER_VALIDATION.md.
- [ ] Wire `aegis-scan` into `aegis-service` runtime + IPC progress events.

## Phase 3 — Detection Engine — VERIFIED ✓

> Validated 2026-06-24: `cargo test -p aegis-detect` (+signatures/yara) 25/25 pass,
> `cargo clippy --workspace --exclude aegis-tauri … -D warnings` clean, benchmark
> 10k files → 1,646 files/s, 607 µs/file, 800 detections, 27.5 MiB. See
> `DETECTION_ENGINE.md` + `CHANGELOG.md`.

- [x] `aegis-signatures` crate: SHA-256/MD5 SignatureDatabase (SQLite + files +
      in-memory cache, load/reload/contains_*).
- [x] Integrate YARA-X (`aegis-yara`): RuleManager load/validate/compile/cache/scan.
- [x] Heuristics (`aegis-detect`): double extension, suspicious extension,
      entropy/packed-executable, script indicators, PowerShell abuse indicators.
- [x] Threat model: ThreatLevel, ThreatEvidence, ThreatDetection.
- [x] Additive 0–100 risk scoring with explainable evidence + level thresholds.
- [x] DB migration 002: signature_sets, signatures, detection_results, scan_events.
- [x] Unit + integration tests + fixtures; 10k-file benchmark.
- [x] `DETECTION_ENGINE.md`; ARCHITECTURE/TASKS/CHANGELOG updated.
- [ ] Wire `aegis-detect` into `aegis-service` scan pipeline + IPC threat events.

## Quarantine System (`aegis-quarantine`) — VERIFIED ✓

> Validated 2026-06-24: `cargo test -p aegis-quarantine` 14/14 pass,
> `cargo clippy … -D warnings` clean, benchmark 1k×64 KiB → quarantine 505 files/s,
> restore 541 files/s, AES-256-GCM 1,323.8 MiB/s. See `QUARANTINE_SYSTEM.md`.

- [x] Encrypted vault: AES-256-GCM, randomized `<uuid>.qbin`, collision-safe.
- [x] `QuarantineRecord` model + SQLite (`quarantine_records`, migration 003).
- [x] Actions: `quarantine_file`/`quarantine_detection`/`restore_file`/
      `delete_file`/`get_record`/`list_records`.
- [x] Safety: SHA-256 integrity check, path-traversal + overwrite guards,
      status checks (no double-restore).
- [x] Audit trail (`audit_log`): quarantine/restore/delete with actor + result.
- [x] Unit + integration tests; quarantine/restore/encryption benchmark.
- [x] `QUARANTINE_SYSTEM.md`; ARCHITECTURE/TASKS/CHANGELOG updated.
- [ ] Wire `Vault` into `aegis-service` (auto-quarantine high/critical detections).

## Windows Security Scanner (`aegis-windows`) — VERIFIED ✓

> Validated 2026-06-24: `cargo test -p aegis-windows` 23/23 pass (20 unit + 3
> integration), `cargo clippy … -D warnings` clean. See `WINDOWS_SCANNER.md`.

- [x] Startup folder scanner (`startup::scan_dir` + collector).
- [x] Registry Run + RunOnce scanner (HKCU/HKLM via winreg).
- [x] Scheduled task scanner (`schtasks` CSV parser).
- [x] Services scanner (registry, type classification).
- [x] Drivers scanner (`driverquery` CSV parser).
- [x] Browser extension scanner (Chrome/Edge/Firefox).
- [x] Hosts file scanner (`parse_hosts`).
- [x] Heuristics: temp-exe, startup-script, unsigned, LOLBin command lines,
      encoded PowerShell, extension sideload, hosts redirect.
- [x] Findings → `ThreatDetection` (shared model + 2 additive evidence variants).
- [x] Unit + integration tests with mock persistence fixtures.
- [x] `WINDOWS_SCANNER.md`; ARCHITECTURE/TASKS/CHANGELOG updated.
- [x] Wire `WindowsScanner` into `aegis-service` (`run_windows_scan`).

## Service Integration (`aegis-service` orchestrator) — VERIFIED ✓

> Validated 2026-06-24: `cargo test -p aegis-service` 10/10 pass,
> `cargo clippy … -D warnings` clean. See `SERVICE_INTEGRATION.md`.

- [x] `aegis-service` split into library (`AegisOrchestrator`) + service binary.
- [x] Per-engine adapters: scan / detection / quarantine / windows / status.
- [x] IPC contract: start_scan, stop_scan, get_scan_status, get_threats,
      quarantine_detection, restore_file, delete_quarantine_item,
      run_windows_scan, get_service_health.
- [x] Background `JobManager`: queued/running/cancel/status, job history.
- [x] Service health: scanner / database / rules / quarantine + overall.
- [x] DB migration 004: service_events, job_history, service_state.
- [x] Unit + integration + lifecycle + job-manager tests.
- [x] `SERVICE_INTEGRATION.md`; ARCHITECTURE/TASKS/CHANGELOG updated.
- [x] Bind orchestrator to RTP (`start_realtime` / `stop_realtime` / `get_realtime_status`).
- [ ] Bind orchestrator to the Tauri command bridge / named-pipe IPC server.

## CI/CD & Quality Gates (Phase 10) — VERIFIED ✓

> Validated 2026-06-24: all 9 workflow/template YAML files parse (`npx js-yaml`);
> gate commands verified locally (`cargo fmt --check` clean after `cargo fmt`,
> clippy clean, 118 tests, npm build + 10 vitest). See `CI_CD.md`.

- [x] Workflows: `ci.yml` (orchestrator), `rust.yml`, `frontend.yml`,
      `security.yml`, `release.yml` (draft).
- [x] Rust gate: fmt-check, clippy `-D warnings`, test, build; cargo cache.
- [x] Frontend gate: `npm ci` + build + test; npm cache.
- [x] Security: cargo-deny (`deny.toml`), cargo-audit, dependency review, gitleaks.
- [x] Quality `gate` job requires rust+frontend+security to pass.
- [x] `.github/dependabot.yml` (cargo, npm, github-actions, weekly).
- [x] `.github/CODEOWNERS`; improved PR template (required gate + tests + docs).
- [x] README badges (build/tests/release/license); `cargo fmt` applied workspace-wide.
- [x] `CI_CD.md`; README/CONTRIBUTING/TASKS/CHANGELOG updated.

## UI ↔ Service Integration (Phase 9) — VERIFIED ✓

> Validated 2026-06-24: `npm run build` clean, `npm test` 10/10 (vitest),
> `cargo build -p aegis-tauri` compiles, `cargo test --workspace --exclude
> aegis-tauri` 118 pass, clippy clean. See `UI_SERVICE_INTEGRATION.md`.

- [x] Tauri command bridge (21 commands) over managed `AegisOrchestrator`.
- [x] App icons generated (`src-tauri/icons/*`) — `aegis-tauri` now compiles.
- [x] Typed IPC layer (`src/lib/api.ts`, single `invoke` site); DTO types.
- [x] Zustand stores: scan, threat, quarantine, update, realtime, settings (+ health).
- [x] All screens live (Dashboard, Scan, Threats, Quarantine, Real-time, Updates, Settings).
- [x] Mocks removed (useScanSimulation, threat/dashboard seed arrays, securityStore).
- [x] Error/loading/empty states; settings persisted via service (`get/save_settings`).
- [x] Tests: IPC wrappers, stores, Dashboard component (vitest).
- [x] `UI_SERVICE_INTEGRATION.md`; ARCHITECTURE/TASKS/CHANGELOG updated.
- [ ] Run the orchestrator as a separate service process over named-pipe IPC.

## Secure Update System (`aegis-update`) — VERIFIED ✓

> Validated 2026-06-24: `cargo test -p aegis-update` 17/17 pass (+service 12/12),
> `cargo clippy --workspace --exclude aegis-tauri … -D warnings` clean, benchmark
> 8 MiB → 470 MiB/s download+verify, 1725 MiB/s sha256. See `UPDATE_SYSTEM.md`.

- [x] `UpdateManifest` (version, published_at, sha256, signature, url, size,
      component, minimum_app_version) + signed canonical message.
- [x] Crypto verify: SHA-256 + Ed25519 (pinned key), anti-rollback, min-app.
- [x] Download engine (`reqwest`): resume, timeout, retries, gzip; `Fetcher`
      trait + offline `LocalFetcher`.
- [x] Storage: updates/installed/backup + manifest.json; backup-based rollback.
- [x] Scheduler: Manual / Daily / Weekly / Startup (`is_due`).
- [x] DB migration 006 (installed_components) + update_history reuse.
- [x] Service integration: check/download/install/rollback/status + engine reload.
- [x] Unit + integration (tamper/rollback/anti-rollback/min-app/signature) + bench.
- [x] `UPDATE_SYSTEM.md`; ARCHITECTURE/TASKS/CHANGELOG updated.

## Real-Time Protection (`aegis-realtime`) — VERIFIED ✓

> Validated 2026-06-24: `cargo test -p aegis-realtime` 14/14 pass,
> `cargo clippy … -D warnings` clean, benchmark 2k events → 864 events/s,
> 1157 µs/event. See `REALTIME_PROTECTION.md`.

- [x] File monitoring (`notify`): create/modify/rename, default folders, debounce.
- [x] Process monitoring (`sysinfo`): new processes, cmdline, exe path.
- [x] Event pipeline reusing scan → detect → quarantine (no engine rewrite).
- [x] Policies: MonitorOnly / NotifyOnly (default) / AutoQuarantine.
- [x] `RealtimeAlert` + DB migration 005 (realtime_events, realtime_alerts).
- [x] Service integration: start/stop/status, background thread.
- [x] Unit + integration + policy tests with mock events; benchmark.
- [x] `REALTIME_PROTECTION.md`; ARCHITECTURE/TASKS/CHANGELOG updated.

## Phase 5 (legacy roadmap — superseded by the sections above)

- [x] Implement file monitoring.
- [x] Implement process launch monitoring.
- [x] Add alert event pipeline.
- [ ] Add scheduled scan runner.

## Phase 6

- [ ] Implement encrypted quarantine vault.
- [ ] Implement restore.
- [ ] Implement permanent delete.
- [ ] Implement report generation.
- [ ] Implement report export.

## Phase 7

- [ ] Add installer packaging.
- [ ] Add signing pipeline.
- [ ] Add secure update rollout process.
- [ ] Add release checklist.


