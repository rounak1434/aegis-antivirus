---
title: "Windows installer + background service registration"
labels: ["enhancement", "packaging", "windows"]
---

## Objective

Ship a Windows installer that deploys the Tauri app and registers `AegisService`
as a background Windows service.

## Requirements

- App icon set (`icons/icon.ico`, …) so the Tauri bundle builds and
  `aegis-tauri` can re-enter the test/lint gate.
- Installer (MSI via Tauri/WiX, or NSIS) that installs the UI + service.
- Service install/uninstall with correct ACLs; service starts on boot.
- Clean uninstall (removes service, app, and optionally data) with confirmation.
- Bundle the WebView2 bootstrapper or document the dependency.

## Acceptance Criteria

- [ ] `npm run tauri build` produces a working installer.
- [ ] Installing registers + starts `AegisService`; uninstall removes it.
- [ ] Icons present; `aegis-tauri` no longer excluded from the clippy/test gate.
- [ ] Install/uninstall steps documented.
