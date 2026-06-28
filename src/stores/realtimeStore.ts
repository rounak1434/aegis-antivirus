import { create } from "zustand";
import { api } from "../lib/api";
import { msg } from "./healthStore";
import type { ProtectionMode, RealtimeStatus } from "../types/ipc";

interface RealtimeState {
  status: RealtimeStatus | null;
  loading: boolean;
  error: string | null;
  load: () => Promise<void>;
  start: (mode: ProtectionMode) => Promise<void>;
  stop: () => Promise<void>;
}

export const useRealtimeStore = create<RealtimeState>((set, get) => ({
  status: null,
  loading: false,
  error: null,
  async load() {
    set({ loading: true });
    try {
      set({ status: await api.realtime.status(), error: null, loading: false });
    } catch (e) {
      set({ status: null, error: msg(e), loading: false });
    }
  },
  async start(mode) {
    try {
      await api.realtime.start(mode);
      await get().load();
    } catch (e) {
      set({ error: msg(e) });
    }
  },
  async stop() {
    try {
      await api.realtime.stop();
      await get().load();
    } catch (e) {
      set({ error: msg(e) });
    }
  },
}));
