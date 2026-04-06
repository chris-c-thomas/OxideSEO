/**
 * Hook to subscribe to AI batch analysis progress events.
 *
 * The backend emits `ai://progress` events during batch page analysis.
 */

import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { BatchProgress } from "@/types";

/**
 * Subscribe to AI analysis progress events.
 * Calls `onProgress` with each progress update.
 */
export function useAiProgress(onProgress: (progress: BatchProgress) => void) {
  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    const subscribe = async () => {
      unlisten = await listen<BatchProgress>("ai://progress", (event) => {
        onProgress(event.payload);
      });
    };

    subscribe();

    return () => {
      unlisten?.();
    };
  }, [onProgress]);
}
