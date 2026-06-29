# Aegis Antivirus v1.0.0 — Release Notes (prepared)

> **Status: prepared draft.** These notes ship with the `v1.0.0` tag once the
> on-host validation gate in `FINAL_VALIDATION_REPORT.md` passes. The code is
> frozen at `v1.0.0-rc1`; GA is a tag-only promotion.

Aegis is an open-source, Rust-powered, privilege-separated antivirus for
Windows 10/11 — explainable detections, encrypted quarantine, and a secure
signed-update system.

## Highlights

- **Multi-threaded file scanner** — SHA-256 + MD5, hidden/symlink handling, live
  throughput/ETA, cancellation. ~70k files/s at 100k files.
- **Layered detection** — hash signatures + YARA-X rules + heuristics
  (entropy/packing, double/suspicious extensions, script & PowerShell abuse),
  additive 0–100 score with **explainable evidence** (no black-box scores).
- **Encrypted quarantine vault** — AES-256-GCM at rest, SHA-256 integrity-checked
  restore, path-traversal guards, audit trail.
- **Windows persistence scanner** — startup, registry Run/RunOnce, scheduled
  tasks, services, drivers, browser extensions, hosts file.
- **Real-time protection** — file + process monitoring (notify/sysinfo) with
  monitor / notify / auto-quarantine policies.
- **Secure updates** — Ed25519-signed + SHA-256 + anti-rollback + min-app gate,
  with rollback.
- **Central service** — `AegisService` orchestrates all engines; the UI is
  non-privileged and reaches engines only through it.

## Install

MSI, NSIS installer, or portable ZIP. Machine-wide install registers the
`AegisService` (auto-start + crash recovery). See `INSTALLATION.md`. Verify a
download with `deploy/verify-release.ps1` (`SHA256SUMS` + optional signature).

## Security & supply chain

Every release carries `SHA256SUMS` and a CycloneDX **SBOM**. Optional Authenticode
signing. 0 `unsafe` in library code; `cargo audit` green (transitive advisories
assessed in `SECURITY_REVIEW.md`).

## Known issues

No Critical/High. Medium/Low items (single-threaded detection, in-memory scan
report, polling process monitor, user-mode RTP) in `KNOWN_ISSUES.md`.

## Not included (by design)

No telemetry/analytics/phone-home. No cloud reputation, behavioral AI, or EDR.
No kernel minifilter (user-mode; cannot block on-write inline).

## Thanks

Built in the open. Contributions welcome — see `CONTRIBUTING.md`.

---

### Artifacts (attached at GA)

- `Aegis Antivirus_1.0.0_x64_en-US.msi`
- `Aegis Antivirus_1.0.0_x64-setup.exe`
- `Aegis-Antivirus-portable.zip`
- `SHA256SUMS` (+ `SHA256SUMS.sig` if signed)
- `sbom.cargo.cdx.json`, `sbom.npm.cdx.json`, `license-inventory.csv`
- `release-manifest.json`
