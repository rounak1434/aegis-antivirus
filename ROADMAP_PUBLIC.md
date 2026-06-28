# Aegis Antivirus — Public Roadmap

A high-level view of where Aegis is and where it's going. For engineering detail
see [`ROADMAP.md`](ROADMAP.md) and [`TASKS.md`](TASKS.md).

## ✅ Completed & Verified

Each milestone below ships with tests, `clippy -D warnings` clean, and (where
relevant) benchmarks. See the per-phase docs for evidence.

| Milestone | Crate(s) | Docs |
|-----------|----------|------|
| **Foundation** — workspace, SQLite + migrations, IPC contracts, service skeleton | `aegis-common`, `aegis-ipc`, `aegis-db` | [`ARCHITECTURE.md`](ARCHITECTURE.md) |
| **Scanner** — multi-threaded traversal, SHA-256/MD5, metrics, cancellation | `aegis-scan` | [`SCANNER_VALIDATION.md`](SCANNER_VALIDATION.md) |
| **Detection** — hash + YARA-X + heuristics, explainable 0–100 scoring | `aegis-detect`, `aegis-yara`, `aegis-signatures` | [`DETECTION_ENGINE.md`](DETECTION_ENGINE.md) |
| **Quarantine** — AES-256-GCM vault, restore/delete, audit trail | `aegis-quarantine` | [`QUARANTINE_SYSTEM.md`](QUARANTINE_SYSTEM.md) |
| **Windows Scanner** — startup, registry, tasks, services, drivers, extensions, hosts | `aegis-windows` | [`WINDOWS_SCANNER.md`](WINDOWS_SCANNER.md) |
| **Service Integration** — central orchestrator, job manager, health, IPC contract | `aegis-service` | [`SERVICE_INTEGRATION.md`](SERVICE_INTEGRATION.md) |

## 🔜 Upcoming

Roughly in priority order. Scope may shift — feedback and contributions welcome.

| Item | Goal |
|------|------|
| **Real-Time Protection** | Watch filesystem + processes (`notify` / `sysinfo`), feed events through scan → detect → policy (monitor / notify / auto-quarantine). |
| **Signature Updates** | Signed signature + rule bundle delivery, atomic apply, rollback. |
| **UI Wiring** | Finish the prototype → React migration and bind screens to the orchestrator over IPC. |
| **Reporting** | Export scan/detection reports as JSON, HTML, and PDF. |
| **Installer** | Windows installer (MSI/NSIS) with the background service. |
| **Packaging** | Release build pipeline + artifact bundling. |
| **Code Signing** | Authenticode signing for binaries and the installer. |

## How to follow / help

- ⭐ Star the repo to follow progress.
- Tracking issues for upcoming work live in
  [`.github/ISSUES/`](.github/ISSUES/) (drafts) and the GitHub issue tracker.
- See [`CONTRIBUTING.md`](CONTRIBUTING.md) — there's a first-time-contributor
  section and `good first issue` labels.
