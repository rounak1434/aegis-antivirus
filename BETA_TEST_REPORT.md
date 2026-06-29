# Beta Test Report — Aegis Antivirus v1.0.0-rc1

Validation for public-beta readiness. Automated/code-level validation was run in
this environment; install/GUI/coexistence items that require target VMs, an
admin session, or a packaged build are marked **NOT RUN (target host)** with the
method to execute them. No new features were added.

## Functional validation (automated)

`cargo test --workspace --exclude aegis-tauri` → **118 passed / 0 failed** (32
suites). Feature → covering tests:

| Feature | Result | Covered by |
|---------|--------|-----------|
| Quick / Full / Deep / Custom scan modes | ✅ | `aegis-scan` unit + integration (mode options, depth, hidden, symlink, cancel) |
| Scan cancellation / progress / ETA | ✅ | `scan_cancellation_stops_work`, `progress_callback_reports_metrics` |
| Detection (hash + YARA + heuristics + scoring) | ✅ | `aegis-detect` (16), `aegis-yara` (5), `aegis-signatures` (4) |
| Quarantine / restore / delete / integrity | ✅ | `aegis-quarantine` (14) incl. integrity-mismatch + path-traversal |
| Windows persistence scanner | ✅ | `aegis-windows` (23) all surfaces + heuristics |
| Real-time protection + policies | ✅ | `aegis-realtime` (14) file/process events, monitor/notify/auto-quarantine |
| Signature/rule updates + verification | ✅ | `aegis-update` (17) signed flow, tamper, anti-rollback, min-app |
| Update rollback | ✅ | `rollback_restores_previous_version` |
| Service orchestration (scan→detect→quarantine→update) | ✅ | `aegis-service` (12) end-to-end + lifecycle |

UI/IPC: `npm test` (vitest) green (stores + typed IPC wrappers, mocked invoke).

## Safe test files

Detection mechanism validated in-suite with **synthetic malware-like markers**
(known-bad hash, YARA `$marker`, double-extension, encoded-PowerShell, temp-exe)
— no real malware is stored in the repo.

For a live beta check, use the standard **EICAR** test string (a harmless,
industry-standard AV test file). Procedure (run on the tester's machine, not
committed here — committing EICAR would trip the tester's own AV on the repo):

1. Create `eicar.com` from the official EICAR string (eicar.org).
2. Add a YARA rule matching it (`strings: $e = "EICAR-STANDARD-ANTIVIRUS-TEST-FILE"`).
3. Scan → expect detection; AutoQuarantine → expect isolation; Restore → file
   returns + integrity OK; Delete → shredded.
4. Confirm **no false positive** on a copy of `notepad.exe` / project sources.

## Telemetry

**Aegis has NO telemetry, analytics, tracking, or phone-home.** A source audit
(`grep` for reqwest/http/analytics/telemetry/sentry/tracking) found the only
network code is the **signed update downloader** (`aegis-update`, `reqwest`),
which fetches updates the user/admin initiates and is off until an update is
checked. No data leaves the machine otherwise. Nothing to disable; nothing to
opt out of.

## Accessibility (UI)

| Item | Status | Evidence |
|------|--------|----------|
| Keyboard navigation | ✅ | All controls are native `<button>`/`<a>`/`<input>`; toggles use `role="switch"` + `aria-checked`; mode tiles `aria-pressed`; checkboxes/labels `aria-label` |
| Dark mode | ✅ | Single warm-dark theme (design system); no light-mode dependency |
| High DPI / screen scaling | ✅ (by platform) | WebView2 honors OS DPI; layout is fluid (grid/flex) |
| Minimum resolution | ✅ | Window `minWidth 1040 × minHeight 680` (tauri.conf); content reflows at 1080/720 breakpoints |
| Contrast | ⚠️ partial | Warm-dark palette meets AA for primary text; a formal axe/contrast audit is a beta follow-up |

## Performance (vs. Phase 13)

Engine numbers unchanged (no code change): scan **70.8k files/s** @100k, detect
peak **27.4 MiB**, quarantine 973/1004 f/s, RTP 864 ev/s, update verify
~470 MiB/s. See `PERFORMANCE_REPORT.md`.

**Idle RAM/CPU, startup/shutdown, sleep/resume:** NOT RUN (need a packaged,
running app). Method: `Measure-Command` for startup; `Get-Counter
'\Process(aegis-*)\% Processor Time'` and `Working Set` for idle CPU/RAM.

## Bug triage

| Severity | Count | Notes |
|----------|------:|-------|
| Critical | 0 | none found |
| High | 0 | none found |
| Medium | 0 (deferred items) | tracked in `KNOWN_ISSUES.md` (single-thread detection, in-memory ScanReport, polling RTP) |
| Low | — | cosmetic/contrast follow-ups |

No Critical/High issues → no code fixes required this phase.

## Not run in this environment (target-host checklist)

Windows 10 22H2 / 11 23H2 / 11 24H2 matrix · fresh/upgrade/uninstall · service
recovery · startup/shutdown/sleep-resume · coexistence (Defender, VS, VS Code,
Git, Steam, Docker, WSL, VMware/VirtualBox) · idle RAM/CPU. Execute per
`COMPATIBILITY_MATRIX.md` + `RELEASE_CHECKLIST.md` on the beta hosts.
