/**
 * Live crawl monitor: real-time stats, progress bar, URL stream, controls.
 */

import { useCrawlProgress } from "@/hooks/useCrawlProgress";
import { pauseCrawl, resumeCrawl, stopCrawl } from "@/lib/commands";
import { formatDuration, formatNumber, formatRps, truncate } from "@/lib/utils";
import { useCrawlStore } from "@/stores/crawlStore";

interface CrawlMonitorProps {
  crawlId: string | null;
  onCompleted: () => void;
}

export function CrawlMonitor({ crawlId, onCompleted }: CrawlMonitorProps) {
  useCrawlProgress(crawlId);

  const progress = useCrawlStore((s) => s.progress);
  const state = useCrawlStore((s) => s.state);
  const setCrawlState = useCrawlStore((s) => s.setCrawlState);

  if (!crawlId) {
    return (
      <div className="flex h-full items-center justify-center">
        <p style={{ color: "var(--color-muted-foreground)" }}>
          No active crawl. Start a new crawl from the configuration page.
        </p>
      </div>
    );
  }

  const handlePause = async () => {
    try {
      await pauseCrawl(crawlId);
      setCrawlState("paused");
    } catch (err) {
      console.error("Failed to pause:", err);
    }
  };

  const handleResume = async () => {
    try {
      await resumeCrawl(crawlId);
      setCrawlState("running");
    } catch (err) {
      console.error("Failed to resume:", err);
    }
  };

  const handleStop = async () => {
    try {
      await stopCrawl(crawlId);
      setCrawlState("stopped");
    } catch (err) {
      console.error("Failed to stop:", err);
    }
  };

  const urlsCrawled = progress?.urlsCrawled ?? 0;
  const urlsQueued = progress?.urlsQueued ?? 0;
  const urlsErrored = progress?.urlsErrored ?? 0;
  const total = urlsCrawled + urlsQueued;
  const pct = total > 0 ? Math.round((urlsCrawled / total) * 100) : 0;

  return (
    <div className="mx-auto max-w-5xl space-y-6 p-8">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold tracking-tight">Crawl Monitor</h1>
        <div className="flex items-center gap-2">
          {state === "running" && (
            <button
              onClick={handlePause}
              className="rounded-md border px-3 py-1.5 text-sm"
              style={{ borderColor: "var(--color-border)" }}
            >
              Pause
            </button>
          )}
          {state === "paused" && (
            <button
              onClick={handleResume}
              className="rounded-md border px-3 py-1.5 text-sm"
              style={{ borderColor: "var(--color-border)" }}
            >
              Resume
            </button>
          )}
          {(state === "running" || state === "paused") && (
            <button
              onClick={handleStop}
              className="rounded-md px-3 py-1.5 text-sm"
              style={{
                backgroundColor: "var(--color-destructive)",
                color: "var(--color-destructive-foreground)",
              }}
            >
              Stop
            </button>
          )}
          {(state === "completed" || state === "stopped") && (
            <button
              onClick={onCompleted}
              className="rounded-md px-3 py-1.5 text-sm font-medium"
              style={{
                backgroundColor: "var(--color-primary)",
                color: "var(--color-primary-foreground)",
              }}
            >
              View Results
            </button>
          )}
        </div>
      </div>

      {/* Progress bar */}
      <div>
        <div
          className="mb-1 flex items-center justify-between text-xs tabular-nums"
          style={{ color: "var(--color-muted-foreground)" }}
        >
          <span>{pct}% complete</span>
          <span>{formatDuration(progress?.elapsedMs ?? 0)}</span>
        </div>
        <div
          className="h-2 overflow-hidden rounded-full"
          style={{ backgroundColor: "var(--color-muted)" }}
        >
          <div
            className="h-full rounded-full transition-all duration-300"
            style={{
              width: `${pct}%`,
              backgroundColor: "var(--color-status-running)",
            }}
          />
        </div>
      </div>

      {/* Stats row */}
      <div className="grid grid-cols-4 gap-4">
        <StatCard label="Crawled" value={formatNumber(urlsCrawled)} />
        <StatCard label="Queued" value={formatNumber(urlsQueued)} />
        <StatCard label="Errors" value={formatNumber(urlsErrored)} />
        <StatCard label="Speed" value={formatRps(progress?.currentRps ?? 0)} />
      </div>

      {/* Recent URLs table */}
      <section>
        <h2 className="mb-2 text-sm font-semibold">Recent URLs</h2>
        <div
          className="custom-scrollbar max-h-80 overflow-auto rounded-lg border"
          style={{ borderColor: "var(--color-border)" }}
        >
          <table className="w-full text-sm">
            <thead>
              <tr style={{ backgroundColor: "var(--color-muted)" }}>
                <th className="px-3 py-2 text-left font-medium">URL</th>
                <th className="w-20 px-3 py-2 text-right font-medium">Status</th>
                <th className="w-20 px-3 py-2 text-right font-medium">Time</th>
              </tr>
            </thead>
            <tbody>
              {(progress?.recentUrls ?? []).map((url, i) => (
                <tr
                  key={`${url.url}-${i}`}
                  className="border-t"
                  style={{ borderColor: "var(--color-border)" }}
                >
                  <td className="px-3 py-1.5 font-mono text-xs">
                    {truncate(url.url, 80)}
                  </td>
                  <td className="px-3 py-1.5 text-right tabular-nums">
                    {url.statusCode ?? "..."}
                  </td>
                  <td
                    className="px-3 py-1.5 text-right tabular-nums"
                    style={{ color: "var(--color-muted-foreground)" }}
                  >
                    {url.responseTimeMs != null ? `${url.responseTimeMs}ms` : "..."}
                  </td>
                </tr>
              ))}
              {(!progress || progress.recentUrls.length === 0) && (
                <tr>
                  <td
                    colSpan={3}
                    className="px-3 py-4 text-center"
                    style={{ color: "var(--color-muted-foreground)" }}
                  >
                    Waiting for crawl data...
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border p-3" style={{ borderColor: "var(--color-border)" }}>
      <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
        {label}
      </p>
      <p className="text-xl font-bold tabular-nums">{value}</p>
    </div>
  );
}
