# Aegis Real-Time Protection (Phase 7)

`aegis-realtime` watches filesystem and process activity and feeds events into
the **already-verified** engines. No engine is rewritten — the pipeline calls
`aegis-scan`, `aegis-detect`, and `aegis-quarantine` directly.

## Pipeline

```text
File event ──► scan (aegis-scan, single file) ──► detect (aegis-detect)
                                                       │
                                                  ThreatDetection
                                                       │
                                              policy.decide(mode, level)
                                                       │
                              ┌────────────────────────┼───────────────┐
                              ▼                         ▼               ▼
                          Monitored                 Notified       Quarantined
                          (log only)               (alert)     (aegis-quarantine)

Process event ─► analyze cmdline + exe (aegis-detect heuristics) ─► detect ─► policy
```

Each event is persisted to `realtime_events`; each detection raises a
`RealtimeAlert` persisted to `realtime_alerts`.

## File Monitoring

- Library: [`notify`] (`RecommendedWatcher`, recursive).
- Watched folders (default): **Downloads, Desktop, Documents, Temp, user
  profile** (`%USERPROFILE%`, `%TEMP%`) — existing dirs only.
- Event kinds → `Create`, `Modify`, `Rename` (move ≈ rename). Deletions are
  ignored (nothing to scan).
- **Debounced**: repeat events for the same path within 500 ms are dropped.

## Process Monitoring

- Library: [`sysinfo`] — the poller refreshes processes (~1 s) and diffs PIDs to
  detect **new** processes, capturing name, exe path, and command line.
- Heuristics (reused from `aegis-detect`): temp-dir launch, double extension,
  suspicious extension, and PowerShell-abuse / `-enc` command lines.

## Policies

| Mode | Behavior |
|------|----------|
| `MonitorOnly` | Observe + log; never act (`Monitored`). |
| `NotifyOnly` | Alert on detections, take no action (**default**). |
| `AutoQuarantine` | Quarantine **High/Critical**, notify otherwise. |

## Alerts

`RealtimeAlert { id, timestamp, path, process, threat_level, score, action,
reason }`. `action ∈ { monitored, notified, quarantined, quarantine_failed }`.
The `reason` is the joined evidence labels — alerts stay explainable.

## Database (migration `005_realtime.sql`)

- `realtime_events` — every monitored file/process event.
- `realtime_alerts` — alerts with level, score, action, and detail.

`aegis-db::apply_migrations` applies 001 → 005 idempotently.

## Service Integration

`AegisOrchestrator` owns RTP and runs it in the background:

| Method | Purpose |
|--------|---------|
| `start_realtime(mode)` | Start watching the default folders. |
| `start_realtime_with_paths(mode, paths)` | Start with explicit folders (config/tests). |
| `stop_realtime()` | Stop the monitor threads. |
| `get_realtime_status()` | `running`, `mode`, watched paths, events/alerts counts. |

The `RealtimeEngine` shares the orchestrator's signature DB, YARA rules, and
quarantine vault (the same `Arc`s) — RTP and on-demand scans use one set of
engines.

## Performance

Benchmark (`cargo bench -p aegis-realtime --bench realtime_bench`, release,
2,000 synthetic file events, ~10 % malicious):

| Metric | Value |
|--------|-------|
| Events | 2,000 |
| Alerts raised | 201 |
| Total | 2,314 ms |
| Throughput | 864 events/sec |
| Event latency (avg) | 1,157 µs |
| Single-file scan | 2,233 µs (cold) |

Per-event cost is dominated by the single-file scan (hash) + a SQLite write.
The debounce + new-process diffing keep real-world event volume far below this.

## Testing

- **Unit** — policy (all modes × levels), debouncer (window + per-path),
  notify event-kind mapping.
- **Integration / mock events** — malicious file → notify alert (file kept);
  clean file → no alert; known-bad hash + AutoQuarantine → file isolated;
  Medium/Low under AutoQuarantine → notify only; temp + encoded-PowerShell
  process → alert; benign process → none; MonitorOnly logs without acting.
- **Service** — RTP start/stop/status lifecycle through the orchestrator.

`cargo test -p aegis-realtime`: **14/14 pass**. `cargo test -p aegis-service`:
11/11. `cargo clippy --workspace --exclude aegis-tauri -- -D warnings`: clean.

## Limitations

- **Polling, not eventing, for processes** — sysinfo diff every ~1 s can miss a
  very short-lived process between polls. A kernel ETW/WMI consumer would be
  exact (future work).
- **Scan-on-event reads the whole file** — large files cost more; a size cap or
  async queue would smooth bursts.
- **No on-write blocking** — detection/quarantine happen *after* the write
  lands (user-mode). Inline blocking needs a minifilter driver (out of scope).
- **Per-event SQLite open** — fine at current rates; a pooled connection would
  cut latency under heavy load.
