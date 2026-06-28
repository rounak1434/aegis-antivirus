---
title: "Authenticode code signing for binaries + installer"
labels: ["enhancement", "packaging", "security"]
---

## Objective

Sign the Aegis binaries and installer so Windows SmartScreen/UAC trust them and
the service can enforce signed-client IPC.

## Requirements

- Authenticode signing of the UI binary, `AegisService`, and the installer in
  the release pipeline.
- Signing integrated into the release build scripts; secrets sourced from CI
  secrets, never committed.
- Document the certificate requirements (EV/OV) and the local + CI signing flow.
- Enable the "reject unsigned IPC clients" path so only the signed UI talks to
  the service (ties into the tamper-protection setting).

## Acceptance Criteria

- [ ] Release artifacts are signed; signature verifies with `signtool verify`.
- [ ] No signing material is committed; CI uses secrets.
- [ ] Signing process documented for maintainers.
- [ ] Service can reject unsigned IPC clients when enabled.
