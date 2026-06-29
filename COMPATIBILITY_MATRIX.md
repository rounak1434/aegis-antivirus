# Compatibility Matrix — v1.0.0-rc1

Status legend: ✅ pass · ⚠️ caveat · ❌ fail · ☐ **NOT RUN** (needs target host —
no admin / packaged build / VMs in the build environment). Beta testers fill the
☐ boxes on real machines.

## Windows versions

| OS | Fresh install | Upgrade | Uninstall | Service recovery | Startup | Shutdown | Sleep/Resume |
|----|:--:|:--:|:--:|:--:|:--:|:--:|:--:|
| Windows 10 22H2 | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ |
| Windows 11 23H2 | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ |
| Windows 11 24H2 | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ | ☐ |

Architecture: x64 (MSVC). ARM64 untested.

## Coexistence (developer tools)

Goal: no crash, no excessive CPU, no false positives on the tool's own files.

| Software | Coexists | No FP | No CPU spike | Notes |
|----------|:--:|:--:|:--:|-------|
| Microsoft Defender | ☐ | ☐ | ☐ | Aegis is user-mode (no minifilter), so it does **not** conflict with Defender's filter driver; both can run. RTP is scan-after-write, not inline. |
| Visual Studio | ☐ | ☐ | ☐ | Large build trees — exclude `target/`, `obj/` from real-time watch if noisy. |
| VS Code | ☐ | ☐ | ☐ | |
| Git | ☐ | ☐ | ☐ | `.git` churn; watched folders default to user dirs, not repos. |
| Steam | ☐ | ☐ | ☐ | Large game files — Deep scan only on demand. |
| Docker Desktop | ☐ | ☐ | ☐ | WSL2 backend; Aegis scans Windows FS, not the Linux VM. |
| WSL | ☐ | ☐ | ☐ | `\\wsl$` paths not watched by default. |
| VMware / VirtualBox | ☐ | ☐ | ☐ | Large `.vmdk`/`.vdi` — exclude from RTP if scanning on write is heavy. |

## Design notes that aid coexistence

- **No kernel driver / minifilter** — Aegis is user-mode; it cannot conflict
  with another AV's filter stack (it also can't block on-write — see KNOWN_ISSUES).
- **RTP watches user folders by default** (Downloads/Desktop/Documents/Temp/
  profile), not build/VM/game directories — limits false-positive surface.
- **Heuristics are conservative** — suspicious-only persistence reporting; signed
  + known binaries are not flagged.

## Recommended exclusions (beta guidance)

`target/`, `node_modules/`, `.git/`, Docker/VM disk images, Steam library —
add to scan exclusions to avoid noise and CPU on large dev trees.

## How to run the matrix

Per host: install (MSI or NSIS) → reboot → confirm `sc query AegisService` =
RUNNING → run each scan mode → toggle RTP → quarantine/restore/delete a synthetic
sample → sleep/resume → upgrade-install → uninstall (data preserved). Record CPU
(`Get-Counter`) and any FP. See `RELEASE_CHECKLIST.md`.
