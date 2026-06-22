# Aegis Antivirus Roadmap

## Phase 1: Foundation And Architecture

Status: in progress

Scope:

- Create architecture documentation.
- Create native Tauri v2 + React + TypeScript scaffold.
- Create Rust workspace.
- Create `AegisService` Windows service skeleton.
- Create SQLite migration system.
- Create IPC contracts.
- Preserve the existing HTML prototype as design reference.

Exit criteria:

- Project opens as a native Tauri project once Rust and npm dependencies are installed.
- Service/UI boundary is represented in code and documentation.
- Database schema can be migrated by the service layer.
- No placeholder detection code exists.

## Phase 2: Scanner Engine

Scope:

- Scan job orchestration.
- Quick, Full, Deep, and Custom scan modes.
- File traversal with hidden/system file support.
- Cancellation and progress events.
- Fixture-backed tests.

## Phase 3: Detection Engine

Scope:

- SHA256 and MD5 matching.
- YARA-X integration.
- Entropy analysis.
- Suspicious extensions and double-extension detection.
- Packed executable indicators.
- Script and PowerShell abuse indicators.
- Threat scoring model.

## Phase 4: Windows Security Scanner

Scope:

- Startup folders.
- Registry Run and RunOnce keys.
- Scheduled Tasks.
- Services.
- Drivers.
- Browser extensions.
- Hosts file.

## Phase 5: Real-Time Protection

Scope:

- File create/modify/delete monitoring.
- Process launch monitoring.
- Alert generation.
- Service-managed scheduled scans.

## Phase 6: Quarantine And Reporting

Scope:

- Encrypted quarantine vault.
- Restore/delete operations.
- Audit logging.
- Scan reports.
- Exportable report formats.

## Phase 7: Packaging And Release

Scope:

- Windows installer.
- Code signing.
- Secure updates.
- CI.
- Release hardening.
