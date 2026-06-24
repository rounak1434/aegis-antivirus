# Security Policy

Aegis Antivirus is a security tool, and we take the security of the project
itself seriously. Thank you for helping keep Aegis and its users safe.

## Supported Versions

Aegis is pre-1.0 and under active development. Security fixes are applied to the
latest `main` branch only.

| Version | Supported |
|---------|-----------|
| `main` (latest) | ✅ |
| pre-release tags | ❌ |

## Reporting a Vulnerability

**Please do not open a public issue for security vulnerabilities.**

Report privately via one of:

- GitHub **Security Advisories** → *Report a vulnerability* on this repository
  (preferred), or
- a direct message to the maintainer.

Include, where possible:

- a description of the vulnerability and its impact,
- steps to reproduce or a proof of concept,
- affected component/crate and version/commit,
- any suggested remediation.

## Disclosure Policy

- We follow **coordinated disclosure**.
- Please give us a reasonable window to investigate and ship a fix before any
  public disclosure.
- We will credit reporters in the release notes unless you prefer to remain
  anonymous.

## Response Process

| Stage | Target |
|-------|--------|
| Acknowledge report | within 3 business days |
| Initial assessment / severity | within 7 business days |
| Fix or mitigation plan | tracked in a private advisory |
| Public advisory + patched release | after the fix lands |

## Scope

In scope: the Aegis source in this repository (Rust crates, Tauri/React UI,
build and migration scripts).

Out of scope: vulnerabilities in third-party dependencies (report upstream;
we will bump the dependency once fixed), and issues requiring a
already-compromised host or physical access.

## Handling of Malicious Samples

This repository contains **no real malware**. Test fixtures generate benign,
harmless markers (e.g. EICAR-style strings) in temporary directories at runtime.
Never commit real malware samples, quarantine vault data, or vault keys —
`.gitignore` is configured to block them.
