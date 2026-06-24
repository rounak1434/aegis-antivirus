# Aegis Quarantine System (Phase 4)

The quarantine subsystem (`aegis-quarantine`) receives malicious files — by path
or by [`aegis_detect::ThreatDetection`] — and isolates them in an encrypted
vault. **Plaintext malware is never stored at rest.** Every action is audited;
restores are integrity-checked and path-validated.

## Vault Layout

```
data/quarantine/
  vault.key            # 256-bit AES key (generated on first open)
  <uuid>.qbin          # one encrypted blob per quarantined file
```

- **Randomized filenames** — each item is `<uuid-v4>.qbin`; collision-safe.
- **Metadata** lives in SQLite (`quarantine_records`), not in the vault dir.
- The original file is **shredded** (zero-overwrite) and removed after isolation.

## Encryption (AES-256-GCM)

On-disk format per file: `[12-byte random nonce][ciphertext + 16-byte GCM tag]`.

- A fresh random nonce per encryption → identical plaintext yields different
  ciphertext.
- GCM authenticates: any tampering with the blob fails decryption.
- The 256-bit key is generated with the OS CSPRNG on first vault open and stored
  as `vault.key`.

> **Key-management limitation.** Storing the key beside the vault guarantees
> malware is never at rest in plaintext and stops casual access, but it is not
> OS-backed key protection. Hardening step (future): wrap `vault.key` with
> Windows DPAPI / TPM and tighten the vault-dir ACL to the service account.

## Quarantine Record

`QuarantineRecord` (persisted in `quarantine_records`):

| Field | Meaning |
|-------|---------|
| `id` | UUID v4 |
| `original_path` | where the file came from |
| `quarantine_path` | the `<uuid>.qbin` vault file |
| `sha256` | digest of the **plaintext** (integrity anchor) |
| `threat_level` | `aegis_common::ThreatLevel` |
| `reason` | why it was quarantined (evidence labels) |
| `timestamp` | when quarantined (UTC) |
| `size` | original plaintext size (bytes) |
| `encrypted` | always `true` |
| `status` | `quarantined` \| `restored` \| `deleted` |

## Actions (`Vault`)

- `quarantine_file(path, threat_level, reason, actor)` — read → SHA-256 →
  encrypt → write vault blob → shred original → insert record → audit.
- `quarantine_detection(&ThreatDetection, actor)` — convenience over the above.
- `restore_file(id, dest, actor)` — verify record + status, decrypt, verify
  SHA-256, validate target path, write, shred vault copy, audit.
- `delete_file(id, actor)` — shred vault blob, mark deleted, audit.
- `get_record(id)` / `list_records()`.

## Safety

Before a restore:
1. **Record exists** and is still `quarantined` (else `NotFound` / `NotInVault`).
2. **Integrity**: decrypted bytes are re-hashed and must equal the recorded
   SHA-256 (else `IntegrityMismatch`). GCM tampering fails earlier at decrypt.
3. **Path validation** on the restore target:
   - must be **absolute** (no relative paths),
   - must contain no `..` component (**path-traversal** guard),
   - parent directory must exist,
   - target must not already exist (**overwrite** guard → `TargetExists`).

Each guard has a dedicated test (`path_traversal_and_overwrite_rejected`,
`integrity_mismatch_blocks_restore`, `tampered_ciphertext_fails_to_decrypt`,
`double_restore_blocked`).

## Database (migration `003_quarantine.sql`)

- `quarantine_records` — full record + `restored_at_utc` / `deleted_at_utc`,
  indexed by `status` and `sha256`.
- `audit_log` (reused from `001`) — one row per quarantine / restore / delete,
  storing `actor`, `action`, `subject` (record id), `details_json` (result),
  `created_at_utc`.

`aegis-db::apply_migrations` applies the ordered list 001 → 002 → 003 idempotently.

## Audit Trail

Every action logs: action, timestamp, actor (user), subject (record id), and
result (`ok` / `integrity_mismatch` / …) to `audit_log`.

## Performance

Benchmark (`cargo bench -p aegis-quarantine --bench quarantine_bench`, release,
1,000 files × 64 KiB = 62.5 MiB):

| Operation | Time | files/sec | MiB/sec |
|-----------|-----:|----------:|--------:|
| Quarantine | 1,981 ms | 505 | 31.5 |
| Restore | 1,850 ms | 541 | 33.8 |
| AES-256-GCM (encryption only) | 47 ms | — | 1,323.8 |

Encryption is ~2% of the cost; quarantine/restore are dominated by per-file disk
I/O (read + write + shred + SQLite insert/update). Encryption is **not** the
bottleneck.

## Limitations

- **Key stored beside the vault** (see Encryption note) — DPAPI/TPM wrapping is
  the planned hardening.
- **Shred is a single zero-pass** — defeats casual recovery; not a
  forensic-grade multi-pass wipe, and on SSDs wear-levelling can retain remnants.
- **Whole-file in memory** — quarantine reads the entire file into RAM before
  encrypting; very large files need a streaming path.
- **Single connection, not concurrent** — the vault holds one SQLite connection;
  concurrent callers must serialize (the service owns one vault instance).
- **Restore defaults to the original path** — callers wanting a different
  location must pass a validated absolute `dest`.
