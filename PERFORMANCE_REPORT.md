# Performance Report (Phase 13)

Measured on the development workstation (Windows 11, 8 logical cores, MSVC,
release builds). Numbers from the in-repo benchmarks — reproduce with the
commands shown.

## Scan throughput & load test (`aegis-scan`)

`cargo run --release --example bench -p aegis-scan -- <files> <kib>`

| Files | File size | Duration | Files/sec | MiB/sec | Threads |
|------:|----------:|---------:|----------:|--------:|--------:|
| 100 | 16 KiB | 3 ms | 26,344 | 411 | 8 |
| 10,000 | 16 KiB | 212 ms | 46,994 | 734 | 8 |
| 100,000 | 4 KiB | 1,413 ms | **70,759** | 276 | 8 |

Throughput **rises with corpus size** (fixed startup amortized) and scales with
core count (rayon). 100k files (SHA-256 + MD5 each) in ~1.4 s.

## Detection (`aegis-detect`)

`cargo bench -p aegis-detect --bench detect_bench`

| Metric | Value |
|--------|-------|
| Files | 10,000 |
| Detect phase | 5.46 s |
| Throughput | 1,830 files/sec |
| Latency | 546 µs/file |
| Detections | 800 |
| **Peak working set** | **27.4 MiB** |

Single-threaded (per-file YARA `Scanner`) — the throughput headroom item (see
KNOWN_LIMITATIONS).

## Quarantine latency (`aegis-quarantine`)

| Op | Throughput | Bandwidth |
|----|-----------|-----------|
| Quarantine (encrypt + isolate) | 973 files/sec | 60.8 MiB/s |
| Restore (decrypt + integrity) | 1,004 files/sec | 62.7 MiB/s |
| AES-256-GCM (crypto only) | — | 1,379 MiB/s |

Crypto is not the bottleneck; per-file I/O + DB writes dominate.

## Real-time protection (`aegis-realtime`)

| Metric | Value |
|--------|-------|
| Throughput | 864 events/sec |
| Event latency (avg) | 1,157 µs |
| Single-file scan (cold) | 2,233 µs |

Per-event cost ≈ single-file scan + a SQLite write; debounce keeps real volume low.

## Update verification (`aegis-update`)

| Metric (8 MiB payload) | Value |
|------------------------|-------|
| Download + verify (SHA-256 + Ed25519) | 17 ms (~470 MiB/s) |
| Install (re-verify + swap + persist) | 17.6 ms |
| SHA-256 throughput | ~1,725 MiB/s |

Ed25519 verification is negligible vs. hashing.

## Memory & stability

- **Peak working set** (detect, 10k files): **27.4 MiB** — bounded; content
  heuristics cap reads at 1 MiB/file.
- **Crash count: 0 · panic count: 0** across all benchmark + test runs
  (118 tests + 5 benches, repeated). No panic on any exercised path.
- Background threads (RTP monitor, scan jobs) shut down cleanly (join on stop /
  Drop) — see HARDENING_REPORT.

## Not instrumented here

Cold-process startup, UI (WebView) startup, sustained CPU%, and raw disk-I/O
counters require a packaged build + profiler on a target machine (no installer
build/admin in this environment). Method: `Measure-Command` for startup,
`Get-Counter '\Process(aegis-*)\% Processor Time'` for CPU, and the bundled
benchmarks for engine throughput. Tracked in KNOWN_LIMITATIONS.
