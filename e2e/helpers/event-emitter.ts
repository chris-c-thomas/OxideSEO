/**
 * Helpers for simulating Tauri backend events in Playwright E2E tests.
 *
 * These call the `window.__E2E_EMIT_EVENT__()` function injected by
 * the tauri-mock setup, triggering any registered event listeners
 * in the React app.
 */

import type { Page } from "@playwright/test";
import type { CrawlProgress, CrawlStateChange } from "@/types";

/** Emit a `crawl://progress` event with the given payload. */
export async function emitCrawlProgress(
  page: Page,
  progress: CrawlProgress,
): Promise<void> {
  await page.evaluate((payload) => {
    (
      window as unknown as {
        __E2E_EMIT_EVENT__: (event: string, payload: unknown) => void;
      }
    ).__E2E_EMIT_EVENT__("crawl://progress", payload);
  }, progress);
}

/** Emit a `crawl://state` event with the given payload. */
export async function emitCrawlState(
  page: Page,
  change: CrawlStateChange,
): Promise<void> {
  await page.evaluate((payload) => {
    (
      window as unknown as {
        __E2E_EMIT_EVENT__: (event: string, payload: unknown) => void;
      }
    ).__E2E_EMIT_EVENT__("crawl://state", payload);
  }, change);
}

/** Emit any arbitrary Tauri event. */
export async function emitTauriEvent(
  page: Page,
  event: string,
  payload: unknown,
): Promise<void> {
  await page.evaluate(
    ({ event: evt, payload: pl }) => {
      (
        window as unknown as {
          __E2E_EMIT_EVENT__: (event: string, payload: unknown) => void;
        }
      ).__E2E_EMIT_EVENT__(evt, pl);
    },
    { event, payload },
  );
}
