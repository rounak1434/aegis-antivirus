# Aegis Antivirus Tasks

## Prototype Migration (UI)

### Phase A — Prototype Analysis
- [x] Analyze every prototype page (index, dashboard, scan, threats, quarantine, realtime, settings, widget, architecture).
- [x] Produce design token inventory, UI/component inventory, navigation map, screen map.
- [x] Create `PROTOTYPE_AUDIT.md`.
- [x] Identify that the prior `src/` React scaffold was a generic template diverging from the prototype.

### Phase B — React Conversion
- [x] Install `react-router-dom`.
- [x] Port prototype design tokens into `src/styles.css` + `tailwind.config.ts`.
- [x] Port prototype component classes (.card/.btn/.pill/.stat/.table/.toggle/…) verbatim.
- [x] Create typed `<Icon>` component from `shell.js` icon map.
- [x] Rebuild app shell: `WinBar`, grouped `Sidebar`, `TopBar`, `AppShell` layout route.
- [x] Convert Dashboard screen with full visual parity (first vertical slice).
- [x] Wire `HashRouter` with interim `SectionComingNext` for unconverted routes.
- [x] Verify frontend build (`tsc && vite build`) passes.
- [x] Convert Scan Center screen (`src/features/scan-center`) — live progress simulation hook.
- [x] Convert Threat Center screen + evidence drawer (`src/features/threat-center`).
- [x] Convert Quarantine screen (`src/features/quarantine`) — selection + delete confirm.
- [x] Convert Real-time screen (`src/features/realtime`) — shield toggles + event feed.
- [x] Convert Settings screen (`src/features/settings`) — sub-tabs + signed update flow.
- [ ] Convert Launcher and Architecture pages.

### Phase C — State Layer
- [ ] Zustand stores: app, scan, threat, quarantine, settings, realtime.

### Phase D — Backend Integration
- [ ] Typed Tauri command interfaces: start/stop scan, progress, threats, quarantine, restore, settings.

### Phase E — Production Polish
- [ ] Loading / error / empty states, notifications, keyboard shortcuts, accessibility.

## Phase 1

- [x] Analyze existing project directory.
- [x] Identify current project as static HTML prototype.
- [x] Preserve prototype files under `design-prototype/`.
- [x] Create `ARCHITECTURE.md`.
- [x] Create `ROADMAP.md`.
- [x] Create `TASKS.md`.
- [x] Create `CHANGELOG.md`.
- [x] Add Tauri v2 React TypeScript scaffold.
- [x] Add TailwindCSS configuration.
- [x] Add Zustand state store.
- [x] Add Rust workspace manifest.
- [x] Add `src-tauri` application crate.
- [x] Add `aegis-common` crate.
- [x] Add `aegis-ipc` crate.
- [x] Add `aegis-db` crate.
- [x] Add `aegis-service` crate.
- [x] Add `aegis-update` crate.
- [x] Add `aegis-quarantine` crate.
- [x] Add initial SQLite migration.
- [x] Add UI-to-service IPC bridge contract.
- [x] Document Rust installation requirement if local Cargo is unavailable.

## Phase 2

- [ ] Implement scan job model.
- [ ] Implement scan mode planner.
- [ ] Implement filesystem traversal.
- [ ] Add scan cancellation.
- [ ] Add progress event stream.
- [ ] Add scanner fixture tests.

## Phase 3

- [ ] Implement hash signature matcher.
- [ ] Integrate YARA-X.
- [ ] Implement entropy analyzer.
- [ ] Implement filename and extension heuristics.
- [ ] Implement script indicators.
- [ ] Implement threat scoring.

## Phase 4

- [ ] Implement startup folder scanner.
- [ ] Implement registry Run key scanner.
- [ ] Implement scheduled task scanner.
- [ ] Implement services scanner.
- [ ] Implement drivers scanner.
- [ ] Implement browser extension scanner.
- [ ] Implement hosts file scanner.

## Phase 5

- [ ] Implement file monitoring.
- [ ] Implement process launch monitoring.
- [ ] Add alert event pipeline.
- [ ] Add scheduled scan runner.

## Phase 6

- [ ] Implement encrypted quarantine vault.
- [ ] Implement restore.
- [ ] Implement permanent delete.
- [ ] Implement report generation.
- [ ] Implement report export.

## Phase 7

- [ ] Add installer packaging.
- [ ] Add signing pipeline.
- [ ] Add secure update rollout process.
- [ ] Add release checklist.


