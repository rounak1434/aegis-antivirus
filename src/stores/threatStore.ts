import { create } from "zustand";
import { api } from "../lib/api";
import { msg } from "./healthStore";
import type { ThreatDetection } from "../types/ipc";

interface ThreatState {
  items: ThreatDetection[];
  loading: boolean;
  error: string | null;
  load: () => Promise<void>;
  quarantine: (detection: ThreatDetection) => Promise<void>;
}

export const useThreatStore = create<ThreatState>((set, get) => ({
  items: [],
  loading: false,
  error: null,
  async load() {
    set({ loading: true });
    try {
      set({ items: await api.threats.list(), error: null, loading: false });
    } catch (e) {
      set({ items: [], error: msg(e), loading: false });
    }
  },
  async quarantine(detection) {
    try {
      await api.threats.quarantine(detection);
      await get().load();
    } catch (e) {
      set({ error: msg(e) });
    }
  },
}));
