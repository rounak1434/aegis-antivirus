export type ServiceHealth = "starting" | "running" | "degraded" | "stopped";

export type ScanMode = "quick" | "full" | "deep" | "custom";

export interface ProtectionStatus {
  health: ServiceHealth;
  realTimeProtection: boolean;
  fileMonitor: boolean;
  processMonitor: boolean;
  scheduledScans: boolean;
  signatureVersion: string | null;
  lastServiceHeartbeatUtc: string | null;
}

export interface StartScanCommand {
  mode: ScanMode;
  roots: string[];
}

export interface StartScanResult {
  scanId: string;
  acceptedAtUtc: string;
}
