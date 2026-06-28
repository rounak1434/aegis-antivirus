# Code Signing (Windows Authenticode)

Signing is **optional**. Aegis builds and runs unsigned; signing adds trust
(SmartScreen/UAC) for distributed releases. Signed targets:

- `aegis-tauri.exe` (the desktop app — "Aegis.exe")
- `aegis-service.exe` (the background service)
- the MSI installer
- the NSIS `-setup.exe` installer

All signatures are **timestamped** (so they remain valid after the cert expires).

## Prerequisites

- A code-signing certificate (`.pfx`), ideally **OV/EV** from a trusted CA.
- `signtool.exe` (Windows SDK). `deploy/sign.ps1` auto-locates it.

## Local signing

```powershell
# From a .pfx on disk
deploy\sign.ps1 -PfxPath cert.pfx -PfxPassword 'p@ss'

# From a cert already imported into the certificate store
deploy\sign.ps1 -Thumbprint A1B2C3D4...

# Defaults sign the built release artifacts; pass -Paths to target specific files.
```

Each file is signed with `signtool sign /fd sha256 /tr <timestamp> /td sha256`
and then verified with `signtool verify /pa`.

## CI signing (optional, via secrets)

`release.yml` signs only when these repository secrets exist:

| Secret | Purpose |
|--------|---------|
| `AEGIS_SIGN_PFX_BASE64` | base64 of the `.pfx` file |
| `AEGIS_SIGN_PFX_PASSWORD` | the `.pfx` password |

`deploy/sign.ps1` reads `AEGIS_SIGN_PFX_BASE64` / `AEGIS_SIGN_PFX_PASSWORD` from
the environment, writes a temporary `.pfx`, signs, and exits **0 with a notice**
if no certificate is present — so forks and unsigned builds never fail.

> Never commit a `.pfx`, password, or thumbprint. Provide them only via local
> args or CI secrets.

## Verifying signatures

```powershell
signtool verify /pa /v "Aegis Antivirus_<ver>_x64-setup.exe"
Get-AuthenticodeSignature ".\dist\*.exe"
deploy\verify-release.ps1 -Dir .\dist     # checks all artifacts
```

## Timestamp authority

Default `http://timestamp.digicert.com` (override with `-TimestampUrl`). Use the
TSA recommended by your certificate provider.

## Tauri built-in signing (alternative)

Tauri can sign during `tauri build` via
`bundle.windows.certificateThumbprint` + `timestampUrl` in `tauri.conf.json`.
We deliberately leave those **empty** so default builds are unsigned and
reproducible; `deploy/sign.ps1` signs as a separate, secret-gated step instead.
