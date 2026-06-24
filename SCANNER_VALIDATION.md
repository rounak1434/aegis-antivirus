# Scanner Validation Report — `aegis-scan`

> **VERIFICATION STATUS: PASSED — VERIFIED**
>
> Phase 2 scanner engine validated on a working MSVC toolchain. Tests pass,
> clippy is clean under `-D warnings`, and the throughput benchmark runs.
> Date of run: 2026-06-24.

## Environment

| Item | Value |
|------|-------|
| rustc | 1.96.0 (ac68faa20 2026-05-25) |
| cargo | 1.96.0 (30a34c682 2026-05-25) |
| Host toolchain | x86_64-pc-windows-msvc (active, default) |
| Linker | MSVC `link.exe` from VS Build Tools (Desktop Development with C++), located by rustc via vcvars/registry — not the Git `link.exe` on PATH |
| OS | Windows 11 Home Single Language 10.0.26200 |
| CPU threads (rayon) | 8 |

`where link` reports `C:\Program Files\Git\usr\bin\link.exe` (Git coreutils) on
the plain PATH; rustc does **not** use it — the MSVC linker is resolved
out-of-band, proven by the successful link of all test/bench binaries below.

## Testing

Command: `cargo test -p aegis-scan`

| Metric | Count |
|--------|-------|
| Test suites | 3 (unit lib, integration, doctests) |
| Tests run | 12 |
| Passed | 12 |
| Failed | 0 |
| Duration | 0.29 s |

Unit tests (`src/lib.rs`): `known_hashes_for_abc`, `scans_files_and_counts`,
`respects_max_depth`, `cancellation_is_reported`, `missing_root_errors`.

Integration tests (`tests/integration.rs`):
`nested_directories_are_scanned`, `hidden_files_excluded_then_included`,
`symlinks_are_handled`, `permission_denied_is_counted_not_fatal`,
`large_file_hashing_and_throughput`, `scan_cancellation_stops_work`,
`progress_callback_reports_metrics`.

## Code Quality

Command: `cargo clippy -p aegis-scan --all-targets --all-features -- -D warnings`
→ **No issues found.**

Workspace-wide: `cargo clippy --workspace --exclude aegis-tauri --all-targets
--all-features -- -D warnings` → **No issues found.**

Warnings fixed during validation:
- `aegis-service::runtime::subscribe_status` — `dead_code` under `-D warnings`.
  Method is the IPC status-subscription API consumed in a later phase; annotated
  `#[allow(dead_code)]` with a rationale comment rather than deleted.

Out of scope (not a Rust warning): `aegis-tauri` build script fails with
`icons/icon.ico not found; required for generating a Windows Resource file`.
This is a packaging asset (Phase 12), unrelated to the scanner; the Tauri crate
is excluded from the workspace clippy gate until the icon set lands.

## Benchmark Results

Command: `cargo run --release --example bench -p aegis-scan`
(synthetic tree: 4000 files × 16 KiB across 16 subdirectories, Full scan,
SHA-256 + MD5 per file)

| Metric | Value |
|--------|-------|
| Files scanned | 4,000 |
| Bytes scanned | 65,536,000 (62.5 MiB) |
| Errors | 0 |
| Duration | 133 ms |
| Throughput (files) | 29,981 files/sec |
| Throughput (bytes) | 468.5 MiB/sec |
| Worker threads | 8 |

Throughput scales with `--release` and core count; debug builds are slower. The
benchmark accepts `<num_files> <file_kib>` args to vary the workload.

### Metrics & ETA

The engine reports these via `ScanProgress` (live, per file) and `ScanReport`
(final aggregate):
- file count — `files_scanned` / `total_files`
- bytes scanned — `bytes_scanned`
- percent complete — `percent`
- duration — `elapsed_ms` (live) / `duration_ms` (final)
- throughput — `files_per_sec`, `bytes_per_sec`
- ETA — `eta_ms = remaining_files / files_per_sec`, recomputed each file from
  the cumulative average rate.

**ETA accuracy:** ETA is a live estimate driven by the running average
throughput. With steady throughput it tracks closely; it self-corrects as the
rate stabilises and converges to 0 at completion (`percent` reaches 100.0,
asserted by `progress_callback_reports_metrics`). Because it uses the cumulative
average, the estimate is least accurate in the first moments of a scan (small
sample) and tightens as more files complete. It does not yet model per-file-size
variance — large files late in a scan can briefly inflate the estimate.

## Limitations & Known Edge Cases

- **Symlinks are recorded but not hashed** — avoids double-counting and
  symlink-loop hashing; `Deep` mode follows links during traversal.
- **Hidden detection is platform-specific** — Windows uses
  `FILE_ATTRIBUTE_HIDDEN`/`SYSTEM`; other platforms fall back to dotfile names.
  The Windows hidden test shells out to `attrib +h`.
- **Permission-denied / locked files** are counted in `errors` and recorded
  per-file (`ScannedFile.error`); never fatal.
- **`ScanReport.files` holds every scanned file in memory** — fine for typical
  trees; a streaming/bounded variant is needed for whole-disk Full scans.
- **`opts_mode` is a best-effort options→mode reverse map** for reporting
  (Custom with default flags reports as Full); informational only.
- **Cancellation is cooperative** — checked per file in the rayon map; an
  in-flight large-file hash finishes before the worker observes cancel.
- **ETA ignores per-file size variance** (see above).
- **Archive recursion (zip/7z) is out of scope for Phase 2.**
