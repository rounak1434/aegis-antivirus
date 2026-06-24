# Aegis Detection Engine (Phase 3)

The detection engine sits **above** the verified `aegis-scan` output and turns
scanned files into explainable threat detections. It is composed of three
crates plus the shared threat model.

## Crates

| Crate | Responsibility |
|-------|----------------|
| `aegis-signatures` | Hash signature database (SHA-256 / MD5): SQLite + local file sources + in-memory cache. |
| `aegis-yara` | YARA-X rule manager: load / validate / compile / cache / scan. |
| `aegis-detect` | Threat model, heuristics, risk scoring, evidence, orchestration, and persistence. |

`aegis-detect` re-exports the layer types, so callers depend on it alone:
`SignatureDatabase`, `RuleManager`, `DetectionEngine`, `ThreatDetection`,
`ThreatEvidence`, `ThreatLevel`.

## Architecture & Data Flow

```text
aegis-scan ──► ScanReport { files: [ScannedFile{ metadata, hashes, error }] }
                     │
                     ▼
            DetectionEngine::analyze(file, &SignatureDatabase, Option<&RuleManager>)
                     │
   ┌─────────────────┼───────────────────────────────┐
   ▼                 ▼                                 ▼
Hash layer        Heuristic layer                  YARA layer
 contains_sha256   double / suspicious ext          scan_bytes(content)
 contains_md5      entropy / packed executable      → YaraMatch[]
                   script / powershell indicators
                     │
                     ▼
            Vec<ThreatEvidence>  ──►  score = Σ weight (clamp 0–100)
                                       level = level_for_score(score)
                     │
                     ▼
            ThreatDetection { id, path, threat_level, score, evidence[], timestamp }
                     │
                     ▼
   persist_detection() → detection_results ;  record_scan_event() → scan_events
```

The engine is **stateless**: the signature DB and optional YARA manager are
passed per call so the caller (the service) owns their lifecycle and reloads.

## Threat Model

- **`ThreatLevel`** (from `aegis-common`): `Safe`, `Low`, `Medium`, `High`, `Critical`.
- **`ThreatEvidence`** (tagged enum, each variant preserves its trigger data):
  `HashMatch`, `YaraMatch`, `EntropyDetection`, `PackedExecutable`,
  `DoubleExtension`, `SuspiciousExtension`, `ScriptIndicator`, `PowerShellIndicator`.
- **`ThreatDetection`**: `id`, `path`, `threat_level`, `score`, `evidence[]`, `timestamp`.

## Risk Scoring

Additive, then clamped to 0–100. Every point traces to one evidence item with a
`reason()` — no black-box scores.

| Evidence | Weight |
|----------|-------:|
| HashMatch | +100 |
| YaraMatch | +80 |
| PowerShellIndicator | +25 |
| DoubleExtension | +20 |
| PackedExecutable | +20 |
| SuspiciousExtension | +15 |
| EntropyDetection | +15 |
| ScriptIndicator | +10 |

Level thresholds: `0–9 Safe · 10–29 Low · 30–59 Medium · 60–84 High · 85–100 Critical`.

## Heuristics

- **Double extension** — `name.<decoy>.<exec>` where decoy ∈ {pdf, docx, jpg, …}
  and exec ∈ executable set (e.g. `invoice.pdf.exe`, `photo.jpg.scr`).
- **Suspicious extension** — final ext ∈ `{scr, pif, com, hta, js, jse, vbs, vbe, wsf, ps1}`.
- **Entropy** — Shannon entropy (0–8). Executable (`MZ` magic or exec ext) with
  entropy ≥ 7.0 → `PackedExecutable`; any file ≥ 7.5 → `EntropyDetection`. Files
  < 256 bytes skip the entropy heuristic (noise control).
- **Script indicators** — content contains `powershell`, `cmd.exe`, `wscript`, `cscript`.
- **PowerShell abuse** — content contains `EncodedCommand`, `DownloadString`,
  `Invoke-Expression`, `IEX`, `Bypass` (case-insensitive).

Content heuristics inspect at most the first **1 MiB** of each file.

## Evidence Model

Detections are explainable. Each `ThreatEvidence` serializes with its data and
exposes `label()` (stable tag), `reason()` (human explanation), and `weight()`.
Stored as `evidence_json` in `detection_results`.

## Database (migration `002_detection.sql`)

- `signature_sets` — named hash/yara feeds (id, name, kind, source, count, updated_at).
- `signatures` — individual hashes (set_id FK, algo, hex, threat_name), `UNIQUE(algo, hex)`.
- `detection_results` — engine output (id, path, threat_level, score, evidence_json, detected_at).
- `scan_events` — lifecycle audit (id, scan_id, event_type, path, detail_json, created_at).

`aegis-db::apply_migrations` now applies an ordered migration list (001 + 002),
recording each in `schema_migrations` and skipping already-applied versions.

## Performance

Benchmark (`cargo bench -p aegis-detect --bench detect_bench`, release,
10,000 files, realistic clean/malicious mix, SHA-256 hash layer + 1 YARA rule +
all heuristics):

| Metric | Value |
|--------|-------|
| Files | 10,000 |
| Scan phase | 171 ms |
| Detect phase | 6,074 ms |
| Detect throughput | 1,646 files/sec |
| Detect latency | 607 µs/file |
| Detections | 800 |
| Peak working set | 27.5 MiB |

The detect phase is single-threaded and dominated by per-file YARA scanning
(a fresh `Scanner` per file). Throughput is the headroom item — see Limitations.

## Limitations

- **Detection is single-threaded** — unlike the rayon-parallel scanner. A
  rayon pass over `ScanReport.files` is the obvious next optimization; YARA
  `Scanner` is not `Sync`, so each worker needs its own (cheap to construct).
- **Per-file `Scanner` allocation** — fine for correctness, costs throughput;
  pooling scanners per thread would cut the 607 µs/file latency.
- **Content heuristics cap at 1 MiB** — abuse strings past that offset are missed.
- **Hash layer needs digests from the scanner** — files the scanner could not
  hash (locked, symlink) get no hash-layer coverage.
- **`threat_name` on hash matches is `None`** until the SQLite `signatures`
  table (which carries `threat_name`) feeds the in-memory cache with names.
- **Entropy is whole-head, not sectioned** — a small encrypted blob inside a
  large benign file can be diluted below threshold.
- **No archive/Office-macro introspection yet** — out of Phase 3 scope.
