# Deployment & Packaging

How the Aegis installers are built and how the AegisService is deployed. No
engine code is involved here — this is packaging only.

## Building the installers

```powershell
# From the repo root, on Windows with Rust (MSVC) + Node + Tauri prereqs:
deploy\build-installers.ps1
# or a fast app-only smoke build (no MSI/NSIS):
deploy\build-installers.ps1 -SkipBundle
```

`build-installers.ps1`:

1. Builds `aegis-service` (release) and stages it as the Tauri **sidecar**
   `src-tauri/binaries/aegis-service-x86_64-pc-windows-msvc.exe`
   (referenced by `bundle.externalBin` in `tauri.conf.json`).
2. Builds the UI (`npm ci && npm run build`).
3. Runs `npm run tauri build` → MSI + NSIS under
   `src-tauri/target/release/bundle/`.
4. Produces a portable ZIP under `target/portable/`.

The Tauri bundler fetches the WiX (MSI) and NSIS toolchains on first run.

## Bundle configuration (`src-tauri/tauri.conf.json`)

- `bundle.targets`: `["msi", "nsis"]`.
- `bundle.externalBin`: bundles `aegis-service.exe` alongside the UI.
- `bundle.windows.wix.fragmentPaths`: `wix/service.wxs` (+ `componentRefs`).
- `bundle.windows.nsis.installerHooks`: `nsis/installer-hooks.nsh`;
  `installMode: both` (per-user or machine-wide).

## Service registration

Two equivalent paths register the service; both require Administrator:

- **Installer-driven** — NSIS `installer-hooks.nsh` (`NSIS_HOOK_POSTINSTALL`) and
  the MSI `wix/service.wxs` custom actions run `sc.exe` to create, describe,
  configure recovery, and start `AegisService`.
- **Manual / scripted** — `deploy/service-control.ps1`:

```powershell
deploy\service-control.ps1 -Action install -BinPath "C:\Program Files\Aegis Antivirus\aegis-service.exe"
deploy\service-control.ps1 -Action start
deploy\service-control.ps1 -Action stop
deploy\service-control.ps1 -Action restart
deploy\service-control.ps1 -Action remove
deploy\service-control.ps1 -Action status
```

Service settings applied:

| Setting | Value |
|---------|-------|
| Start type | `auto` (starts at boot / after reboot) |
| Recovery | restart at 60s, 60s, then 120s; reset count daily (`sc failure`) |
| Display name | Aegis Security Service |
| Account | LocalSystem (default) |
| Stop | graceful (the service handles `SERVICE_CONTROL_STOP`) |

## Data directories

Created by the installer (and `service-control.ps1`):

```
%ProgramData%\Aegis\{Updates, Quarantine, Logs, Database}
```

These are **preserved** on upgrade and standard uninstall.

## Logs

| Log | Path |
|-----|------|
| Installer (NSIS hook) | `%ProgramData%\Aegis\Logs\install.log` |
| Service control | `%ProgramData%\Aegis\Logs\service.log` |
| Uninstall (NSIS hook) | `%ProgramData%\Aegis\Logs\uninstall.log` |
| MSI (when run with `/l*v`) | path passed to `msiexec` |

## Test matrix

| Scenario | How |
|----------|-----|
| Fresh install | run MSI/NSIS on a clean VM; `sc query AegisService` = RUNNING |
| Upgrade | install older → newer; confirm `ProgramData\Aegis` retained |
| Repair | `msiexec /fa <msi>` (MSI) / re-run setup (NSIS) |
| Uninstall | Apps → uninstall; data retained; `cleanup-data.ps1` for full wipe |
| Service restart | `service-control.ps1 -Action restart` |
| Silent install/uninstall | `msiexec /qn` · NSIS `/S` |

## Validation status (this repo)

- ✅ `tauri.conf.json` valid; `wix/service.wxs` XML well-formed; all
  `deploy/*.ps1` parse; `nsis/installer-hooks.nsh` present.
- ⏳ **Not run here:** the full `tauri build` (MSI/NSIS bundling fetches the
  WiX/NSIS toolchains) and the actual MSI/NSIS install + service registration —
  these need the Tauri bundler toolchain **and an Administrator session**, which
  this build environment does not provide. Run `deploy\build-installers.ps1` on a
  Windows machine with admin to produce + verify the installers.

## Out of scope (next phase)

Code signing (Authenticode) of the binaries + installers, and release
engineering, are intentionally **not** part of this phase.
