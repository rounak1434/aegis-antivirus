# CI / CD — Continuous Integration & Quality Gates

Every push to `main` and every pull request runs the GitHub Actions pipeline.
A PR is mergeable only when all gates are green.

## Workflows (`.github/workflows/`)

| Workflow | Trigger | Runs |
|----------|---------|------|
| `ci.yml` | push to `main`, every PR | Orchestrator — fans out to the three reusable workflows below, then a `gate` job that succeeds only if all pass. |
| `rust.yml` | reusable / manual | `cargo fmt --check`, `clippy -D warnings`, `test`, `build` (Windows/MSVC). |
| `frontend.yml` | reusable / manual | `npm ci`, `npm run build` (tsc + vite), `npm test` (vitest). |
| `security.yml` | reusable / manual | `cargo-deny`, `cargo-audit`, dependency review (PRs), gitleaks secret scan. |
| `release.yml` | tag `v*`, manual | **Draft-only** GitHub release with auto-generated notes (no packaging/signing). |

`ci.yml` uses reusable workflows (`uses: ./.github/workflows/<name>.yml`) and a
`concurrency` group that cancels superseded runs.

## Rust pipeline (`rust.yml`)

Runs on `windows-latest` (Aegis targets the MSVC toolchain):

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --exclude aegis-tauri -- -D warnings
cargo test --workspace --exclude aegis-tauri
cargo build --workspace --exclude aegis-tauri
```

`aegis-tauri` is excluded from the per-PR gate: it requires the full
Tauri/WebView2 + icon bundle build, which belongs in the (future) packaging
workflow, not on every commit.

**Caching:** `Swatinem/rust-cache@v2` caches `~/.cargo/registry`, `~/.cargo/git`,
and `./target` keyed by the lockfile.

## Frontend pipeline (`frontend.yml`)

Runs on `ubuntu-latest`:

```
npm ci
npm run build
npm test
```

**Caching:** `actions/setup-node@v4` with `cache: npm` (keyed by `package-lock.json`).

## Security pipeline (`security.yml`)

| Check | Tool |
|-------|------|
| RustSec advisories + licenses + bans + sources | `cargo-deny` (`deny.toml`) |
| RustSec advisories | `cargo-audit` |
| New-dependency review | `actions/dependency-review-action` (PRs only) |
| Secret scanning | `gitleaks` |

`deny.toml` pins an allowed-license list (with `ring` clarified) and denies
unknown registries/sources.

## Quality gates

A pull request **fails** if any of these fail:

- `cargo fmt --check`
- `cargo clippy -D warnings`
- `cargo test`
- `npm run build`
- `npm test`
- security audit (`cargo-deny` / `cargo-audit`) or secret scan

The `gate` job in `ci.yml` depends on `rust`, `frontend`, and `security`, so it is
green only when all three succeed — wire this as the required status check in the
branch-protection rule for `main`.

## Dependabot (`.github/dependabot.yml`)

Weekly update PRs for **cargo**, **npm**, and **github-actions**, labelled and
prefixed per ecosystem.

## CODEOWNERS / PR template

`.github/CODEOWNERS` assigns default + engine/CI/security reviewers.
`.github/pull_request_template.md` requires the local gate commands, tests, and
docs before review.

## Running locally

The pipeline mirrors the commands you already run:

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --exclude aegis-tauri -- -D warnings
cargo test --workspace --exclude aegis-tauri
npm ci && npm run build && npm test
```

YAML was validated with `npx js-yaml` for every workflow + template.

## Limitations

- `aegis-tauri` (the desktop binary) is not built in CI yet — it needs the
  packaging workflow (WebView2 + icon bundle), which is a later phase.
- `release.yml` is **draft-only**: no installer build, MSI packaging, or code
  signing (those are explicitly out of scope until the packaging phase).
- Secret-scanning and audit actions run with the default `GITHUB_TOKEN`; some
  org-level features (private vulnerability reporting, etc.) are repo settings,
  not workflow code.
