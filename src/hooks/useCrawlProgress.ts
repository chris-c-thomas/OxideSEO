/**
 * Hook to subscribe to real-time crawl progress events from the Rust backend.
 *
 * The backend emits `crawl://progress` events at ~4Hz during a crawl.
 * This hook manages the event listener lifecycle and updates the crawl store.
 */

import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useCrawlStore } from "@/stores/crawlStore";
import type { CrawlProgress } from "@/types";

/**
 * Subscribe to crawl progress events for the given crawl ID.
 * Automatically unsubscribes on unmount or when crawlId changes.
 */
export function useCrawlProgress(crawlId: string | null) {
  const updateProgress = useCrawlStore((s) => s.updateProgress);

  useEffect(() => {
    if (!crawlId) return;

    let unlisten: UnlistenFn | null = null;

    const subscribe = async () => {
      unlisten = await listen<CrawlProgress>(
        "crawl://progress",
        (event) => {
          if (event.payload.crawlId === crawlId) {
            updateProgress(event.payload);
          }
        },
      );
    };

    subscribe();

    return () => {
      unlisten?.();
    };
  }, [crawlId, updateProgress]);
}
