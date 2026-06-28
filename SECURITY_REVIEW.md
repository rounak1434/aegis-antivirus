# Security Review (Phase 13)

A validation-only review of the existing platform. No new features; no detection
changes. No exploitable bug was found, so no engine code was modified — the only
changes are documented advisory acceptances (`deny.toml` / `.cargo/audit.toml`).

## Static analysis

| Tool | Result |
|------|--------|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --exclude aegis-tauri --all-targets --all-features -- -D warnings` | **0 issues** |
| `cargo audit` | 1 vuln + 18 warnings, all transitive — assessed below, then ignored with justification (gate green) |
| `cargo deny` | advisories/licenses/bans/sources (CI); ignores mirror audit |
| `cargo geiger` | not run (heavy); `unsafe` audited manually → **0 unsafe blocks in library code** |

### `unsafe`

`grep` over `crates/*/src`: **zero `unsafe` blocks** in library code. The only
`unsafe` in the repo is `K32GetProcessMemoryInfo` inside a *benchmark* (memory
reporting), never shipped.

### Panic surface

Every non-test `.unwrap()/.expect()` in library code is either a
`Mutex::lock().unwrap()` (panics only if a lock is already poisoned by another
panic) or a lookup of a just-created job (logically infallible). **No
`.unwrap()`/`panic!` on untrusted/parsed input** — parsers return `Result`/
`Option`. Crash/panic count across all runs: **0**.

## Advisory assessment (`cargo audit`)

726 dependencies scanned. Findings, all **transitive** with no upstream fix:

| Advisory | Crate | Source | Reachable in Aegis? |
|----------|-------|--------|---------------------|
| RUSTSEC-2023-0071 (rsa Marvin timing, 5.9) | `rsa` | yara-x | **No** — Aegis performs no RSA. Signing is Ed25519; vault is AES-256-GCM. |
| RUSTSEC-2024-0411..0420, 0429 (gtk/atk/gdk/glib) | GTK3 bindings | Tauri (Linux) | **No** — Linux-only deps, not compiled in the Windows (MSVC) target. |
| RUSTSEC-2024-0370, 2025-0075/0080/0081/0098/0100/0141 | unic-ucd, bincode, … | yara-x / regex / tauri | Unmaintained warnings only; no known vuln. |

Each is ignored in `deny.toml` and `.cargo/audit.toml` **with a rationale
comment** so the CI security gate stays green without hiding a real risk. Tracked
for removal if/when upstreams ship fixes.

## Manual hardening review

| Area | Finding |
|------|---------|
| **Path traversal** | Quarantine restore rejects `..`, relative paths, missing parents, and overwrites (`vault.rs` `validate_*`). Update install writes only under `<data>/installed` via fixed names. |
| **Symlink handling** | Scanner records symlinks but never hashes/follows them unless `Deep` mode; no symlink-loop hashing. |
| **Quarantine integrity** | Restore decrypts then verifies SHA-256 vs. the stored digest; mismatch → `integrity_mismatch` audit event + refusal. Originals are AES-256-GCM (never plaintext at rest). |
| **Update verification** | Full gate: Ed25519 signature (pinned key) + SHA-256 payload hash + anti-rollback (strictly-newer) + min-app-version. Verified before download *and* before install. |
| **Rollback protection** | Anti-rollback rejects older/equal versions; tested (`anti_rollback_rejects_older_version`). |
| **Temp files** | Atomic install via copy-to-`.tmp` + rename; tests use `tempfile` (secure, unique). No predictable temp paths in shipped code. |
| **TOCTOU** | Scan/restore open-then-read; window is small and the integrity check is on the bytes actually read. Acceptable for a user-mode scanner; noted in KNOWN_LIMITATIONS. |
| **Error propagation** | Typed `thiserror` errors throughout; engine-boundary errors map to `Result<_, String>` at the Tauri layer (no panic across FFI). |
| **IPC** | Tauri commands validate enum args via serde; unknown commands are rejected by the framework. Service-process named-pipe + signed-client enforcement is future (KNOWN_LIMITATIONS). |

## Input-rejection (fuzz-lite) coverage

Malformed-input handling is covered by existing negative tests (no panic, typed
error/`None` returned): tampered update payload (hash mismatch), invalid/garbage
Ed25519 signature + key encoding, anti-rollback, min-app, bad signature-file
lines, broken YARA rule compilation, non-JSON settings (`save_settings` rejects),
quoted/garbage CSV (schtasks/driverquery), hosts parsing, version parsing. A
dedicated `cargo-fuzz` harness (nightly) is recommended future work.

## Conclusion

No exploitable defect found. Security posture is strong: zero library `unsafe`,
no untrusted-input panics, signed+hashed updates, integrity-checked encrypted
quarantine, path-traversal guards. The only outstanding items are **transitive,
unreachable advisories** (documented + ignored) and architectural hardening
(separate service process / signed IPC) tracked in KNOWN_LIMITATIONS.
