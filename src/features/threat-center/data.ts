/**
 * Threat seed data mirroring design-prototype/threats.html. Replaced by the
 * threat Zustand store (Phase C) / IPC ListThreats (Phase D).
 */
export type ThreatSeverity = "malicious" | "suspicious" | "pua";

export interface ThreatLayer {
  layer: string;
  detail: string;
  tone: "danger" | "warn" | "info";
}

export interface ThreatRow {
  name: string;
  family: string;
  severity: ThreatSeverity;
  score: number;
  file: string;
  path: string;
  sha: string;
  size: string;
  signed: string;
  layers: ThreatLayer[];
  rec: string;
  reasons: string;
}

export const THREATS: ThreatRow[] = [
  {
    name: "Trojan:Win32/Wacatac.B!ml",
    family: "Wacatac",
    severity: "malicious",
    score: 92,
    file: "invoice_2024.exe",
    path: "C:\\Users\\admin\\Downloads\\invoice_2024.exe",
    sha: "a3f9...e21c · 64-bit PE",
    size: "2.4 MB",
    signed: "No — unsigned",
    layers: [
      { layer: "Signature", detail: "SHA-256 match", tone: "danger" },
      { layer: "Heuristic", detail: "suspicious imports (VirtualAllocEx, WriteProcessMemory)", tone: "warn" },
      { layer: "ML", detail: "score 0.96 · packed, high entropy", tone: "danger" }
    ],
    rec: "Quarantine immediately. This file matches a known trojan signature and exhibits process-injection capability.",
    reasons: "Packed executable with entropy 7.91/8. Imports classic injection APIs and resolves them dynamically. Hash matches the Wacatac family in the local signature DB. Downloaded 4h ago, not yet executed."
  },
  {
    name: "Heuristic:PowerShell/Obfuscated",
    family: "Script",
    severity: "suspicious",
    score: 74,
    file: "update.ps1",
    path: "C:\\Users\\admin\\AppData\\Local\\Temp\\update.ps1",
    sha: "7b22...10af · text/script",
    size: "14 KB",
    signed: "N/A — script",
    layers: [
      { layer: "Heuristic", detail: "base64 + IEX download cradle", tone: "warn" },
      { layer: "Behavioral", detail: "spawned by scheduled task", tone: "warn" }
    ],
    rec: "Review before removing. Obfuscated PowerShell is often malicious but can be legitimate automation — confirm the source.",
    reasons: "Contains base64-encoded payload piped into Invoke-Expression and a remote download cradle. Launched by an unsigned scheduled task. No signature match, but the pattern is a strong abuse indicator."
  },
  {
    name: "Trojan:Win32/Masquerade",
    family: "Masquerade",
    severity: "malicious",
    score: 81,
    file: "setup.scr (in archive.zip)",
    path: "D:\\Backups\\archive.zip › setup.scr",
    sha: "c901...77be · PE inside ZIP",
    size: "880 KB",
    signed: "No — unsigned",
    layers: [
      { layer: "Signature", detail: "rule: double-extension + icon spoof", tone: "danger" },
      { layer: "Heuristic", detail: "PE masquerading as document", tone: "warn" }
    ],
    rec: "Quarantine the archive entry. A screensaver executable disguised with a document icon and double extension.",
    reasons: "Found nested inside archive.zip during recursive extraction. Uses a .scr executable extension behind a spoofed PDF icon and a \"setup.pdf.scr\" style name. Matches the masquerade heuristic rule set."
  }
];

export function sevPill(s: ThreatSeverity): "danger" | "warn" | "info" {
  return s === "malicious" ? "danger" : s === "suspicious" ? "warn" : "info";
}

export function sevColor(n: number): string {
  return n >= 85 ? "var(--danger)" : n >= 60 ? "var(--warn)" : "var(--info)";
}
