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

## Phase 4

- [ ] Implement startup folder scanner.
- [ ] Implement registry Run key scanner.
- [ ] Implement scheduled task scanner.
- [ ] Implement services scanner.
- [ ] Implement drivers scanner.
- [ ] Implement browser extension scanner.
- [ ] Implement hosts file scanner.

## Phase 5

- [ ] Implement file monitoring.
- [ ] Implement process launch monitoring.
- [ ] Add alert event pipeline.
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


