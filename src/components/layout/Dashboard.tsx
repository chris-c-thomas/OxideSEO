/**
 * Dashboard view: recent crawls, summary stats, and quick-start new crawl.
 */

import { useEffect, useState } from "react";
import type { AppView } from "@/App";
import { getRecentCrawls } from "@/lib/commands";
import { formatNumber, stateColor } from "@/lib/utils";
import type { CrawlSummary } from "@/types";

interface DashboardProps {
  onNavigate: (view: AppView, crawlId?: string) => void;
}

export function Dashboard({ onNavigate }: DashboardProps) {
  const [recentCrawls, setRecentCrawls] = useState<CrawlSummary[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getRecentCrawls(20)
      .then(setRecentCrawls)
      .catch((err) => setError(String(err)))
      .finally(() => setIsLoading(false));
  }, []);

  return (
    <div className="mx-auto max-w-5xl space-y-8 p-8">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Dashboard</h1>
          <p className="mt-1 text-sm" style={{ color: "var(--color-muted-foreground)" }}>
            Overview of your SEO crawls and audits.
          </p>
        </div>
        <button
          onClick={() => onNavigate("crawl-config")}
          className="rounded-md px-4 py-2 text-sm font-medium"
          style={{
            backgroundColor: "var(--color-primary)",
            color: "var(--color-primary-foreground)",
          }}
        >
          New Crawl
        </button>
      </div>

      {/* Recent crawls */}
      <section>
        <h2 className="mb-4 text-lg font-semibold">Recent Crawls</h2>
        {error && !isLoading ? (
          <div
            className="rounded-lg border p-4"
            style={{ borderColor: "var(--color-severity-error)" }}
          >
            <p className="text-sm" style={{ color: "var(--color-severity-error)" }}>
              Failed to load recent crawls: {error}
            </p>
          </div>
        ) : isLoading ? (
          <p style={{ color: "var(--color-muted-foreground)" }}>Loading...</p>
        ) : recentCrawls.length === 0 ? (
          <div
            className="rounded-lg border p-8 text-center"
            style={{ borderColor: "var(--color-border)" }}
          >
            <p className="text-sm" style={{ color: "var(--color-muted-foreground)" }}>
              No crawls yet. Start your first crawl to see results here.
            </p>
            <button
              onClick={() => onNavigate("crawl-config")}
              className="mt-4 rounded-md px-4 py-2 text-sm font-medium"
              style={{
                backgroundColor: "var(--color-primary)",
                color: "var(--color-primary-foreground)",
              }}
            >
              Configure Crawl
            </button>
          </div>
        ) : (
          <div className="space-y-2">
            {recentCrawls.map((crawl) => (
              <button
                key={crawl.crawlId}
                onClick={() => onNavigate("results", crawl.crawlId)}
                className="flex w-full items-center gap-4 rounded-lg border p-4 text-left transition-colors hover:bg-[var(--color-muted)]"
                style={{ borderColor: "var(--color-border)" }}
              >
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium">{crawl.startUrl}</p>
                  <p
                    className="text-xs"
                    style={{ color: "var(--color-muted-foreground)" }}
                  >
                    {crawl.startedAt ?? "Not started"}
                  </p>
                </div>
                <div
                  className="text-right text-xs tabular-nums"
                  style={{ color: "var(--color-muted-foreground)" }}
                >
                  <p>{formatNumber(crawl.urlsCrawled)} pages</p>
                  <div className="flex items-center justify-end gap-2">
                    {crawl.issueCounts.errors > 0 && (
                      <span className="flex items-center gap-1">
                        <span
                          className="inline-block h-2 w-2 rounded-full"
                          style={{ backgroundColor: "var(--color-severity-error)" }}
                        />
                        {crawl.issueCounts.errors}
                      </span>
                    )}
                    {crawl.issueCounts.warnings > 0 && (
                      <span className="flex items-center gap-1">
                        <span
                          className="inline-block h-2 w-2 rounded-full"
                          style={{ backgroundColor: "var(--color-severity-warning)" }}
                        />
                        {crawl.issueCounts.warnings}
                      </span>
                    )}
                    {crawl.issueCounts.errors === 0 &&
                      crawl.issueCounts.warnings === 0 && <span>No issues</span>}
                  </div>
                </div>
                <span
                  className="rounded-full px-2 py-0.5 text-xs font-medium capitalize"
                  style={{
                    color: stateColor(crawl.status),
                    backgroundColor: "var(--color-muted)",
                  }}
                >
                  {crawl.status}
                </span>
              </button>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
