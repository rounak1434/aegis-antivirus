# Known Limitations

Honest, consolidated list as of Phase 13. None are exploitable defects; they are
scope boundaries and future-hardening items.

## Architecture / deployment

- **In-process orchestrator** — the UI hosts `AegisOrchestrator` directly rather
  than talking to a separate elevated `AegisService` process over a named pipe.
  Signed-IPC client enforcement is therefore not yet active.
- **Service runtime is a skeleton** — the installed `aegis-service.exe` runs the
  lifecycle/runtime; wiring it to the orchestrator + scheduled scans is pending.
- **`aegis-tauri` excluded from the PR gate** — built only in the release
  workflow (full WebView2/WiX/NSIS), not on every PR.

## Detection / scanning

- **Detection is single-threaded** (per-file YARA `Scanner`) — ~1.8k files/s;
  parallelizing over `ScanReport.files` is the main throughput win.
- **`ScanReport` is fully in memory** — fine for typical trees; a streaming
  variant is needed for whole-disk Full scans of millions of files.
- **No archive recursion** (zip/7z) or Office-macro/WMI/COM-hijack introspection.
- **Content heuristics cap at 1 MiB/file** — abuse strings past that offset are
  missed; entropy is whole-head, not sectioned.
- **TOCTOU window** — scan/restore open-then-read; integrity is checked on the
  bytes actually read. Acceptable for a user-mode scanner; a minifilter would
  close it.
- **No on-write blocking** — RTP acts *after* a write lands (user-mode); inline
  blocking needs a kernel minifilter driver.

## Real-time protection

- **Process monitoring is polling** (sysinfo, ~1 s) — a very short-lived process
  between polls can be missed; ETW/WMI would be exact.
- **Scan-on-event reads the whole file** — large files cost more; no async queue
  for bursts yet.

## Updates

- **Pinned key is config-provided** — no production key in the repo; no key
  rotation/revocation flow; no delta updates; feed-index transport out of scope.

## Supply chain (accepted, transitive)

- `rsa` Marvin advisory (RUSTSEC-2023-0071) is present via **yara-x** but
  **unreachable** — Aegis does no RSA (Ed25519 + AES-256-GCM). Ignored with
  justification.
- GTK3 binding advisories (gtk/atk/gdk/glib) are **Linux-only** (Tauri) and not
  in the Windows build. Ignored.
- Several **unmaintained** transitive crates (bincode, unic-ucd, …) via yara-x /
  tauri — informational only. See SECURITY_REVIEW.

## Not measured / not run in this environment

- Cold-process + UI (WebView) startup time, sustained CPU%, raw disk-I/O counters
  — need a packaged build + profiler on a target machine.
- Full `tauri build` of installers, MSI/NSIS install, service registration, and
  Authenticode signing — need the bundler toolchain, a cert, and Administrator.
- Multi-GB archive and true millions-of-tiny-files disk runs, and the
  Windows 10 / Windows 11 / fresh-VM / upgrade compatibility matrix — need
  dedicated VMs. Methods are documented in PERFORMANCE_REPORT / DEPLOYMENT.

## Packaging

- Code signing is **optional** (secret-gated) — releases are unsigned unless a
  certificate is configured.
