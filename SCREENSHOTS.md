# Screenshots

Inventory of the UI screens to capture for the README and release pages. The
design source of truth is the prototype under
[`design-prototype/`](design-prototype/); the React UI is mid-migration, so the
images in [`docs/screenshots/`](docs/screenshots/) are **placeholders** until the
screens are wired.

## Inventory

| # | Screen | File | Prototype source | What it shows |
|---|--------|------|------------------|---------------|
| 1 | Dashboard | `docs/screenshots/dashboard.png` | `design-prototype/dashboard.html` | Protection posture ring, scan launch, stat tiles, persistence-surface health, recent activity |
| 2 | Scan Center | `docs/screenshots/scan-center.png` | `design-prototype/scan.html` | Live scan progress ring, per-engine activity, throughput, streaming detections |
| 3 | Threat Center | `docs/screenshots/threat-center.png` | `design-prototype/threats.html` | Detection list, risk scores, the evidence detail drawer |
| 4 | Quarantine | `docs/screenshots/quarantine.png` | `design-prototype/quarantine.html` | Encrypted vault, restore / delete actions, audit info |
| 5 | Real-Time Protection | `docs/screenshots/realtime.png` | `design-prototype/realtime.html` | Shield toggles, live event feed, controlled-folder access |
| 6 | Settings | `docs/screenshots/settings.png` | `design-prototype/settings.html` | Engine toggles, sensitivity, exclusions, update flow, tamper protection |

## How to capture

### From the React app (preferred, once a screen is wired)

```bash
npm install
npm run tauri dev      # launches the desktop app
```

Then capture each window (Windows: `Win + Shift + S`) and save as the file name
in the table above (PNG, 2× scale, the app's default 1280×820 window).

### From the prototype (until the React screen lands)

```bash
# serve the static prototype
npx serve design-prototype
# open http://localhost:3000/dashboard.html (etc.) and capture
```

## Guidelines

- **PNG**, 16:10-ish, no personal data on screen (use the prototype's mock data).
- Keep the dark "warm-terracotta" theme — do not restyle.
- Optimize before committing (e.g. `oxipng -o4` or `pngquant`); keep each under
  ~400 KB.
- Update the README image table only after the real files land.
