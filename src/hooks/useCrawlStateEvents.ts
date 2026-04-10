/**
 * Global listener for crawl state change events from the Rust backend.
 *
 * Subscribes to `crawl://state` events emitted when a crawl transitions
 * between states (running, paused, completed, stopped). Updates the crawl
 * store and shows toast notifications for terminal states.
 *
 * Mount once at the App level so state changes are detected regardless
 * of which view the user is on. Do not duplicate in child components.
 */

import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useCrawlStore } from "@/stores/crawlStore";
import { toast } from "sonner";
import type { CrawlStateChange } from "@/types";

export function useCrawlStateEvents() {
  const activeCrawlId = useCrawlStore((s) => s.activeCrawlId);
  const setCrawlState = useCrawlStore((s) => s.setCrawlState);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;
    let cancelled = false;

    const subscribe = async () => {
      const fn = await listen<CrawlStateChange>("crawl://state", (event) => {
        const { crawlId, state } = event.payload;

        // Only process events for the active crawl.
        if (crawlId !== activeCrawlId) return;

        // Deduplicate: skip if the state hasn't actually changed.
        const currentState = useCrawlStore.getState().state;
        if (state === currentState) return;

        setCrawlState(state);

        if (state === "completed") {
          toast.success("Crawl completed successfully.");
        } else if (state === "error") {
          toast.error("Crawl encountered an error.");
        } else if (state === "stopped") {
          toast.info("Crawl stopped.");
        }
      });

      if (cancelled) {
        fn(); // Already cleaned up -- immediately unsubscribe.
      } else {
        unlisten = fn;
      }
    };

    subscribe().catch((err) => {
      console.error("Failed to subscribe to crawl state events:", err);
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [activeCrawlId, setCrawlState]);
}
