# UI ↔ Service Integration (Phase 9)

The desktop UI is now **live**: every screen reads/writes real data from
`AegisService` through a typed IPC layer. No mock/simulated data remains in the
app. No engine was modified — the UI reaches engines only through the
orchestrator.

## Architecture

```text
React + TypeScript + Zustand
        │  (stores call api.*, never invoke directly)
        ▼
Typed IPC layer  (src/lib/api.ts → src/lib/ipc.ts, the only invoke() site)
        │  Tauri v2 invoke
        ▼
Tauri command bridge  (src-tauri/src/commands) — AppState { AegisOrchestrator }
        │
        ▼
AegisService orchestrator  →  existing engines (scan / detect / quarantine /
                               windows / realtime / update)
```

The Tauri app constructs **one** `AegisOrchestrator` (DB + vault under
`%LOCALAPPDATA%\Aegis`) and holds it in managed state; every command delegates
to it.

## IPC Layer

- `src/lib/ipc.ts` — the single `invoke` call site. `call<T>()` normalizes all
  failures to `ServiceError`; `inTauri()` detects the shell vs. browser preview.
- `src/lib/api.ts` — typed wrappers grouped by domain. Components/stores import
  `api`, never `invoke`:

| Domain | Methods |
|--------|---------|
| `api.health` | `get` |
| `api.scan` | `start`, `stop`, `status`, `jobs` |
| `api.threats` | `list`, `quarantine` |
| `api.quarantine` | `list`, `restore`, `delete` |
| `api.windows` | `scan` |
| `api.realtime` | `start`, `stop`, `status` |
| `api.updates` | `check`, `download`, `install`, `rollback`, `status` |
| `api.settings` | `load`, `save` |

DTO types (`src/types/ipc.ts`) mirror the Rust serde output (snake_case).

## Rust Command Bridge

`src-tauri/src/commands/mod.rs` exposes 21 `#[tauri::command]`s, each a thin
delegate to an orchestrator method (e.g. `start_scan`, `list_threats`,
`quarantine_detection`, `run_windows_scan`, `start_realtime`, `check_updates`,
`load_settings`). String→enum args (`ScanMode`, `ProtectionMode`,
`UpdateComponent`) are parsed via serde. The settings API was added to the
orchestrator (`get_settings`/`save_settings`, backed by the `settings` table) —
the only new service-layer code; no engine changed.

## Zustand Stores

| Store | Backs |
|-------|-------|
| `healthStore` | service health (dashboard) |
| `scanStore` | start/stop/poll scans + job list |
| `threatStore` | threat list + quarantine action |
| `quarantineStore` | vault records, restore, delete |
| `realtimeStore` | RTP status, start/stop, policy mode |
| `updateStore` | installed components, check, install, rollback |
| `settingsStore` | load/edit/save persisted settings |

Each store carries `loading`/`error` and funnels failures from `ServiceError`
into the UI's error banners.

## Screens (all live, mocks removed)

| Screen | Data source |
|--------|-------------|
| **Dashboard** | health + RTP status + threats + quarantine count + installed signatures |
| **Scan Center** | `scanStore` — Quick/Full/Deep/Custom start, **Cancel**, live progress %, ETA, throughput, threat count (polled `JobState`) |
| **Threat Center** | `threatStore` — detections, evidence drawer, risk score, filter (by level), search (path), sort (score) |
| **Quarantine** | `quarantineStore` — live records, restore, delete, metadata panel |
| **Real-time** | `realtimeStore` — engine state, start/stop, policy mode, watched folders, event/alert counters |
| **Updates** | `updateStore` — installed components, check, install progress, rollback |
| **Settings** | `settingsStore` — all values loaded from and saved to the service (no frontend-only settings) |

## Error Handling

`src/components/States.tsx` renders `ErrorBanner` / `Loading` / `Empty`.
`ErrorBanner` classifies raw service messages into: engine unavailable, database
unavailable, not-configured, network/download failure, scan failure — and shows
the raw message + a retry. In a plain browser preview (`!inTauri()`) it explains
the desktop-only nature.

## Testing

- **IPC** (`src/lib/api.test.ts`) — mocks `@tauri-apps/api/core` invoke; asserts
  command names + argument shapes + `ServiceError` normalization.
- **Stores** (`src/stores/stores.test.ts`) — mocks `api`; asserts load/start
  flows and error capture.
- **Component** (`Dashboard.test.tsx`) — renders the Dashboard with a mocked
  service and asserts live (not mock) content appears.

`npm test` (vitest): **10/10 pass**. `npm run build` (tsc + vite): clean.
`cargo build -p aegis-tauri`: compiles (247 crates). `cargo test --workspace
--exclude aegis-tauri`: 118 pass. `cargo clippy … -D warnings`: clean.

## Limitations

- **Pause/Resume not supported** — the scanner runs to completion (rayon); only
  **Cancel** is real. Adding pause would require modifying the verified scanner
  engine, which Phase 9 forbids.
- **Update feed transport** — the Update page shows installed components and can
  install/rollback, but discovering available updates needs a configured signed
  feed + pinned key (`init_updates`); without it, "check" reports none.
- **In-process orchestrator** — the UI hosts the orchestrator directly rather
  than talking to a separate `AegisService` process over a named pipe; the
  service boundary (and signed-IPC enforcement) is the next hardening step.
- **GUI not auto-tested** — verification is compile + typecheck + unit tests;
  the live window is exercised manually via `npm run tauri dev`.
