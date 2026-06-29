# Final Validation Report — v1.0.0-rc1 → GA decision

Stabilization pass on the release candidate. No features/architecture changes.
This report separates **what was executed and verified** from **what the GA gate
requires but could not be executed in this environment** — and states the
resulting release decision on the evidence.

## Release decision: **REQUIRES RC2**

**Not** because a defect was found — none was. The GA criteria for v1.0.0
explicitly require **real-host validation** (Windows 10/11 install, service
lifecycle, installer MSI/NSIS, GUI functional pass, idle RAM/CPU, sleep/resume).
Those were **NOT EXECUTED** here (no Administrator session, no packaged-installer
build toolchain, no target VMs, no running GUI). Per "every recommendation must
be supported by observed results," GA cannot be certified on unobserved gates.

→ Complete the on-host matrix below on real Windows machines. If it passes with
no new Critical/High issue, promote `v1.0.0-rc1` → **`v1.0.0`** (a tag-only
promotion; the code is frozen). If any Critical/High surfaces, fix → cut RC2.

## Executed & verified (this environment)

| Check | Result |
|-------|--------|
| `cargo test --workspace --exclude aegis-tauri` | **118 / 118 pass** (reproducible across runs) |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy … -D warnings` | clean (0 issues) |
| `cargo audit` | green (transitive advisories justified — SECURITY_REVIEW) |
| `npm run build` + `npm test` (vitest) | green |
| Library `unsafe` / untrusted-input panics | 0 / 0 |
| Crash count / panic count | 0 / 0 |

### Functional (automated, via the suite)

Every user-facing workflow is exercised by tests and passes: Quick/Full/Deep/
Custom scan, RTP (+ monitor/notify/auto-quarantine policies), quarantine /
restore / delete (+ integrity + path-traversal guards), updates (signed flow,
tamper-reject, rollback, anti-rollback, min-app), Windows persistence scanner,
service orchestration, settings persistence (orchestrator `get/save_settings`).

### Safe detection (mechanism)

Detection validated in-suite with **synthetic, non-malicious markers** (known-bad
hash, YARA `$marker`, double-extension, encoded-PowerShell). The **EICAR** live
procedure is documented for the host run (not committed — committing EICAR would
trip the tester's own AV on the repo).

### Performance (reproducible benchmarks)

| Metric | Value (vs. Phase 13) |
|--------|----------------------|
| Scan @10k files | 46,297 files/s · 723 MiB/s (≈ prior 47k) |
| Scan @100k files | ~70.8k files/s (Phase 13) |
| Detect @10k | 1,647 files/s · 607 µs/file · **peak 27.2 MiB** |
| Quarantine | 973 / 1,004 files/s (q/restore) |
| RTP | 864 events/s · 1,157 µs/event |
| Update verify (8 MiB) | ~470 MiB/s; install 17.6 ms |

Numbers are stable across repeated runs — no regression.

## NOT EXECUTED — GA gate (run on real hosts)

These are the outstanding gates; methods are in `RELEASE_CHECKLIST.md` /
`COMPATIBILITY_MATRIX.md`:

- **OS matrix:** Windows 10 22H2 · 11 23H2 · 11 24H2.
- **Install:** admin + standard-user install · upgrade · repair · uninstall ·
  reinstall (MSI, NSIS, portable ZIP) · silent install/uninstall · ProgramData
  preservation · directory permissions.
- **Service:** registration · auto-start · crash recovery · graceful shutdown ·
  reboot · sleep/resume · Windows-Update reboot.
- **GUI functional:** every workflow through the running app.
- **Performance on host:** cold/warm startup · idle RAM · idle CPU · shutdown.
- **Coexistence:** Defender, VS, VS Code, Git, Steam, Docker, WSL, VMware/VBox.

Blocker is environmental (sandbox: no admin / no installer toolchain / no VMs /
no GUI), identical to Phases 11 & 14 — not a code defect.

## Bug summary

| Severity | Count | Action |
|----------|------:|--------|
| Critical | 0 | — |
| High | 0 | — |
| Medium | 5 | deferred + documented (KNOWN_ISSUES M1–M5) |
| Low | 4 | deferred + documented (L1–L4) |

No Critical/High → no code fix this phase. No speculative refactors.

## Recommendation

Code and automation are **GA-quality and frozen**. Tag the binary-identical
`v1.0.0` **only after** the on-host matrix passes. Until then the honest status
is **RC — REQUIRES RC2 (on-host validation outstanding)**, and `V1_RELEASE_NOTES.md`
is prepared for the eventual promotion.
