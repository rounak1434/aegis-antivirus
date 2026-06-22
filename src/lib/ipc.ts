import { invoke } from "@tauri-apps/api/core";
import type { ProtectionStatus, StartScanCommand, StartScanResult } from "../types/ipc";

export async function getServiceStatus(): Promise<ProtectionStatus> {
  return invoke<ProtectionStatus>("get_service_status");
}

export async function startScan(command: StartScanCommand): Promise<StartScanResult> {
  return invoke<StartScanResult>("start_scan", { command });
}
