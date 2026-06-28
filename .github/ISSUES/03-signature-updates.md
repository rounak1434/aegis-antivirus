---
title: "Signature & YARA rule updates (signed, atomic, rollback)"
labels: ["enhancement", "engine", "security"]
---

## Objective

Deliver and apply signature/rule bundle updates safely, so detection stays
current without a full app update.

## Requirements

- Update transport over TLS with a pinned signing key; verify the bundle
  signature before applying.
- Atomic apply with rollback on failure; record `update_history`.
- Feed the in-memory `SignatureDatabase` and `RuleManager` from the applied
  bundle (reuse `aegis-signatures` / `aegis-yara`; no engine rewrite).
- Configurable channel (stable/beta) and auto-update interval, owned by the
  service.

## Acceptance Criteria

- [ ] Tampered/unsigned bundles are rejected (covered by tests).
- [ ] Apply is atomic; a simulated mid-apply failure rolls back cleanly.
- [ ] `update_history` rows are written; `get_service_health` reflects rule state.
- [ ] `cargo test` + `cargo clippy … -D warnings` pass; docs updated.
