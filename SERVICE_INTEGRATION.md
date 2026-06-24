# Aegis Service Integration (Phase 6)

`aegis-service` becomes the **central orchestrator**: every security engine is
reachable only through `AegisOrchestrator`. The UI calls the orchestrator over
IPC and never touches the engine crates directly.

## Layout

The crate now ships both a **library** (the orchestrator, testable on any
platform) and the existing **Windows-service binary** (the thin host that drives
it).

```
crates/aegis-service/
├── src/
│   ├── lib.rs                 # AegisOrchestrator + ServiceError (library)
│   ├── orchestrator.rs        # central API — the IPC contract surface
│   ├── jobs.rs                # background JobManager (queue/run/cancel/status)
│   ├── db.rs                  # service_events / job_history / service_state + detection read-back
│   ├── runtime.rs             # service status watch channel
│   ├── service/
│   │   ├── scan_service.rs        # adapter → aegis-scan
│   │   ├── detection_service.rs   # adapter → aegis-detect
│   │   ├── quarantine_service.rs  # adapter → aegis-quarantine
│   │   ├── windows_service.rs     # adapter → aegis-windows
│   │   └── status_service.rs      # ServiceHealth / ComponentStatus
│   ├── main.rs                # Windows-service binary entry
│   └── windows_host.rs        # SCM dispatcher (cfg windows)
```

Engine crates are **not modified** — each adapter is a thin wrapper over the
already-verified API (integration adapters only).

## Data Flow

```text
UI ──IPC──► AegisOrchestrator
                │
   start_scan ──┤── JobManager.create (queued) ─► background thread:
                │       ScanService(aegis-scan) ──► ScanReport
                │       DetectionService(aegis-detect) ──► Vec<ThreatDetection>
                │       persist → detection_results ; job → job_history
                │
   get_threats ─┤── db::list_detections ◄── detection_results
                │
   quarantine ──┤── QuarantineService(aegis-quarantine Vault)
                │
   windows_scan ┤── WindowsSecurityService(aegis-windows) ──► persist
                │
   health ──────┴── ServiceHealth::compute(scanner, db, rules, quarantine)
```

A single SQLite database (`<data_dir>/aegis.db`, migrations 001–004) is the
shared store; the vault lives at `<data_dir>/quarantine/`. Components open their
own connections (WAL) — no shared `Connection` across threads.

## IPC Contract (orchestrator methods)

| Method | Purpose |
|--------|---------|
| `start_scan(roots, mode) -> job_id` | Queue a file scan; runs in the background. |
| `stop_scan(job_id) -> bool` | Cancel a running/queued scan. |
| `get_scan_status(job_id) -> JobState` | Live status + progress. |
| `list_jobs() -> [JobState]` | All jobs. |
| `get_threats() -> [ThreatDetection]` | Detections (highest score first). |
| `quarantine_detection(det, actor) -> QuarantineRecord` | Encrypt + isolate. |
| `restore_file(id, dest, actor)` | Integrity-checked restore. |
| `delete_quarantine_item(id, actor)` | Shred + delete. |
| `list_quarantine() -> [QuarantineRecord]` | Vault contents. |
| `run_windows_scan() -> [ThreatDetection]` | Persistence sweep (persisted). |
| `get_service_health() -> ServiceHealth` | Component statuses. |

## Background Jobs

`JobManager` tracks jobs in a thread-safe map. Each job has a `JobType`
(`file_scan` / `windows_scan`), a `JobStatus`
(`queued → running → completed | cancelled | failed`), live progress
(`ScanProgress`), and timestamps. Scans run on a dedicated thread; cancellation
is cooperative via the scanner's `Arc<AtomicBool>`. A cancel that arrives while a
job is still queued is preserved (running-state transition is guarded against
overwriting a terminal status). Job state is mirrored to `job_history` for audit.

## Service Health

`ServiceHealth` reports `ComponentStatus` (`ok` / `degraded` / `unavailable`)
for the **scanner**, **database**, **rules**, and **quarantine**, plus the
active-job count. `overall` is the worst component. With no YARA rules loaded the
rules component is `degraded` (hash + heuristic detection still operate).

## Database (migration `004_service.sql`)

- `service_events` — lifecycle/orchestration events.
- `job_history` — one row per job (upserted across its lifecycle).
- `service_state` — key/value service state (version, data dir, markers).

`aegis-db::apply_migrations` applies 001 → 004 idempotently.

## Testing

- **Unit** — `JobManager` lifecycle (queue/run/complete/cancel/fail, terminal
  guards), `ServiceHealth` aggregation.
- **Integration / lifecycle** — through `AegisOrchestrator`: scan → detect →
  persist → `get_threats`; quarantine → restore → delete; Windows analysis +
  persistence; health reporting; `stop_scan` cancellation.

`cargo test -p aegis-service`: **10/10 pass**.
`cargo clippy --workspace --exclude aegis-tauri --all-targets --all-features
-- -D warnings`: clean.

## Limitations

- **Per-file progress takes a lock** — fine at current scale; a throttled or
  batched update would reduce contention on very large scans.
- **One vault connection behind a mutex** — quarantine actions serialize; the
  service owns a single vault instance by design.
- **No IPC transport wiring yet** — the orchestrator *is* the contract; binding
  it to the Tauri command bridge / named-pipe server is the next integration
  step. Real-time protection (RTP) will feed `analyze_windows_entries` and the
  scan pipeline once implemented (not part of Phase 6).
