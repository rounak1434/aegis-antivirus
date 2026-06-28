---
title: "Wire the React UI to the AegisService orchestrator over IPC"
labels: ["enhancement", "frontend", "ipc", "help wanted"]
---

## Objective

Connect the migrated React screens to the `AegisOrchestrator` so the UI drives
real scans, threats, quarantine, and health instead of mock data.

## Requirements

- Define typed Tauri commands that map 1:1 to the orchestrator IPC contract:
  `start_scan`, `stop_scan`, `get_scan_status`, `get_threats`,
  `quarantine_detection`, `restore_file`, `delete_quarantine_item`,
  `run_windows_scan`, `get_service_health`.
- Mirror the Rust DTOs as TypeScript types in `src/types/`.
- Stream scan progress + alerts to the UI (Tauri events) and reflect them in the
  Zustand stores.
- Preserve the prototype's visual design (see `PROTOTYPE_AUDIT.md`); no new
  visual system.
- The UI must remain non-privileged — all engine access goes through the service.

## Acceptance Criteria

- [ ] Dashboard, Scan Center, Threat Center, and Quarantine read live data.
- [ ] Start/stop scan + quarantine/restore/delete work end-to-end from the UI.
- [ ] TypeScript types match the Rust contract (no `any`).
- [ ] `npm run build` and `cargo build` succeed; visual parity with the prototype.
