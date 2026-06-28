/**
 * Typed mirrors of the Rust DTOs returned by AegisService commands. Field names
 * are snake_case to match serde's default serialization exactly — do not rename.
 */

export type ScanMode = "quick" | "full" | "deep" | "custom";
export type ThreatLevel = "safe" | "low" | "medium" | "high" | "critical";
export type ComponentStatus = "ok" | "degraded" | "unavailable";
export type ProtectionMode = "monitor_only" | "notify_only" | "auto_quarantine";
export type JobType = "file_scan" | "windows_scan";
export type JobStatus = "queued" | "running" | "completed" | "cancelled" | "failed";
export type RealtimeAction = "monitored" | "notified" | "quarantined" | "quarantine_failed";
export type UpdateComponent =
  | "signature_database"
  | "yara_rules"
  | "threat_metadata"
  | "engine_config";

export interface ServiceHealth {
  scanner: ComponentStatus;
  database: ComponentStatus;
  rules: ComponentStatus;
  quarantine: ComponentStatus;
  active_jobs: number;
  overall: ComponentStatus;
}

export interface ScanProgress {
  files_scanned: number;
  total_files: number;
  bytes_scanned: number;
  errors: number;
  percent: number;
  elapsed_ms: number;
  files_per_sec: number;
  bytes_per_sec: number;
  eta_ms: number;
  current_path: string;
}

export interface JobState {
  id: string;
  job_type: JobType;
  status: JobStatus;
  roots: string[];
  progress: ScanProgress | null;
  files_scanned: number;
  threats_found: number;
  error: string | null;
  queued_at: string;
  started_at: string | null;
  finished_at: string | null;
}

/** One typed evidence item; carries `kind` plus variant-specific fields. */
export interface ThreatEvidence {
  kind: string;
  [field: string]: unknown;
}

export interface ThreatDetection {
  id: string;
  path: string;
  threat_level: ThreatLevel;
  score: number;
  evidence: ThreatEvidence[];
  timestamp: string;
}

export interface QuarantineRecord {
  id: string;
  original_path: string;
  quarantine_path: string;
  sha256: string;
  threat_level: ThreatLevel;
  reason: string;
  timestamp: string;
  size: number;
  encrypted: boolean;
  status: string;
}

export interface RealtimeStatus {
  running: boolean;
  mode: ProtectionMode;
  watched_paths: string[];
  events_processed: number;
  alerts_raised: number;
}

export interface UpdateManifest {
  version: string;
  published_at: string;
  sha256: string;
  signature: string;
  url: string;
  size: number;
  component: UpdateComponent;
  minimum_app_version: string;
}

export interface InstallOutcome {
  component: UpdateComponent;
  version: string;
  installed_path: string;
}

/** [component, version] pairs from get_update_status. */
export type InstalledComponent = [string, string];
