import { create } from "zustand";
import { api } from "../lib/api";
import type { ServiceHealth } from "../types/ipc";

interface HealthState {
  health: ServiceHealth | null;
  loading: boolean;
  error: string | null;
  load: () => Promise<void>;
}

export const useHealthStore = create<HealthState>((set) => ({
  health: null,
  loading: false,
  error: null,
  async load() {
    set({ loading: true });
    try {
      const health = await api.health.get();
      set({ health, error: null, loading: false });
    } catch (e) {
      set({ health: null, error: msg(e), loading: false });
    }
  },
}));

export function msg(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}
