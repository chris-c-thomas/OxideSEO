/**
 * Settings store — persists app preferences and default crawl config.
 */

import { create } from "zustand";
import type { AppSettings } from "@/types";

interface SettingsStore {
  settings: AppSettings | null;
  isLoading: boolean;
  setSettings: (settings: AppSettings) => void;
  setLoading: (loading: boolean) => void;
}

export const useSettingsStore = create<SettingsStore>((set) => ({
  settings: null,
  isLoading: true,

  setSettings: (settings) => set({ settings, isLoading: false }),
  setLoading: (isLoading) => set({ isLoading }),
}));
