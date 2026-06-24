# Aegis Windows Security Scanner (Phase 5)

`aegis-windows` discovers persistence mechanisms across Windows autostart
surfaces, applies heuristics, and converts **suspicious** findings into uniform
[`aegis_detect::ThreatDetection`] objects. Benign autostart entries are not
reported (avoids alert flooding).

## Design

Two layers, cleanly separated for testability:

- **Collectors** (`cfg(windows)`, best-effort) gather raw entries from the live
  system. They never panic and return `[]` when a source is absent or access is
  denied. On non-Windows hosts they compile to empty stubs.
- **Parsers / scanners** (pure, unit-tested on every platform) turn tool output
  or a directory into normalized `PersistenceEntry` values. Collectors are thin
  wrappers over these.

```text
collectors ──► Vec<PersistenceEntry> ──► heuristics::analyze_entry ──► Vec<ThreatEvidence>
  (live)            (normalized)              (per entry)                     │
                                                                             ▼
                                              ThreatDetection::from_evidence (score + level)
```

A `PersistenceEntry` is `{ kind, name, command, location, signed, detail }`.

## Modules & Scan Targets

| Module | Surface | Collector source | Pure test seam |
|--------|---------|------------------|----------------|
| `startup` | Startup folders (user + all-users) | `%APPDATA%` / `%ProgramData%` Start Menu | `scan_dir(dir)` |
| `registry` | `HKCU`/`HKLM` `…\CurrentVersion\Run` + `RunOnce` | `winreg` | `entries_from_pairs` |
| `scheduled_tasks` | Task Scheduler | `schtasks /query /fo CSV /v` | `parse_schtasks_csv` |
| `services` | Windows services | `HKLM\SYSTEM\CurrentControlSet\Services` | `classify` / `entry_from_service` |
| `drivers` | Kernel/FS drivers | `driverquery /v /fo csv` | `parse_driverquery_csv` |
| `browser_extensions` | Chrome, Edge, Firefox | profile `Extensions` dirs / `.xpi` | `scan_chromium_extensions`, `scan_firefox_extensions` |
| `hosts_file` | `…\drivers\etc\hosts` | the hosts file | `parse_hosts` |

`PersistenceKind` (the finding type) maps to the spec's evidence names:
`startup_entry`, `registry_run_key`, `registry_run_once`, `scheduled_task`,
`service_persistence`, `driver_persistence`, `browser_extension`,
`hosts_file_modification`.

## Heuristics

`heuristics::analyze_entry` emits `ThreatEvidence`; an entry becomes a detection
only if at least one heuristic fires (then a `PersistenceMechanism` context item
is prepended). Flags:

- **Executable in a temp directory** — image path under `\Temp\`,
  `\AppData\Local\Temp`, `\Windows\Temp`, `%TEMP%` → `SuspiciousLocation` (+20).
- **Script in a startup location** — `.ps1/.vbs/.bat/.js/.cmd/.hta/.wsf` in the
  Startup folder → `SuspiciousLocation` + `SuspiciousExtension`.
- **Unsigned binary** — `signed == Some(false)` on an executable →
  `SuspiciousLocation`. (Signature state is populated where feasible; `None`
  when not determined.)
- **Suspicious command line** — LOLBin patterns (`regsvr32 /i:http`,
  `mshta http`, `certutil -urlcache`, `bitsadmin /transfer`, `-w hidden`,
  `FromBase64String`, …) → `SuspiciousLocation`.
- **Encoded PowerShell** — `-enc` / `-EncodedCommand` and the
  `aegis-detect` PowerShell-abuse set (`IEX`, `DownloadString`, `Bypass`, …) →
  `PowerShellIndicator` (+25).
- **Browser-extension sideloading** — unpacked (no `update_url`), `external`,
  `development`, or registry-installed → `SuspiciousLocation`.
- **Hosts-file redirect** — overrides a sensitive domain (Windows Update,
  Microsoft, Google, AV vendors) or maps a host to a non-loopback address →
  `SuspiciousLocation`.

### Scoring

Reuses the `aegis-detect` additive 0–100 model. Two evidence variants were added
to the shared model in this phase (integration requirement):
`PersistenceMechanism` (+15) and `SuspiciousLocation` (+20). Combined with
existing `PowerShellIndicator` (+25) / `SuspiciousExtension` (+15), a typical
malicious autostart entry lands in **Medium–High**, and an encoded-PowerShell
task dropped in Temp reaches **High/Critical**.

## Output

`WindowsScanner::scan_all()` runs every collector and returns
`Vec<ThreatDetection>`. `analyze_entries(&[PersistenceEntry])` analyzes a
caller-supplied batch (used by the service and tests). Each detection carries
explainable evidence (`reason()`), a score, and a `ThreatLevel`.

## Testing

- **Unit** — every parser/scanner/heuristic (CSV parsing, hosts parsing, temp
  detection, encoded-PS, LOLBin, unsigned, service classification, sideload).
- **Integration** — mock fixtures across all seven surfaces produce exactly the
  expected six detections; a benign-only environment yields none; `scan_all()`
  never panics.

`cargo test -p aegis-windows`: **23/23 pass** (20 unit + 3 integration).
`cargo clippy --workspace --exclude aegis-tauri --all-targets --all-features
-- -D warnings`: clean.

## Limitations

- **Signature verification is not yet wired** — `signed` is `None` from the live
  collectors (the unsigned heuristic works once a WinVerifyTrust/Authenticode
  check populates it). Planned hardening.
- **Collectors require the host's tools/permissions** — `schtasks` /
  `driverquery` and some `HKLM` keys need elevation for full coverage; missing
  access degrades gracefully to fewer entries, never an error.
- **Firefox parsing is `.xpi`-listing only** — it does not yet read
  `extensions.json` metadata.
- **WMI event-consumer persistence and COM hijacks are out of scope** for
  Phase 5.
- **Collectors are not benchmarked** — output volume is small (hundreds of
  entries); analysis is linear and negligible next to collection I/O.
