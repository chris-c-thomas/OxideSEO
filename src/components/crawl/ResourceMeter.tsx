/**
 * Resource meter: displays memory usage and throughput during an active crawl.
 */

import { formatBytes, formatRps } from "@/lib/utils";
import type { CrawlProgress } from "@/types";

interface ResourceMeterProps {
  progress: CrawlProgress | null;
  maxMemoryMb: number;
}

export function ResourceMeter({ progress, maxMemoryMb }: ResourceMeterProps) {
  const memoryBytes = progress?.memoryRssBytes ?? null;
  const budgetBytes = maxMemoryMb * 1024 * 1024;
  const memoryPct =
    memoryBytes != null ? Math.min((memoryBytes / budgetBytes) * 100, 100) : 0;
  const rps = progress?.currentRps ?? 0;
  const urlsCrawled = progress?.urlsCrawled ?? 0;
  const elapsedSecs = (progress?.elapsedMs ?? 0) / 1000;
  const avgRps = elapsedSecs > 0 ? urlsCrawled / elapsedSecs : 0;

  const memoryColor =
    memoryPct > 90
      ? "var(--color-severity-error)"
      : memoryPct > 70
        ? "var(--color-severity-warning)"
        : "var(--color-status-running)";

  return (
    <div className="rounded-lg border p-4" style={{ borderColor: "var(--color-border)" }}>
      <h3 className="mb-3 text-sm font-semibold">Resources</h3>
      <div className="space-y-3">
        {/* Memory gauge */}
        <div>
          <div
            className="mb-1 flex items-center justify-between text-xs"
            style={{ color: "var(--color-muted-foreground)" }}
          >
            <span>Memory (RSS)</span>
            <span className="tabular-nums">
              {memoryBytes != null
                ? `${formatBytes(memoryBytes)} / ${formatBytes(budgetBytes)}`
                : "N/A"}
            </span>
          </div>
          <div
            className="h-2 overflow-hidden rounded-full"
            style={{ backgroundColor: "var(--color-muted)" }}
          >
            <div
              className="h-full rounded-full transition-all duration-300"
              style={{
                width: `${memoryPct}%`,
                backgroundColor: memoryColor,
              }}
            />
          </div>
        </div>

        {/* Throughput stats */}
        <div className="grid grid-cols-2 gap-3 text-xs">
          <div>
            <span style={{ color: "var(--color-muted-foreground)" }}>Current</span>
            <p className="font-medium tabular-nums">{formatRps(rps)}</p>
          </div>
          <div>
            <span style={{ color: "var(--color-muted-foreground)" }}>Average</span>
            <p className="font-medium tabular-nums">{formatRps(avgRps)}</p>
          </div>
        </div>
      </div>
    </div>
  );
}
