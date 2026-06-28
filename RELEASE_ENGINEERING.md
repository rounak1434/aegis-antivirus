# Release Engineering

How a trustworthy Aegis release is produced. No engine, service, or installer
*behavior* changes here — this is packaging, signing, checksums, SBOM, and
GitHub-release automation.

## One command

```powershell
deploy\build-installers.ps1      # MSI + NSIS + portable ZIP  (Phase 11)
deploy\sign.ps1                   # OPTIONAL Authenticode signing  (CODE_SIGNING.md)
deploy\make-release.ps1           # checksums + SBOM + inventories + manifest
```

`make-release.ps1` assembles everything into `./dist`.

## Artifacts (in `dist/`)

| Artifact | Produced by |
|----------|-------------|
| `Aegis Antivirus_<ver>_x64_en-US.msi` | Tauri (WiX) |
| `Aegis Antivirus_<ver>_x64-setup.exe` | Tauri (NSIS) |
| `Aegis-Antivirus-portable.zip` | `build-installers.ps1` |
| `SHA256SUMS` | `make-release.ps1` (`Get-FileHash`) |
| `SHA256SUMS.sig` | `make-release.ps1` (GPG, **optional**) |
| `sbom.cargo.cdx.json` / `sbom.npm.cdx.json` | `generate-sbom.ps1` (CycloneDX) |
| `license-inventory.csv` | `generate-sbom.ps1` (cargo metadata) |
| `dependency-tree.txt` | `generate-sbom.ps1` (`cargo tree`) |
| `release-manifest.json` | `make-release.ps1` (version, commit, per-artifact hash/size) |

## Signing (optional)

Authenticode signing is **opt-in** and never required — unsigned development
builds work fully. See [`CODE_SIGNING.md`](CODE_SIGNING.md). The signature over
`SHA256SUMS` is likewise optional (GPG via `AEGIS_GPG_KEY_ID`).

## Verifying a release

```powershell
deploy\verify-release.ps1 -Dir .\dist
# checks: SHA256SUMS match, SHA256SUMS.sig (if present), Authenticode signatures
```

## CI/CD (`.github/workflows/release.yml`)

Triggered by a `v*` tag or manual dispatch (Windows runner):

1. Build installers (`build-installers.ps1`).
2. **Sign** if `AEGIS_SIGN_PFX_BASE64` secret is set (else skip).
3. Assemble checksums + SBOM + manifest (`make-release.ps1`).
4. Upload `dist/*` as a workflow artifact.
5. Publish a **draft** GitHub release with all files + auto-generated notes.

Signing is gated entirely on secrets, so forks/PRs build unsigned without
failing.

## Supply chain

- **SHA256SUMS** for every artifact; optional detached **SHA256SUMS.sig**.
- **CycloneDX SBOM** for the Rust workspace (prefers `cargo cyclonedx`, falls
  back to a `cargo metadata` → CycloneDX transform so an SBOM always exists)
  and the frontend (`cyclonedx-npm` / `npm ls` fallback).
- **License inventory** (`license-inventory.csv`) and **dependency tree**.

## Test matrix (release artifacts)

| Scenario | Windows 10 | Windows 11 |
|----------|-----------|-----------|
| Fresh install (MSI / NSIS) | ☐ | ☐ |
| Upgrade (data preserved) | ☐ | ☐ |
| Uninstall (data kept; full wipe optional) | ☐ | ☐ |
| Portable ZIP execution | ☐ | ☐ |
| Signature + checksum verify | ☐ | ☐ |

Run on clean VMs per release; check boxes in the release PR/checklist.

## Validation status (this repo)

- ✅ All `deploy/*.ps1` parse; `release.yml` YAML valid.
- ✅ **SBOM generated** (CycloneDX 1.5, 726 components), license inventory (727
  rows), dependency tree, and `SHA256SUMS` produced locally by the scripts.
- ✅ **Unsigned builds work** — signing is skipped cleanly with no cert.
- ⏳ **Not run here:** the full `tauri build` of the installers and Authenticode
  signing — these need the Tauri WiX/NSIS toolchain, a code-signing certificate,
  and (for install) Administrator. Run on a release host.
