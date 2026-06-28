import { create } from "zustand";
import { api } from "../lib/api";
import { msg } from "./healthStore";
import type { QuarantineRecord } from "../types/ipc";

interface QuarantineState {
  items: QuarantineRecord[];
  loading: boolean;
  error: string | null;
  load: () => Promise<void>;
  restore: (id: string) => Promise<void>;
  remove: (id: string) => Promise<void>;
}

export const useQuarantineStore = create<QuarantineState>((set, get) => ({
  items: [],
  loading: false,
  error: null,
  async load() {
    set({ loading: true });
    try {
      set({ items: await api.quarantine.list(), error: null, loading: false });
    } catch (e) {
      set({ items: [], error: msg(e), loading: false });
    }
  },
  async restore(id) {
    try {
      await api.quarantine.restore(id);
      await get().load();
    } catch (e) {
      set({ error: msg(e) });
    }
  },
  async remove(id) {
    try {
      await api.quarantine.delete(id);
      await get().load();
    } catch (e) {
      set({ error: msg(e) });
    }
  },
}));
