<!-- Thanks for contributing to Aegis! Please complete the checklist below. -->

## Summary

<!-- What does this PR change, and why? -->

## Related issue

<!-- e.g. Closes #123 -->

## Type of change

- [ ] Bug fix
- [ ] New feature
- [ ] Refactor / cleanup
- [ ] Documentation
- [ ] Build / CI

## Checklist

CI enforces these on every PR — please run them locally first.

**Required**

- [ ] `cargo fmt --all -- --check` is clean
- [ ] `cargo clippy --workspace --exclude aegis-tauri --all-targets --all-features -- -D warnings` is clean
- [ ] `cargo test --workspace --exclude aegis-tauri` passes
- [ ] `npm run build` (tsc + vite) succeeds *(if the UI changed)*
- [ ] `npm test` (vitest) passes *(if the UI/IPC changed)*
- [ ] **Tests added/updated** for the change
- [ ] **Docs updated** (`ARCHITECTURE.md` / `CHANGELOG.md` / relevant `*.md`)
- [ ] No secrets, vault keys, databases, or real malware committed

**If applicable**

- [ ] New detections carry explainable evidence (typed `ThreatEvidence` + `reason()`)
- [ ] Engine crates not modified (or change justified in the summary)
- [ ] New dependencies pass `cargo deny check` / `cargo audit`
