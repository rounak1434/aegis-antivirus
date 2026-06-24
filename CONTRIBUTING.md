# Contributing to Aegis Antivirus

Thanks for your interest in contributing! Aegis is an open-source Windows
antivirus built in Rust + Tauri. This guide covers setup, standards, testing,
and pull requests.

## Getting Started

### Prerequisites

- Rust stable (1.96+) with the **MSVC** toolchain (`rustup default stable-x86_64-pc-windows-msvc`)
- Visual Studio Build Tools — *Desktop development with C++* workload
- Node.js 20+ and npm
- WebView2 runtime (bundled with Windows 11)

See [`DEVELOPMENT.md`](DEVELOPMENT.md) for detailed toolchain notes.

### Build & run

```bash
git clone https://github.com/rounak1434/aegis-antivirus
cd aegis-antivirus

cargo build --workspace
npm install
npm run tauri dev
```

## Project Layout

The backend is a Cargo workspace of focused crates under `crates/` (see the
Project Structure section in [`README.md`](README.md)). The UI is in `src/`
(React/TypeScript) and `src-tauri/`. SQLite schema lives in `migrations/`.

The architecture is **privilege-separated**: the UI is non-privileged and talks
to the engine only through the typed IPC contracts in `aegis-ipc`. Keep
privileged logic in the engine crates, not the UI.

## Coding Standards

### Rust

- **Format:** `cargo fmt --all` before committing.
- **Lint:** code must pass
  `cargo clippy --workspace --exclude aegis-tauri --all-targets --all-features -- -D warnings`
  with **zero** warnings.
- No `unwrap()`/`expect()` on fallible paths in library code — return typed
  errors (`thiserror`). Tests may `unwrap`.
- Keep detections **explainable**: any new threat signal must carry typed
  evidence with a human-readable `reason()`, never a bare score.
- Match the style of the surrounding code; document public items.

### Frontend

- TypeScript strict mode; no `any` without justification.
- Preserve the prototype's visual design (see `design-prototype/` and
  `PROTOTYPE_AUDIT.md`) — do not introduce a new visual system.

## Testing Requirements

- New code needs tests. Backend: unit tests in-module + integration tests under
  `crates/<crate>/tests/`.
- The full suite must pass:
  ```bash
  cargo test --workspace --exclude aegis-tauri
  ```
- Performance-sensitive changes should include or update a benchmark
  (`crates/<crate>/benches/`).
- Never commit real malware, vault keys, databases, or runtime data — generate
  fixtures at runtime in temp dirs (see existing tests for the pattern).

## Commit & Pull Request Guidelines

- Use clear, imperative commit subjects (e.g. `Add RunOnce registry collector`).
- Keep PRs focused; one logical change per PR.
- Before opening a PR, ensure:
  - [ ] `cargo fmt --all` clean
  - [ ] `cargo clippy … -D warnings` clean
  - [ ] `cargo test --workspace --exclude aegis-tauri` passes
  - [ ] docs updated (`ARCHITECTURE.md` / `CHANGELOG.md` / relevant `*.md`)
- Describe **what** changed and **why**, and link any related issue.
- Maintainers may request changes; please keep the discussion in the PR.

## Reporting Bugs & Requesting Features

Use the GitHub issue templates. For **security** vulnerabilities, do **not** open
a public issue — follow [`SECURITY.md`](SECURITY.md).

## License

By contributing, you agree that your contributions will be licensed under the
[Apache License 2.0](LICENSE).
