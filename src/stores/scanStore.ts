import { create } from "zustand";
import { api } from "../lib/api";
import { msg } from "./healthStore";
import type { JobState, ScanMode } from "../types/ipc";

interface ScanState {
  activeJobId: string | null;
  job: JobState | null;
  jobs: JobState[];
  starting: boolean;
  error: string | null;
  start: (mode: ScanMode, roots: string[]) => Promise<void>;
  stop: () => Promise<void>;
  poll: () => Promise<void>;
  refreshJobs: () => Promise<void>;
}

export const useScanStore = create<ScanState>((set, get) => ({
  activeJobId: null,
  job: null,
  jobs: [],
  starting: false,
  error: null,
  async start(mode, roots) {
    set({ starting: true, error: null });
    try {
      const activeJobId = await api.scan.start(mode, roots);
      set({ activeJobId, starting: false });
      await get().poll();
    } catch (e) {
      set({ error: msg(e), starting: false });
    }
  },
  async stop() {
    const id = get().activeJobId;
    if (!id) return;
    try {
      await api.scan.stop(id);
      await get().poll();
    } catch (e) {
      set({ error: msg(e) });
    }
  },
  async poll() {
    const id = get().activeJobId;
    if (!id) return;
    try {
      const job = await api.scan.status(id);
      set({ job, error: null });
    } catch (e) {
      set({ error: msg(e) });
    }
  },
  async refreshJobs() {
    try {
      set({ jobs: await api.scan.jobs(), error: null });
    } catch (e) {
      set({ error: msg(e) });
    }
  },
}));
