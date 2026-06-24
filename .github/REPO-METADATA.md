# Repository Metadata Recommendations

Settings to apply in the GitHub UI (Settings → General, and the About panel).
These cannot be set from a commit; apply them manually or via `gh`.

## Short Description (About)

> Open-source Windows antivirus & persistence scanner — Rust engine
> (scanner · YARA-X detection · encrypted quarantine) with a Tauri + React UI.

## Repository Topics

```
antivirus, security, malware-detection, rust, tauri, yara, windows,
endpoint-security, threat-detection, quarantine, react, typescript,
infosec, edr, cybersecurity
```

Apply via CLI:

```bash
gh repo edit rounak1434/aegis-antivirus \
  --description "Open-source Windows antivirus & persistence scanner — Rust engine (scanner, YARA-X detection, encrypted quarantine) with a Tauri + React UI." \
  --add-topic antivirus --add-topic security --add-topic malware-detection \
  --add-topic rust --add-topic tauri --add-topic yara --add-topic windows \
  --add-topic endpoint-security --add-topic threat-detection \
  --add-topic quarantine --add-topic react --add-topic typescript \
  --add-topic infosec --add-topic edr --add-topic cybersecurity
```

## Recommended Labels

| Label | Color | Purpose |
|-------|-------|---------|
| `bug` | `#d73a4a` | Confirmed defect |
| `enhancement` | `#a2eeef` | New feature / request |
| `triage` | `#ededed` | Needs initial review |
| `security` | `#b60205` | Security-sensitive |
| `documentation` | `#0075ca` | Docs only |
| `good first issue` | `#7057ff` | Newcomer-friendly |
| `help wanted` | `#008672` | Maintainers want help |
| `scanner` | `#fbca04` | `aegis-scan` |
| `detection` | `#fbca04` | `aegis-detect` / `yara` / `signatures` |
| `quarantine` | `#fbca04` | `aegis-quarantine` |
| `windows-scanner` | `#fbca04` | `aegis-windows` |
| `ui` | `#c5def5` | Tauri / React frontend |

## Suggested Settings

- Enable **Discussions** (referenced by the issue-template config).
- Enable **Private vulnerability reporting** (Settings → Security).
- Default branch: `master` (or rename to `main`).
- Branch protection on the default branch: require PR review + status checks.
