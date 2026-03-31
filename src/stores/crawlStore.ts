/**
 * Crawl state store — manages active crawl lifecycle and progress.
 *
 * Consumed by CrawlMonitor, Dashboard, and the Sidebar status indicator.
 */

import { create } from "zustand";
import type { CrawlConfig, CrawlProgress, CrawlState } from "@/types";

interface CrawlStore {
  /** Currently active crawl ID, or null if no crawl is running. */
  activeCrawlId: string | null;

  /** Current crawl state. */
  state: CrawlState | null;

  /** Latest progress snapshot from Tauri events. */
  progress: CrawlProgress | null;

  /** The config used for the active crawl. */
  config: CrawlConfig | null;

  // --- Actions ---
  setCrawlStarted: (crawlId: string, config: CrawlConfig) => void;
  updateProgress: (progress: CrawlProgress) => void;
  setCrawlState: (state: CrawlState) => void;
  clearCrawl: () => void;
}

export const useCrawlStore = create<CrawlStore>((set) => ({
  activeCrawlId: null,
  state: null,
  progress: null,
  config: null,

  setCrawlStarted: (crawlId, config) =>
    set({
      activeCrawlId: crawlId,
      state: "running",
      config,
      progress: null,
    }),

  updateProgress: (progress) =>
    set({ progress }),

  setCrawlState: (state) =>
    set({ state }),

  clearCrawl: () =>
    set({
      activeCrawlId: null,
      state: null,
      progress: null,
      config: null,
    }),
}));
