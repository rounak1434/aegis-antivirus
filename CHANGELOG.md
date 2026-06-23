# Changelog

All notable changes to Aegis Antivirus will be documented in this file.

## Unreleased - Prototype Migration (Phase B: all core screens)

### Added
- Converted **Scan Center** (`features/scan-center`) — posture ring, per-engine
  activity, current path, and a live detections feed driven by a self-contained
  `useScanSimulation` hook (swappable for IPC ScanProgress events in Phase D).
- Converted **Threat Center** (`features/threat-center`) — detections table,
  severity/risk chips, filter chips, and the per-threat evidence drawer (risk
  score, recommended action, file evidence, detection layers, reasons; Esc to close).
- Converted **Quarantine** (`features/quarantine`) — encrypted-vault banner,
  stats, bulk selection bar with select-all, and restore/delete (delete confirm).
- Converted **Real-time** (`features/realtime`) — 6 shield cards with working
  toggles, live event feed, controlled-folder access, allowed apps.
- Converted **Settings** (`features/settings`) — sticky sub-tabs, engine/behaviour/
  privacy toggles, exclusions, and the simulated signed signature-update flow.
- Routed all five screens in the HashRouter shell.

## Unreleased - Prototype Migration (Phase A + B slice)

### Added
- `PROTOTYPE_AUDIT.md` — authoritative inventory of the design prototype:
  design tokens, UI/component inventory, navigation map, and 9-screen map.
- React app shell rebuilt to match the prototype: frameless `WinBar` (real
  Tauri window controls), grouped `Sidebar` (Overview/Protect/System), sticky
  `TopBar` with per-route title/crumb.
- Typed `<Icon>` component ported from the prototype's `shell.js` icon set.
- Fully converted **Dashboard** screen (posture ring, scan-launch with mode
  tiles, stat tiles, persistence surface health, recent activity) with typed
  seed data in `features/dashboard/data.ts`.
- `react-router-dom` (HashRouter) with an `AppShell` layout route; interim
  `SectionComingNext` view for screens queued in later slices.
- `.claude/launch.json` dev-server config for the Aegis UI.

### Changed
- **Replaced the generic emerald/slate React template** that diverged from the
  prototype. `tailwind.config.ts` and `src/styles.css` now encode the
  prototype's warm-dark / terracotta Anthropic design system verbatim (the
  single source of truth for pixel parity).

### Verified
- `npm run build` (`tsc && vite build`) passes; 0 npm vulnerabilities after
  adding the router.

## 0.1.0-phase1 - Unreleased

### Added

- Documented service-owned architecture for `AegisService`.
- Defined desktop UI, Windows service, engine crates, database layer, update system, and IPC boundary.
- Added Phase 1 roadmap and task plan.
- Added native Tauri v2 + React + TypeScript + TailwindCSS + Zustand frontend scaffold.
- Added Rust workspace with `aegis-common`, `aegis-ipc`, `aegis-db`, `aegis-service`, `aegis-update`, and `aegis-quarantine` crates.
- Added initial SQLite migration for settings, scan history, threats, detections, quarantine, YARA rules, notifications, update history, and audit logs.
- Added `AegisService` Windows service lifecycle skeleton.
- Added Tauri command bridge for service status and scan requests.
- Added `DEVELOPMENT.md` with local tooling and validation commands.

### Changed

- Reclassified the existing HTML work as design prototype material and preserved it under `design-prototype/`.

### Security

- Established privilege-separation requirement: the Tauri app is UI-only; `AegisService` owns monitoring, scheduled scans, quarantine, and updates.
- Upgraded frontend dev tooling until `npm audit --audit-level=moderate` reports zero vulnerabilities.
