# Aegis Secure Update System (Phase 8)

`aegis-update` downloads, **cryptographically verifies**, installs, and rolls
back updates for the four updatable components. It only produces verified files
on disk and records what's installed; **hot-reloading the running engines is the
service's job** — no engine crate is modified.

## Components

`signature_database` · `yara_rules` · `threat_metadata` · `engine_config`
(`UpdateComponent`).

## Update Manifest

```rust
struct UpdateManifest {
    version: String,
    published_at: DateTime<Utc>,
    sha256: String,          // hex digest of the payload
    signature: String,       // base64 Ed25519 over signed_message()
    url: String,
    size: u64,
    component: UpdateComponent,
    minimum_app_version: String,
}
```

The signature is computed over the canonical message
`"{component}|{version}|{sha256}|{size}"` — binding component, version, payload
digest, and size so none can be altered without breaking the signature.

## Security Model

Every update passes a **full gate** before it goes live (`verify_full`):

1. **Ed25519 signature** over the manifest message, checked against a **pinned**
   public key (provided to the service as hex config — no key is hardcoded).
2. **SHA-256 integrity** — the downloaded payload's digest must equal
   `manifest.sha256`. A mismatch deletes the file and rejects the update.
3. **Anti-rollback** — the candidate version must be **strictly newer** than the
   installed version (dotted-numeric compare). Older/equal versions are rejected
   (defends against rollback/downgrade attacks).
4. **Minimum app version** — the running app must be `>= minimum_app_version`.

Signature is verified **before download** (refuse to fetch unsigned) and the
payload hash is verified **after download**; install **re-verifies the full
gate** before swapping files.

Rejected: invalid signatures, tampered payloads, downgrade attempts, and updates
that need a newer app — each a distinct typed `VerifyError`.

## Download Engine

`Fetcher` trait abstracts transport:
- **`ReqwestFetcher`** (production) — HTTPS via `reqwest` blocking client with
  **byte-range resume**, **timeout**, **bounded retries**, and **gzip**.
- **`LocalFetcher`** (offline/tests) — copies from a local feed dir, also
  supporting resume.

## Local Storage & Rollback

Layout under `<data_dir>/update/`:

```
updates/                 # downloaded payloads (<component>-<version>.bin)
installed/               # live files (<component>.bin)
backup/                  # previous version per component (for rollback)
manifest.json            # installed registry (component → manifest)
backup-manifest.json     # previous manifest per component
```

Install backs up the current live file + manifest, then atomically swaps in the
new payload (copy-to-`.tmp` + rename). **Rollback** restores the backup file and
the previous manifest, and re-points `installed_components`.

## Scheduling

`UpdateSchedule` — `Manual` / `Daily` / `Weekly` / `Startup`. `is_due(last,
now, is_startup)` is pure (unit-tested); the service runs the actual timer.

## Database (migration `006_update.sql`)

- `installed_components` — `component` (PK), `version`, `sha256`,
  `installed_path`, `installed_at_utc`.
- Update actions are logged to the existing `update_history` table (from
  migration 001): downloaded / installed / rolled_back.

## Service Integration

`AegisOrchestrator` owns the update subsystem:

| Method | Purpose |
|--------|---------|
| `init_updates(pubkey_hex, fetcher, app_version)` | Configure with the pinned key. |
| `check_updates(available)` | Filter a manifest feed to signed, app-compatible, newer entries. |
| `download_updates(manifest)` | Fetch + verify (sig before, hash after). |
| `install_updates(manifest)` | Re-verify, back up, swap, **hot-reload** the engine. |
| `rollback_updates(component)` | Restore the previous version. |
| `get_update_status()` | Installed `(component, version)` list. |

On install, the orchestrator reloads the running engine from the new file:
`SignatureDatabase` → `SignatureDatabase::load_file`; `YaraRules` →
`RuleManager::add_source` + `compile_rules`. The engine crates themselves are
unchanged.

## Performance

Benchmark (`cargo bench -p aegis-update --bench update_bench`, release, 8 MiB
payload, `LocalFetcher`):

| Metric | Value |
|--------|-------|
| Download + verify (sha256 + Ed25519) | 17.0 ms (~470 MiB/s) |
| Install (re-verify + backup + swap + persist) | 17.6 ms |
| SHA-256 throughput | ~1,725 MiB/s |

Ed25519 verification is constant-time and negligible next to hashing; the gate is
dominated by reading + hashing the payload.

## Testing

- **Unit** — version compare/anti-rollback, SHA-256 known-answer, key encoding,
  local fetch + resume, schedule due-logic.
- **Integration** — signed download→install→record; **tampered payload**
  rejected (hash mismatch); **invalid signature** rejected; **rollback** restores
  the prior version; **anti-rollback** rejects older versions; **min-app**
  blocks install.
- **Service** — full update flow through the orchestrator with engine reload.

`cargo test -p aegis-update`: **17/17 pass**. `cargo test -p aegis-service`:
12/12. `cargo clippy --workspace --exclude aegis-tauri … -D warnings`: clean.

> **Build note:** the crate embeds an `asInvoker` manifest (via `build.rs`) into
> its test/bench binaries. Windows UAC installer-detection otherwise
> auto-elevates any executable whose name contains "update", which made the test
> binary fail to launch (`os error 740`).

## Limitations

- **Pinned key is config-provided** — there is no production key in the repo; a
  release pins its own. Key rotation/revocation is future work.
- **No delta updates** — full-component payloads only.
- **Manifest feed transport is out of scope** — the service is handed the
  manifest list; fetching/validating the feed index itself comes later.
- **Reqwest path is not exercised in CI** — tests use `LocalFetcher` to stay
  offline and deterministic.
