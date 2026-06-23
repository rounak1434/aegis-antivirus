# Aegis Antivirus — Prototype Audit (Phase A)

This document is the authoritative inventory of the design prototype under
`design-prototype/`. The prototype is the **product specification**: every
React conversion must match it pixel-for-pixel. Numbers and file paths in the
prototype are illustrative mock data and will be replaced by real state
(Phase C) and backend data (Phase D) — the *visuals*, not the data, are the spec.

## 1. Design Token Inventory

Source of truth: `design-prototype/css/app.css` `:root`. The system is the
Anthropic "warm dark" palette — deep warm charcoals, warm ivory text, a single
terracotta chromatic accent. **No pure black, no pure white.**

### Color

| Token | Value | Role |
|-------|-------|------|
| `--bg` | `#141413` | App canvas (warmest near-black) |
| `--bg-2` | `#1c1c1a` | Sidebar, inset wells, mode tiles |
| `--surface` | `#30302e` | Cards, containers |
| `--surface-2` | `#252523` | Active nav, hover, chips |
| `--raised` | `#3d3d3a` | Elevated / button hover |
| `--fg` | `#faf9f5` | Primary text (ivory) |
| `--fg-2` | `#b0aea5` | Secondary text |
| `--muted` | `#87867f` | Body muted |
| `--faint` | `#6b6964` | Metadata, eyebrows |
| `--border` | `#30302e` | Default border |
| `--border-2` | `#3d3d3a` | Divider / lighter border |
| `--hairline` | `rgba(255,255,255,0.06)` | Row separators |
| `--accent` | `#c96442` | **Terracotta** — the one chromatic element |
| `--accent-dim` | `#d97757` | Lighter terracotta for dark borders |
| `--accent-wash` | `rgba(201,100,66,0.15)` | Accent background tint |
| `--danger` | `#d13d3d` | Malicious / critical |
| `--danger-wash` | `rgba(209,61,61,0.12)` | Danger tint |
| `--warn` | `#d9a84a` | Suspicious / review |
| `--warn-wash` | `rgba(217,168,74,0.12)` | Warn tint |
| `--info` | `#3898ec` | Focus blue (only cool color, a11y) |
| `--info-wash` | `rgba(56,152,236,0.12)` | Info tint |

> Note: a few prototype `@keyframes` (scan pulse, launcher radial) contain a
> stray `rgba(54,211,153,...)` green left over from an earlier palette. The
> `:root` terracotta tokens are authoritative; conversions use `--accent`.

### Typography

| Token | Stack | Use |
|-------|-------|-----|
| `--font-display` | `Georgia, "Times New Roman", serif` | Brand wordmark, hero headings |
| `--font-sans` | `"Arial", system-ui, sans-serif` | All UI text |
| `--font-mono` | `ui-monospace, "JetBrains Mono", Menlo, monospace` | Numbers, paths, eyebrows, badges |

Base: `14px / 1.5`, headings `font-weight:600`, `letter-spacing:-0.01em`.
Mono uses `font-variant-numeric: tabular-nums`.

### Geometry & Elevation

| Token | Value |
|-------|-------|
| `--r-sm` / `--r` | `8px` |
| `--r-lg` | `12px` |
| `--sb-w` (sidebar) | `236px` |
| `--tb-h` (topbar) | `56px` |
| winbar height | `36px` |
| `--shadow` | `0 0 0 1px rgba(255,255,255,0.06)` (ring, not drop) |

### Motion

- `@keyframes rise` — content fade + 6px translateY, `.35s ease`. Stagger
  helpers `.rise` / `.rise-2` (50ms) / `.rise-3` (100ms).
- Scan engine `@keyframes p` — pulsing dot, `1.4s infinite`.
- Toggles, drawer, mode tiles: `.12s–.25s` transitions.

## 2. UI / Component Inventory

Reusable primitives defined in `app.css` (shared across all screens):

| Class | Component | Notes |
|-------|-----------|-------|
| `.winbar` + `.win-ctrls` | **WinBar** | Frameless Win11 title bar, drag region, min/max/close |
| `.app` / `.sidebar` / `.main` | **AppShell** | `236px` grid + sticky sidebar |
| `.brand` | **Brand** | Shield logo + serif "Aegis" + mono version |
| `.nav-label` / `.nav-item` | **Sidebar nav** | Grouped, active bar `::before`, optional `.badge` |
| `.shield-mini` | **SidebarStatus** | "Protected" footer card |
| `.topbar` / `.crumb` | **TopBar** | Sticky, blur, crumb + title + search/bell/Protected pill |
| `.btn` (`.primary`/`.danger`/`.ghost`/`.sm`) | **Button** | |
| `.icon-btn` | **IconButton** | 34px square |
| `.card` (`.pad-lg`) + `.card-h` | **Card** / **CardHeader** | |
| `.grid` `.g-2/.g-3/.g-4` | **Grid** | Responsive at 1080/720px |
| `.stat` (`.s-label/.s-val/.s-delta`) | **StatTile** | Mono value, up/down delta |
| `.pill` (`.ok/.danger/.warn/.info/.muted`) | **Pill** | Status badge w/ dot |
| `.sev` + `.bar` | **SeverityChip** | Risk bar + score |
| `.table` | **Table** | Mono header, hairline rows, hover |
| `.bar-track` / `.bar-fill` | **ProgressBar** | `.warn`/`.danger` variants |
| `.toggle` (`.off`) | **Toggle** | 42×24 switch |
| `.chip` (`.on`) | **FilterChip** | |
| `.drawer` + `.drawer-back` | **Drawer** | 460px right slide-over (threats) |
| `.divider` / `.kbd` | misc | |

Icons: a single inline-SVG set in `js/shell.js` (`I` map) +
`aegisIcon(name)`. Keys: `dashboard, scan, threats, quarantine, realtime,
history, settings, arch, widget, shield, home`. Stroke `1.7`, 24×24,
`currentColor`. → becomes a typed `<Icon name>` React component.

## 3. Navigation Map

Source: `js/shell.js` `NAV`. Grouped sidebar, three sections:

```
Overview
  • Launcher      index.html         (home)
  • Dashboard     dashboard.html     (dashboard)
  • Architecture  architecture.html  (arch, wide layout)
Protect
  • Scan          scan.html          (scan)
  • Threats       threats.html       (threats, badge "3")
  • Quarantine    quarantine.html    (quarantine)
  • Real-time     realtime.html      (realtime)
System
  • Mini widget   widget.html        (widget)
  • Settings      settings.html      (settings)
```

Footer: `.shield-mini` "Protected · Real-time on · defs 2h ago".
Each page declares identity via `<body data-shell data-active data-title
data-crumb [data-wide]>`; `shell.js` injects winbar + sidebar + topbar around
the page body at `DOMContentLoaded`.

→ React: a single `<AppShell>` layout route wrapping nested routes; `data-*`
attributes become per-route metadata (`title`, `crumb`, `active`, `wide`).

## 4. Screen Map

| # | Page | Route | data-active | Title / Crumb | Key blocks |
|---|------|-------|-------------|---------------|------------|
| 1 | `index.html` | `/launcher` | — | Launcher (no shell) | Hero, meta pills, 3 galleries of screen cards, info note |
| 2 | `dashboard.html` | `/` | dashboard | Dashboard · Aegis / overview | **PostureRing**, **ScanLaunch** (4 mode tiles), 4 **StatTiles**, **PersistenceSurface** (6 rows), **RecentActivity** (6 items) |
| 3 | `scan.html` | `/scan` | scan | Deep scan in progress · Aegis / scan | **ScanRing** (% + ring), pause/stop, current path, 4 engine rows, 4 StatTiles, **LiveDetections** feed |
| 4 | `threats.html` | `/threats` | threats | Threats & report · Aegis / detections | 4 StatTiles, filter chips, **ThreatTable**, **ThreatDrawer** (risk score, recommend, evidence kv, layers, reasons) |
| 5 | `quarantine.html` | `/quarantine` | quarantine | Quarantine vault · Aegis / quarantine | Vault banner, 3 StatTiles, selection bar, **QuarantineTable** (checkboxes, lock chip, restore/delete) |
| 6 | `realtime.html` | `/realtime` | realtime | Real-time protection · Aegis / protection | Active banner, **ShieldGrid** (6 shields w/ toggles), **LiveEventFeed**, **ControlledFolderAccess** + allowed apps |
| 7 | `settings.html` | `/settings` | settings | Settings & updates · Aegis / settings | Left sub-tabs (Protection/Scanning/Updates/Exclusions/Security) + panels: engine toggles, sensitivity, scan behaviour, schedule, **update flow**, exclusions, tamper, privacy |
| 8 | `widget.html` | `/widget` | widget | Mini widget & system tray · Aegis / system | **MiniWidget**, tray-state previews, tray menu |
| 9 | `architecture.html` | `/architecture` | arch | Architecture & system design (wide) | Module map, detection pipeline, scan flow, crate layout, security model, roadmap — internal reference |

## 5. State Surfaces (feeds Phase C Zustand stores)

Each screen's mock data identifies the store shape to build later:

- **appStore** — protection posture (`protected / N needs review`), defs version/age, real-time on/off, topbar "Protected" pill.
- **scanStore** — mode (quick/full/deep/custom), progress %, scanned/throughput/elapsed/eta, per-engine counters, live detections, paused.
- **threatStore** — detections [{name, family, severity, score, file, path, sha, size, signed, layers[], rec, reasons}], filter, selected/drawer.
- **quarantineStore** — items [{file, origin, threat, severity, when, size, encrypted}], selection set, vault stats.
- **realtimeStore** — 6 shields [{key, on, title, subtitle, desc}], live events, controlled folders, allowed apps.
- **settingsStore** — engine toggles, sensitivity, scan behaviour, schedules, update channel/state, exclusions, tamper, privacy.

## 6. Conversion Strategy (informs Phase B)

1. **Tokens first** — port `:root` into `src/styles.css` as CSS variables and
   mirror them in `tailwind.config.ts`. This guarantees parity and lets us use
   either semantic classes or Tailwind utilities.
2. **Keep the semantic component classes** (`.card`, `.btn`, `.pill`, `.stat`,
   `.table`, `.toggle`, …) ported verbatim from `app.css` — fastest path to
   pixel parity and the cleanest mapping from prototype markup. Per-page
   `<style>` blocks become component-scoped CSS next to each feature.
3. **Shell** — `WinBar` + grouped `Sidebar` + `TopBar` as React, driven by a
   `NAV` config and per-route metadata, replacing `shell.js`'s DOM injection.
4. **Icons** — `shell.js` `I` map → typed `<Icon name>` component.
5. **Screens** — one feature folder per screen (`src/features/*`), starting
   with Dashboard as the first vertical slice. Mock data lives beside each
   feature, typed, ready to be swapped for store/IPC data in Phases C–D.
6. **Routing** — `react-router-dom` with an `AppShell` layout route. The
   Launcher (`index.html`) has no shell and is a standalone route.

### Reusable assets (lift directly, do not redesign)

- `design-prototype/css/app.css` `:root` + component classes → token layer + global styles.
- `design-prototype/js/shell.js` `I` icon map → `<Icon>`; `NAV` array → sidebar config.
- Per-page `<style>` blocks → component CSS.
- All mock datasets in each page's `<script>` → typed seed data per feature.
