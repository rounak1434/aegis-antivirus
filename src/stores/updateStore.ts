import { create } from "zustand";
import { api } from "../lib/api";
import { msg } from "./healthStore";
import type { InstalledComponent, UpdateManifest } from "../types/ipc";

interface UpdateState {
  installed: InstalledComponent[];
  available: UpdateManifest[];
  loading: boolean;
  checking: boolean;
  installing: string | null;
  error: string | null;
  lastAction: string | null;
  load: () => Promise<void>;
  check: (feed: UpdateManifest[]) => Promise<void>;
  install: (manifest: UpdateManifest) => Promise<void>;
  rollback: (component: string) => Promise<void>;
}

export const useUpdateStore = create<UpdateState>((set, get) => ({
  installed: [],
  available: [],
  loading: false,
  checking: false,
  installing: null,
  error: null,
  lastAction: null,
  async load() {
    set({ loading: true });
    try {
      set({ installed: await api.updates.status(), error: null, loading: false });
    } catch (e) {
      set({ installed: [], error: msg(e), loading: false });
    }
  },
  async check(feed) {
    set({ checking: true });
    try {
      set({ available: await api.updates.check(feed), error: null, checking: false });
    } catch (e) {
      set({ available: [], error: msg(e), checking: false });
    }
  },
  async install(manifest) {
    set({ installing: manifest.component });
    try {
      await api.updates.download(manifest);
      const outcome = await api.updates.install(manifest);
      set({ installing: null, lastAction: `installed ${outcome.component} ${outcome.version}` });
      await get().load();
    } catch (e) {
      set({ installing: null, error: msg(e) });
    }
  },
  async rollback(component) {
    try {
      const outcome = await api.updates.rollback(component);
      set({ lastAction: `rolled back ${outcome.component} to ${outcome.version}`, error: null });
      await get().load();
    } catch (e) {
      set({ error: msg(e) });
    }
  },
}));
