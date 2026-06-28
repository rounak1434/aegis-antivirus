# Installing Aegis Antivirus

Aegis ships as a Windows desktop app (the UI) plus the **AegisService**
background service. Three install formats are produced from one Tauri config.

## Formats

| Format | File | Best for |
|--------|------|----------|
| **MSI** | `Aegis Antivirus_<ver>_x64_en-US.msi` | Enterprise / Group Policy, machine-wide |
| **NSIS** | `Aegis Antivirus_<ver>_x64-setup.exe` | Interactive install (per-user or machine-wide) |
| **Portable ZIP** | `Aegis-Antivirus-portable.zip` | No-install trial (UI only; no service) |

## Install modes

- **Per-user** (no admin): installs the UI under the user profile. The
  background **service is not registered** (service registration needs admin).
- **Machine-wide** (Administrator): installs to `Program Files`, registers and
  starts **AegisService**, and creates the `ProgramData\Aegis` data layout.

The NSIS installer asks which mode (`installMode: both`); the MSI installs
machine-wide.

### Interactive

- **MSI:** double-click, or `msiexec /i "Aegis Antivirus_<ver>_x64_en-US.msi"`.
- **NSIS:** run the `-setup.exe`.

### Silent

```powershell
# MSI — silent machine-wide install, with a log
msiexec /i "Aegis Antivirus_<ver>_x64_en-US.msi" /qn /norestart /l*v aegis-install.log

# NSIS — silent install
& ".\Aegis Antivirus_<ver>_x64-setup.exe" /S

# Silent uninstall
msiexec /x "{ProductCode}" /qn            # MSI
& "$env:ProgramFiles\Aegis Antivirus\uninstall.exe" /S   # NSIS
```

## What gets installed

```
C:\Program Files\Aegis Antivirus\        ← UI + aegis-service.exe (machine-wide)
C:\ProgramData\Aegis\                     ← user data (preserved on upgrade/uninstall)
        ├── Updates\                      ← downloaded + installed update components
        ├── Quarantine\                   ← encrypted vault
        ├── Logs\                         ← install / service / uninstall logs
        └── Database\                     ← SQLite store
```

Plus the **AegisService** Windows service: `start= auto`, crash-recovery
(restart on failure), graceful stop.

## Upgrade

Re-run a newer installer. Binaries are replaced; **everything under
`ProgramData\Aegis` is preserved** — settings, database, quarantine vault,
signatures, and YARA rules carry over. The service is re-registered.

## Uninstall

Standard uninstall (Settings → Apps, or `msiexec /x`): stops + removes the
service and binaries, and **keeps your data** under `ProgramData\Aegis`.

For a **full wipe** of user data afterwards:

```powershell
deploy\cleanup-data.ps1            # prompts; deletes C:\ProgramData\Aegis
deploy\cleanup-data.ps1 -Force     # unattended
```

## Verify the install

```powershell
sc.exe query AegisService          # should be RUNNING
sc.exe qfailure AegisService       # recovery actions
Get-Content "$env:ProgramData\Aegis\Logs\install.log"
```

## Requirements

- Windows 10 / 11 (x64)
- WebView2 Runtime (bundled with Windows 11; the installer prompts otherwise)
- Administrator rights for the background service (machine-wide install)
