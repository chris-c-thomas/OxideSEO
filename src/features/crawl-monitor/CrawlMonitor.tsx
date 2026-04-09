/**
 * Live crawl monitor: ProgressRing, live counters, URL stream, controls.
 */

import { useState } from "react";
import { pauseCrawl, resumeCrawl, stopCrawl } from "@/lib/commands";
import { formatDuration, formatNumber, formatRps, formatBytes } from "@/lib/utils";
import { useCrawlStore } from "@/stores/crawlStore";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { ProgressRing } from "@/components/ProgressRing";
import { MetricCard } from "@/components/MetricCard";
import { EmptyState } from "@/components/EmptyState";
import { Panel } from "@/components/Panel";
import { StatusCodeBadge } from "@/components/badges/StatusCodeBadge";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { Pause, Play, Square, ArrowRight, Activity } from "lucide-react";
import { toast } from "sonner";

interface CrawlMonitorProps {
  crawlId: string | null;
  onCompleted: () => void;
}

export function CrawlMonitor({ crawlId, onCompleted }: CrawlMonitorProps) {
  const [showStopConfirm, setShowStopConfirm] = useState(false);

  const progress = useCrawlStore((s) => s.progress);
  const state = useCrawlStore((s) => s.state);
  const maxMemoryMb = useCrawlStore((s) => s.config?.maxMemoryMb ?? 512);

  if (!crawlId) {
    return (
      <div className="flex h-full items-center justify-center">
        <EmptyState
          icon={Activity}
          title="No active crawl"
          description="Start a new crawl from the configuration page."
          /* No action -- navigation is handled by the sidebar */
        />
      </div>
    );
  }

  const handlePause = async () => {
    try {
      await pauseCrawl(crawlId);
    } catch (err) {
      toast.error(`Failed to pause crawl: ${String(err)}`);
    }
  };

  const handleResume = async () => {
    try {
      await resumeCrawl(crawlId);
    } catch (err) {
      toast.error(`Failed to resume crawl: ${String(err)}`);
    }
  };

  const handleStop = async () => {
    try {
      await stopCrawl(crawlId);
    } catch (err) {
      toast.error(`Failed to stop crawl: ${String(err)}`);
    }
  };

  const urlsCrawled = progress?.urlsCrawled ?? 0;
  const urlsQueued = progress?.urlsQueued ?? 0;
  const urlsErrored = progress?.urlsErrored ?? 0;
  const total = urlsCrawled + urlsQueued;
  const pct = total > 0 ? Math.round((urlsCrawled / total) * 100) : 0;
  const memoryPct =
    progress?.memoryRssBytes != null
      ? Math.round((progress.memoryRssBytes / (maxMemoryMb * 1024 * 1024)) * 100)
      : 0;

  return (
    <div className="mx-auto max-w-5xl p-6">
      {/* Header with controls */}
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-fg-default text-xl font-semibold tracking-tight">
          Crawl Monitor
        </h1>
        <div className="flex items-center gap-2">
          {state === "running" && (
            <Button variant="outline" size="sm" onClick={handlePause}>
              <Pause className="size-3.5" strokeWidth={1.75} />
              Pause
            </Button>
          )}
          {state === "paused" && (
            <Button variant="outline" size="sm" onClick={handleResume}>
              <Play className="size-3.5" strokeWidth={1.75} />
              Resume
            </Button>
          )}
          {(state === "running" || state === "paused") && (
            <Button
              variant="destructive"
              size="sm"
              onClick={() => setShowStopConfirm(true)}
            >
              <Square className="size-3.5" strokeWidth={1.75} />
              Stop
            </Button>
          )}
          {(state === "completed" || state === "stopped") && (
            <Button size="sm" onClick={onCompleted}>
              <ArrowRight className="size-3.5" strokeWidth={1.75} />
              View Results
            </Button>
          )}
        </div>
      </div>

      {/* Progress + Stats */}
      <div className="mb-6 flex items-start gap-6">
        {/* Progress Ring */}
        <ProgressRing
          value={pct}
          size={120}
          strokeWidth={8}
          label={formatDuration(progress?.elapsedMs ?? 0)}
        />

        {/* Metric cards */}
        <div className="grid flex-1 grid-cols-4 gap-3">
          <MetricCard label="Crawled" value={formatNumber(urlsCrawled)} />
          <MetricCard label="Queued" value={formatNumber(urlsQueued)} />
          <MetricCard label="Errors" value={formatNumber(urlsErrored)} />
          <MetricCard label="Speed" value={formatRps(progress?.currentRps ?? 0)} />
        </div>
      </div>

      {/* Memory usage */}
      {progress?.memoryRssBytes != null && (
        <div className="mb-6">
          <div className="text-fg-muted mb-1 flex items-center justify-between text-xs">
            <span>Memory Usage</span>
            <span className="tabular-nums">
              {formatBytes(progress.memoryRssBytes)} / {maxMemoryMb} MB
            </span>
          </div>
          <Progress value={memoryPct} className="h-1.5" />
        </div>
      )}

      {/* Recent URLs */}
      <Panel title="Recent URLs">
        <div className="custom-scrollbar max-h-80 overflow-auto">
          <table className="w-full">
            <thead>
              <tr className="border-border-subtle border-b">
                <th className="text-fg-muted px-3 py-1.5 text-left text-xs font-medium">
                  URL
                </th>
                <th className="text-fg-muted w-20 px-3 py-1.5 text-right text-xs font-medium">
                  Status
                </th>
                <th className="text-fg-muted w-20 px-3 py-1.5 text-right text-xs font-medium">
                  Time
                </th>
              </tr>
            </thead>
            <tbody>
              {(progress?.recentUrls ?? []).map((url, i) => (
                <tr
                  key={`${url.url}-${i}`}
                  className="border-border-subtle border-b last:border-b-0"
                >
                  <td className="text-fg-default max-w-0 truncate px-3 py-1.5 font-mono text-xs">
                    {url.url}
                  </td>
                  <td className="px-3 py-1.5 text-right">
                    {url.statusCode != null ? (
                      <StatusCodeBadge code={url.statusCode} />
                    ) : (
                      <span className="text-fg-subtle text-xs">...</span>
                    )}
                  </td>
                  <td className="text-fg-muted px-3 py-1.5 text-right text-xs tabular-nums">
                    {url.responseTimeMs != null ? `${url.responseTimeMs}ms` : "..."}
                  </td>
                </tr>
              ))}
              {(!progress || progress.recentUrls.length === 0) && (
                <tr>
                  <td colSpan={3} className="text-fg-muted px-3 py-8 text-center text-xs">
                    Waiting for crawl data...
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </Panel>

      <ConfirmDialog
        open={showStopConfirm}
        title="Stop Crawl"
        description="This will stop the crawl. In-flight requests will complete but no new URLs will be fetched."
        confirmLabel="Stop"
        variant="destructive"
        onConfirm={() => {
          setShowStopConfirm(false);
          handleStop();
        }}
        onCancel={() => setShowStopConfirm(false)}
      />
    </div>
  );
}
