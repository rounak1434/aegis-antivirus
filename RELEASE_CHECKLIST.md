# Release Checklist — v1.0.0-rc1

Gate before publishing a (pre-)release. ✅ = done in this repo; ☐ = run on a
release/beta host.

## Code & quality

- [x] `cargo fmt --all -- --check` clean
- [x] `cargo clippy --workspace --exclude aegis-tauri --all-targets --all-features -- -D warnings` clean
- [x] `cargo test --workspace --exclude aegis-tauri` (118 pass)
- [x] `npm run build` + `npm test` green
- [x] `cargo audit` green (transitive advisories justified)
- [x] 0 `unsafe` in library code; 0 untrusted-input panics
- [x] CHANGELOG + version bumped (`1.0.0-rc1` in tauri.conf + package.json)

## Build & artifacts (release host)

- [ ] `deploy/build-installers.ps1` → MSI + NSIS + portable ZIP
- [ ] `deploy/sign.ps1` (if cert configured) — sign exes + installers, timestamped
- [ ] `deploy/make-release.ps1` → `SHA256SUMS` (+ `.sig`), SBOM, manifest
- [ ] `deploy/verify-release.ps1` passes on the built artifacts

## Functional smoke (per beta host)

- [ ] Fresh install (MSI) → `sc query AegisService` = RUNNING
- [ ] Quick / Full / Deep / Custom scan complete
- [ ] EICAR detected → AutoQuarantine isolates → Restore → Delete
- [ ] No false positive on `notepad.exe` + project sources
- [ ] Windows persistence scan returns
- [ ] Signature update: check → download → install → rollback
- [ ] RTP: drop synthetic sample in a watched folder → alert
- [ ] Upgrade install preserves `ProgramData\Aegis`
- [ ] Uninstall removes service + binaries, keeps data; `cleanup-data.ps1` wipes

## Compatibility (per matrix)

- [ ] Windows 10 22H2 / 11 23H2 / 11 24H2
- [ ] Coexist: Defender, VS, VS Code, Git, Steam, Docker, WSL, VMware/VirtualBox
- [ ] Sleep/resume; reboot service auto-start; service recovery (kill → restart)

## Performance (per host)

- [ ] Idle RAM/CPU within target (record)
- [ ] Startup / shutdown time (record)
- [ ] Scan throughput matches `PERFORMANCE_REPORT.md` ballpark

## Docs & GitHub

- [x] BETA_TEST_REPORT / COMPATIBILITY_MATRIX / KNOWN_ISSUES / RELEASE_CHECKLIST
- [x] README badges + reports linked
- [ ] Tag `v1.0.0-rc1` pushed → `release.yml` builds + publishes a **draft**
- [ ] Maintainer reviews draft, attaches verified artifacts, marks **pre-release**
- [ ] Beta announcement + feedback channel

## Release-readiness assessment

**RC quality: GREEN for code/automation, PENDING host validation.** Code is
verified, hardened, and reproducible; the remaining gate is the on-host Windows
matrix + installer/signing build. Suitable to tag `v1.0.0-rc1` and run a
controlled public beta.
