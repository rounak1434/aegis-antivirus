---
title: "Real-Time Protection: filesystem + process monitoring"
labels: ["enhancement", "engine", "help wanted"]
---

## Objective

Continuously monitor filesystem and process activity and run new/changed files
and launched processes through the existing detection pipeline, applying a
configurable policy (monitor / notify / auto-quarantine).

## Requirements

- File monitoring via `notify`: create / modify / rename across Downloads,
  Desktop, Documents, Temp, and the user profile; debounce duplicate events.
- Process monitoring via `sysinfo` (or Windows ETW): new processes, command
  line, executable path.
- Event pipeline reuses the verified engines: scan (`aegis-scan`) → detect
  (`aegis-detect`) → policy → quarantine (`aegis-quarantine`). **Do not** fork or
  reimplement engine logic.
- Policy modes: `MonitorOnly`, `NotifyOnly` (default), `AutoQuarantine`.
- Persist `realtime_events` and `realtime_alerts`; expose start/stop/status
  through `AegisService`.

## Acceptance Criteria

- [ ] `cargo test -p aegis-realtime` passes (policy + mock file/process events).
- [ ] `cargo clippy … -D warnings` clean.
- [ ] Benchmark reports event latency, scan latency, events/sec.
- [ ] Orchestrator exposes `start_realtime` / `stop_realtime` / `get_realtime_status`.
- [ ] `REALTIME_PROTECTION.md` + ARCHITECTURE/TASKS/CHANGELOG updated.
