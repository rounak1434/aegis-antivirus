/**
 * Typed IPC wrappers grouped by domain. This is the UI's entire surface onto
 * AegisService — every screen/store calls `api.*`, never `invoke` directly.
 */
import { call } from "./ipc";
import type {
  InstallOutcome,
  InstalledComponent,
  JobState,
  ProtectionMode,
  QuarantineRecord,
  RealtimeStatus,
  ScanMode,
  ServiceHealth,
  ThreatDetection,
  UpdateManifest,
} from "../types/ipc";

export const health = {
  get: () => call<ServiceHealth>("get_service_health"),
};

export const scan = {
  start: (mode: ScanMode, roots: string[]) => call<string>("start_scan", { mode, roots }),
  stop: (jobId: string) => call<boolean>("stop_scan", { jobId }),
  status: (jobId: string) => call<JobState | null>("get_scan_status", { jobId }),
  jobs: () => call<JobState[]>("list_jobs"),
};

export const threats = {
  list: () => call<ThreatDetection[]>("list_threats"),
  quarantine: (detection: ThreatDetection) =>
    call<QuarantineRecord>("quarantine_detection", { detection }),
};

export const quarantine = {
  list: () => call<QuarantineRecord[]>("list_quarantine"),
  restore: (id: string, dest?: string) => call<string>("restore_file", { id, dest: dest ?? null }),
  delete: (id: string) => call<void>("delete_quarantine_item", { id }),
};

export const windows = {
  scan: () => call<ThreatDetection[]>("run_windows_scan"),
};

export const realtime = {
  start: (mode: ProtectionMode) => call<void>("start_realtime", { mode }),
  stop: () => call<void>("stop_realtime"),
  status: () => call<RealtimeStatus>("get_realtime_status"),
};

export const updates = {
  check: (available: UpdateManifest[]) => call<UpdateManifest[]>("check_updates", { available }),
  download: (manifest: UpdateManifest) => call<void>("download_updates", { manifest }),
  install: (manifest: UpdateManifest) => call<InstallOutcome>("install_updates", { manifest }),
  rollback: (component: string) => call<InstallOutcome>("rollback_updates", { component }),
  status: () => call<InstalledComponent[]>("get_update_status"),
};

export const settings = {
  load: () => call<string>("load_settings"),
  save: (settingsJson: string) => call<void>("save_settings", { settings: settingsJson }),
};

export const api = { health, scan, threats, quarantine, windows, realtime, updates, settings };
export default api;
