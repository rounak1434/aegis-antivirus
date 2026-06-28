import { create } from "zustand";
import { api } from "../lib/api";
import { msg } from "./healthStore";

export type SettingsValues = Record<string, boolean | string | number>;

interface SettingsState {
  values: SettingsValues;
  loading: boolean;
  saving: boolean;
  dirty: boolean;
  error: string | null;
  load: () => Promise<void>;
  set: (key: string, value: boolean | string | number) => void;
  save: () => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  values: {},
  loading: false,
  saving: false,
  dirty: false,
  error: null,
  async load() {
    set({ loading: true });
    try {
      const json = await api.settings.load();
      set({ values: JSON.parse(json) as SettingsValues, dirty: false, error: null, loading: false });
    } catch (e) {
      set({ error: msg(e), loading: false });
    }
  },
  set(key, value) {
    set((s) => ({ values: { ...s.values, [key]: value }, dirty: true }));
  },
  async save() {
    set({ saving: true });
    try {
      await api.settings.save(JSON.stringify(get().values));
      set({ saving: false, dirty: false, error: null });
    } catch (e) {
      set({ saving: false, error: msg(e) });
    }
  },
}));
