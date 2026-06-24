# Aegis Antivirus Architecture

Aegis Antivirus is a Windows-first desktop security application. The desktop app is the user interface only. Privileged protection work is owned by a dedicated Windows background service named `AegisService`.

## Goals

- Provide a native Windows 10/11 antivirus experience using Tauri v2, React, TypeScript, TailwindCSS, and Zustand.
- Keep all privileged security operations in `AegisService`, not in the UI process.
- Use Rust for the backend foundation, service runtime, IPC contracts, database access, update flow, and future scanner engines.
- Keep engine crates platform-aware so Linux and macOS support can be added behind traits later.
- Avoid placeholder implementation. Phase 1 establishes boundaries, contracts, schemas, and build structure only.

## Non-Goals For Phase 1

- No malware detection implementation.
- No file scanning implementation.
- No real-time minifilter driver.
- No YARA-X rule execution.
- No quarantine encryption implementation.
- No production updater transport.

Those features begin in later phases after the architecture is stable.

## Top-Level System

```text
+------------------------------+
| Aegis Desktop UI             |
| Tauri v2 + React + Zustand   |
| User-mode, non-privileged    |
+---------------+--------------+
                |
                | Secure IPC boundary
                v
+---------------+--------------+
| AegisService                 |
| Windows service, Rust        |
| Owns privileged operations   |
+---------------+--------------+
                |
                v
+---------------+--------------+
| Engine Crates                |
| scan, yara, heuristics,      |
| realtime, quarantine, update |
+---------------+--------------+
                |
                v
+---------------+--------------+
| Database Layer               |
| SQLite migrations + repos    |
+------------------------------+
```

## 1. Desktop UI

The desktop UI is a Tauri v2 application with React, TypeScript, TailwindCSS, and Zustand.

Responsibilities:

- Render dashboard, scan center, threat center, quarantine, reports, settings, and tray UI.
- Display service status and protection posture.
- Send typed commands to the service through the IPC client.
- Subscribe to status and notification events.
- Never perform privileged scanning, quarantine, monitoring, or updates directly.

The UI process must be safe to restart without interrupting real-time protection. `AegisService` is the source of truth.

## 2. Windows Service

The background service is named `AegisService`.

Responsibilities:

- Real-time protection.
- File monitoring.
- Process monitoring.
- Scheduled scans.
- Quarantine operations.
- Signature and rule updates.
- Database writes for scan history, detections, notifications, and audit logs.
- Secure IPC server.

The service is designed as a long-running Tokio runtime with explicit task supervision. Phase 1 includes the service crate and lifecycle skeleton only.

## 3. Engine Crates

Engine crates are Rust library crates with focused responsibilities.

Initial crates:

- `aegis-common`: shared domain types and version constants.
- `aegis-ipc`: IPC request/response/event contracts.
- `aegis-db`: SQLite migration and connection primitives.
- `aegis-service`: Windows service entrypoint and service runtime skeleton.
- `aegis-update`: signed update metadata types and update planning boundary.
- `aegis-quarantine`: quarantine command/result types and vault boundary.

Implemented engine crates:

- `aegis-scan`: filesystem traversal + scan jobs. Owns `ScanOptions`
  (Quick/Full/Deep/Custom presets controlling depth, hidden/system inclusion,
  and symlink following), streaming SHA-256 + MD5 hashing, `FileMetadata`
  collection, multi-threaded hashing via rayon with atomic progress counters,
  a progress callback, and cooperative cancellation. Filesystem-only and
  platform-aware (Windows hidden/system attribute detection behind `cfg`).
- `aegis-signatures` (Phase 3): SHA-256/MD5 signature database — SQLite + local
  file sources + in-memory cache, with `reload()`.
- `aegis-yara` (Phase 3): YARA-X rule manager — load/validate/compile/cache/scan.
- `aegis-detect` (Phase 3): detection engine above scanner output. Threat model
  (`ThreatEvidence`, `ThreatDetection`), heuristics (double/suspicious
  extension, entropy/packing, script + PowerShell indicators), additive 0–100
  risk scoring with explainable evidence, and persistence to `detection_results`
  / `scan_events`. See `DETECTION_ENGINE.md`.
- `aegis-quarantine` (Phase 4): AES-256-GCM encrypted vault. `Vault` with
  `quarantine_file`/`quarantine_detection`/`restore_file`/`delete_file`/
  `get_record`/`list_records`. Randomized `<uuid>.qbin` blobs, plaintext shred,
  SHA-256 integrity-checked restore, path-traversal/overwrite guards, and full
  audit trail. Persists to `quarantine_records` + `audit_log`. See
  `QUARANTINE_SYSTEM.md`.
- `aegis-windows` (Phase 5): Windows persistence scanner. Collectors for startup
  folders, registry Run/RunOnce, scheduled tasks, services, drivers, browser
  extensions (Chrome/Edge/Firefox), and the hosts file; heuristics (temp-exe,
  startup-script, unsigned, LOLBin command lines, encoded PowerShell, extension
  sideload, hosts redirect) → `ThreatDetection`. Pure parsers + best-effort
  `cfg(windows)` collectors. See `WINDOWS_SCANNER.md`.

Future crates:

- `aegis-scan` (archive recursion limits — extends the above).
- `aegis-realtime`: file/process monitoring and alert generation.
- `aegis-reporting`: report rendering and export.

## 4. Database Layer

SQLite is the local state store. The database layer owns migrations and exposes typed repositories.

Required tables:

- `settings`
- `scan_history`
- `threats`
- `quarantine`
- `yara_rules`
- `detections`
- `notifications`

Phase 1 also adds operational tables:

- `schema_migrations`
- `audit_log`
- `update_history`

Database rules:

- All schema changes must be migrations.
- Service writes are authoritative.
- The UI reads through service IPC, not by opening the database directly.
- Quarantine records must be auditable.

## 5. Update System

The update system is owned by `AegisService`.

Responsibilities:

- Track installed signature/rule bundle metadata.
- Validate signed update manifests before accepting updates.
- Apply updates atomically.
- Record update history in SQLite.

Phase 1 defines update metadata and boundaries only. Transport, signature verification, rollback, and delta updates are later work.

## 6. IPC Boundary

The IPC boundary separates unprivileged UI from privileged service operations.

Phase 1 IPC decisions:

- Shared request/response/event contracts live in `aegis-ipc`.
- The Tauri app calls a local service client from `src-tauri`.
- The service will expose a secure Windows IPC endpoint.
- Commands are typed and explicit.
- UI commands must not accept arbitrary privileged actions.

Initial commands:

- `GetServiceStatus`
- `StartScan`
- `CancelScan`
- `ListThreats`
- `ListQuarantine`
- `QuarantineRestore`
- `QuarantineDelete`
- `CheckForUpdates`
- `GetSettings`
- `UpdateSettings`

Security requirements:

- Authenticate local client identity where possible.
- Validate every path and object identifier server-side.
- Deny unknown commands by default.
- Log security-sensitive actions.

## Privilege Model

```text
Normal user session
  Aegis UI
    - renders state
    - requests service actions
    - receives notifications

Elevated service context
  AegisService
    - monitors files/processes
    - schedules scans
    - quarantines/restores files
    - updates signatures
    - owns SQLite writes
```

## Frontend Implementation (Prototype Migration)

The desktop UI is a faithful React conversion of the design prototype under
`design-prototype/`, which is the **product specification** for all UI/UX. The
prototype's visual system (Anthropic warm-dark palette, terracotta accent,
serif display type) is preserved pixel-for-pixel.

Layering:

- `src/styles.css` — design tokens (`:root` CSS variables) + component classes
  (`.card`, `.btn`, `.pill`, `.stat`, `.table`, `.toggle`, …) ported verbatim
  from `design-prototype/css/app.css`. Single source of truth for parity.
- `tailwind.config.ts` — mirrors the same tokens as Tailwind utilities.
- `src/components/Icon.tsx` — typed icon set from the prototype's `shell.js`.
- `src/components/shell/` — `WinBar`, `Sidebar`, `TopBar`, and the `nav.ts`
  model (groups + per-route metadata), replacing `shell.js`'s DOM injection.
- `src/components/AppShell.tsx` — layout route (winbar + sidebar + topbar +
  `<Outlet>`), driven by `react-router-dom` (HashRouter for file/tauri origin).
- `src/features/<screen>/` — one folder per prototype screen. Each holds the
  component, co-located screen CSS, and typed seed data. Seed data is swapped
  for Zustand stores (Phase C) and live IPC data (Phase D) without touching
  the visual layer.

Conversion is incremental and screen-by-screen; the Dashboard is the first
fully converted screen. Unconverted routes render an interim, fully-functional
`SectionComingNext` view inside the production shell.

## Phase 1 Exit Criteria

- Documentation exists and reflects the service-owned architecture.
- Native Tauri/React/TypeScript project scaffold exists.
- Rust workspace exists.
- SQLite migration system exists.
- `AegisService` crate exists with a real lifecycle skeleton.
- IPC contracts exist and are consumed by both UI-side Rust and service-side Rust.
- No detection engine code has been generated.
