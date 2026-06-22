import { create } from "zustand";
import { getServiceStatus, startScan } from "../lib/ipc";
import type { ProtectionStatus, ScanMode } from "../types/ipc";

interface SecurityState {
  status: ProtectionStatus | null;
  statusError: string | null;
  activeScanId: string | null;
  refreshStatus: () => Promise<void>;
  requestScan: (mode: ScanMode) => Promise<void>;
}

export const useSecurityStore = create<SecurityState>((set) => ({
  status: null,
  statusError: null,
  activeScanId: null,
  async refreshStatus() {
    try {
      const status = await getServiceStatus();
      set({ status, statusError: null });
    } catch (error) {
      set({ status: null, statusError: error instanceof Error ? error.message : String(error) });
    }
  },
  async requestScan(mode) {
    const result = await startScan({ mode, roots: [] });
    set({ activeScanId: result.scanId });
  }
}));
