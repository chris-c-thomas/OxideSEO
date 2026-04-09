/**
 * Application status bar.
 *
 * Fixed 24px strip at the bottom showing crawl state, speed,
 * progress counters, and memory usage. Wired to the crawl store.
 */

import { useCrawlStore } from "@/stores/crawlStore";
import { cn, formatNumber, formatRps, formatBytes } from "@/lib/utils";

const STATE_DOT_COLOR: Record<string, string> = {
  running: "bg-status-running",
  paused: "bg-status-paused",
  error: "bg-status-error",
  completed: "bg-status-completed",
};

export function StatusBar() {
  const state = useCrawlStore((s) => s.state);
  const progress = useCrawlStore((s) => s.progress);

  const isActive = state === "running" || state === "paused";

  return (
    <div className="border-border-subtle bg-bg-subtle text-fg-muted flex h-6 shrink-0 items-center border-t px-3 font-mono text-[0.6875rem]">
      {/* Left: crawl state */}
      <div className="flex items-center gap-2">
        <span
          className={cn(
            "size-1.5 rounded-full",
            state ? (STATE_DOT_COLOR[state] ?? "bg-fg-subtle") : "bg-fg-subtle",
          )}
        />
        <span className="capitalize">{state ?? "Ready"}</span>
      </div>

      {/* Spacer */}
      <div className="flex-1" />

      {/* Right: stats when crawl is active */}
      {isActive && progress && (
        <div className="flex items-center gap-4 tabular-nums">
          <span>{formatRps(progress.currentRps)}</span>
          <span>{formatNumber(progress.urlsCrawled)} crawled</span>
          <span>{formatNumber(progress.urlsQueued)} queued</span>
          {progress.urlsErrored > 0 && (
            <span className="text-danger">
              {formatNumber(progress.urlsErrored)} errors
            </span>
          )}
          {progress.memoryRssBytes != null && (
            <span>{formatBytes(progress.memoryRssBytes)} RSS</span>
          )}
        </div>
      )}
    </div>
  );
}
