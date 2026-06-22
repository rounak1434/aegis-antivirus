# Development Setup

## Required Tooling

Aegis Antivirus is a native Windows desktop application. Install these before building locally:

- Rust stable toolchain with Cargo: https://rustup.rs/
- Node.js 20 LTS or newer.
- Microsoft Visual Studio Build Tools with the Desktop development with C++ workload.
- WebView2 Runtime for Tauri desktop execution.

## Current Machine Note

During Phase 1 scaffolding, Node.js and npm were available, but `cargo` and `rustc` were not found on PATH. Rust validation must be run after installing Rust.

## Validation Commands

```powershell
npm install
npm run build
cargo check --workspace
cargo test --workspace
npm run tauri dev
```

## Service Development

`AegisService` is a Windows service. Service installation, service ACLs, and signed release packaging are intentionally deferred until the packaging phase. During early development, validate the service crate with `cargo check -p aegis-service` before adding installer behavior.
