# Hardening Report (Phase 13)

What was validated and what changed. Every change is justified by a measurable
finding; no engine/detection logic was modified (none needed it).

## Changes made

| Change | Justification |
|--------|---------------|
| `deny.toml` + `.cargo/audit.toml` advisory ignores (19 IDs, each commented) | `cargo audit` flagged 1 transitive vuln (`rsa`, unreachable) + 18 transitive warnings (GTK Linux-only / unmaintained). Documented + accepted so the CI security gate is green without masking a real risk. |

No source-code changes — the review found **no exploitable defect or measurable
regression** to fix.

## Memory & resource validation

| Check | Result |
|-------|--------|
| Memory growth under load | Bounded — detect peak **27.4 MiB** @ 10k files; content reads capped at 1 MiB/file. Scan holds `ScanReport` in memory (noted limitation for whole-disk). |
| Memory leaks | No raw allocation/`unsafe` in library code; all state is RAII (`Arc`/`Box`/`Vec`). Connections are per-use and dropped. |
| Thread leaks | RTP `RealtimeMonitor::stop()` sets the run flag false **and `join()`s** the worker; `Drop` calls `stop()` (no orphaned threads). Scan uses rayon (scoped). Service scan jobs are bounded `std::thread`s that finalize job state on exit. |
| Deadlocks | Locks are short-lived, never nested across subsystems; the scan progress callback takes the JobManager lock only (no lock held across a blocking call). No lock-ordering cycles found. |
| Resource cleanup | Files via `std::fs` (closed on drop); SQLite connections opened per operation (WAL); temp files via `tempfile` (auto-removed). Vault swaps are copy-`.tmp`+rename. |
| Background shutdown | RTP stop joins; the orchestrator drops the monitor on `stop_realtime`. Scan cancellation is cooperative (`Arc<AtomicBool>`), checked per file. |

## Cancellation latency

Scan cancellation is checked per-file in the rayon map: an in-flight large-file
hash finishes (bounded by file size), then the worker observes cancel. For
typical files this is sub-millisecond; the service `stop_scan` returns
immediately and the job transitions to `cancelled` (guarded against a late
`mark_running` overwrite — a bug fixed in Phase 6).

## Static analysis (gate)

`cargo fmt --check` clean · `cargo clippy -D warnings` clean (0 issues) ·
`cargo audit` green (with documented ignores) · `cargo deny` configured. 0
library `unsafe` blocks. See SECURITY_REVIEW.

## Validation summary

| Metric | Value |
|--------|-------|
| Crash count | 0 |
| Panic count | 0 |
| Library `unsafe` blocks | 0 |
| Untrusted-input panics | 0 |
| Clippy issues | 0 |
| Exploitable findings | 0 |
| Tests passing | 118 (Rust) + 10 (vitest) |
