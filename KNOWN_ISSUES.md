# Known Issues — v1.0.0-rc1 (public beta)

> **GA decision (Phase 15): REQUIRES RC2** — not for any defect (none found), but
> the v1.0.0 gate (real-host Windows install/service/GUI matrix) is outstanding.
> See `FINAL_VALIDATION_REPORT.md`. Code is frozen + GA-quality; v1.0.0 is a
> tag-only promotion once the on-host matrix passes.


Beta-facing, severity-classified. No **Critical** or **High** issues are open
(those would block the RC). Items below are **Medium/Low** and deferred, with
workarounds. Full engineering detail in `KNOWN_LIMITATIONS.md`.

## Critical / High

**None.** Test suite green (118 Rust + vitest), 0 `unsafe`, 0 panics, clean
audit. No blocker found.

## Medium (deferred — tracked)

| ID | Issue | Impact | Workaround |
|----|-------|--------|-----------|
| M1 | Detection is single-threaded (~1.8k files/s) | Deep scans of huge trees are slower than the multi-threaded scan stage | Use Quick/Custom on hot paths; parallelization planned |
| M2 | `ScanReport` held fully in memory | Whole-disk Full scan of millions of files uses more RAM | Scan subtrees; streaming variant planned |
| M3 | Process monitoring is polling (~1 s) | A very short-lived process can slip between polls | File-write monitoring still catches dropped payloads |
| M4 | RTP scans after write (user-mode), no inline block | Malware briefly exists on disk before detection | Quarantine fires immediately on detection; kernel minifilter is future |
| M5 | In-process orchestrator (no separate elevated service yet) | Signed-IPC client isolation not active | Service binary ships + registers; wiring is next |

## Low

| ID | Issue | Impact |
|----|-------|--------|
| L1 | No formal contrast/axe accessibility audit yet | Possible AA gaps in secondary text |
| L2 | Content heuristics cap at 1 MiB/file | Abuse strings past 1 MiB missed |
| L3 | Updates: no key rotation / delta updates | Operational only; verification is sound |
| L4 | Large dev/VM/game trees can be noisy under RTP | Use recommended exclusions (COMPATIBILITY_MATRIX) |

## Supply chain (accepted, transitive)

`rsa` Marvin advisory via yara-x — **unreachable** (Aegis uses Ed25519/AES-GCM);
GTK3 binding advisories are **Linux-only** (not in the Windows build). Ignored
with justification — see `SECURITY_REVIEW.md`.

## Reporting a beta issue

Use the GitHub issue templates. **Security vulnerabilities:** do not open a
public issue — see `SECURITY.md`.
