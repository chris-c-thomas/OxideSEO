/**
 * Dashboard view: recent crawls, summary stats, and quick-start new crawl.
 */

import { useEffect, useState } from "react";
import type { AppView } from "@/App";
import { getRecentCrawls, openCrawlFile, saveCrawlFile } from "@/lib/commands";
import { formatNumber, stateColor } from "@/lib/utils";
import type { CrawlSummary } from "@/types";

interface DashboardProps {
  onNavigate: (view: AppView, crawlId?: string, secondCrawlId?: string) => void;
}

export function Dashboard({ onNavigate }: DashboardProps) {
  const [recentCrawls, setRecentCrawls] = useState<CrawlSummary[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [compareMode, setCompareMode] = useState(false);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  const loadCrawls = () => {
    setIsLoading(true);
    setError(null);
    getRecentCrawls(20)
      .then(setRecentCrawls)
      .catch((err) => setError(String(err)))
      .finally(() => setIsLoading(false));
  };

  useEffect(() => {
    loadCrawls();
  }, []);

  const handleOpenFile = async () => {
    try {
      const crawlId = await openCrawlFile();
      if (crawlId) {
        loadCrawls();
        onNavigate("results", crawlId);
      }
    } catch (err) {
      setError(String(err));
    }
  };

  const handleSaveCrawl = async (crawlId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await saveCrawlFile(crawlId);
    } catch (err) {
      setError(String(err));
    }
  };

  const toggleCompareMode = () => {
    setCompareMode((prev) => !prev);
    setSelectedIds(new Set());
  };

  const toggleCrawlSelection = (crawlId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(crawlId)) {
        next.delete(crawlId);
      } else if (next.size < 2) {
        next.add(crawlId);
      }
      return next;
    });
  };

  const handleCompare = () => {
    const ids = Array.from(selectedIds);
    if (ids.length !== 2) return;

    // Sort so the older crawl is base, newer is compare.
    const crawl0 = recentCrawls.find((c) => c.crawlId === ids[0]);
    const crawl1 = recentCrawls.find((c) => c.crawlId === ids[1]);
    const [baseId, compareId] =
      (crawl0?.startedAt ?? "") <= (crawl1?.startedAt ?? "")
        ? [ids[0], ids[1]]
        : [ids[1], ids[0]];

    onNavigate("crawl-comparison", baseId, compareId);
  };

  const isSelectable = (crawl: CrawlSummary) =>
    crawl.status === "completed" || crawl.status === "stopped";

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
        <div className="flex gap-2">
          <button
            onClick={toggleCompareMode}
            className="rounded-md border px-4 py-2 text-sm font-medium transition-colors hover:bg-[var(--color-muted)]"
            style={{
              borderColor: compareMode ? "var(--color-primary)" : "var(--color-border)",
              backgroundColor: compareMode ? "var(--color-muted)" : "transparent",
            }}
          >
            {compareMode ? "Cancel Compare" : "Compare"}
          </button>
          <button
            onClick={handleOpenFile}
            className="rounded-md border px-4 py-2 text-sm font-medium transition-colors hover:bg-[var(--color-muted)]"
            style={{ borderColor: "var(--color-border)" }}
          >
            Open File
          </button>
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
                onClick={
                  compareMode
                    ? (e) =>
                        isSelectable(crawl)
                          ? toggleCrawlSelection(crawl.crawlId, e)
                          : undefined
                    : () => onNavigate("results", crawl.crawlId)
                }
                className="flex w-full items-center gap-4 rounded-lg border p-4 text-left transition-colors hover:bg-[var(--color-muted)]"
                style={{
                  borderColor: selectedIds.has(crawl.crawlId)
                    ? "var(--color-primary)"
                    : "var(--color-border)",
                  opacity: compareMode && !isSelectable(crawl) ? 0.5 : 1,
                }}
              >
                {/* Compare checkbox */}
                {compareMode && (
                  <div
                    className="flex h-5 w-5 shrink-0 items-center justify-center rounded border"
                    style={{
                      borderColor: selectedIds.has(crawl.crawlId)
                        ? "var(--color-primary)"
                        : "var(--color-border)",
                      backgroundColor: selectedIds.has(crawl.crawlId)
                        ? "var(--color-primary)"
                        : "transparent",
                    }}
                  >
                    {selectedIds.has(crawl.crawlId) && (
                      <span
                        className="text-xs"
                        style={{ color: "var(--color-primary-foreground)" }}
                      >
                        &#10003;
                      </span>
                    )}
                  </div>
                )}

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
                {!compareMode && (
                  <button
                    onClick={(e) => handleSaveCrawl(crawl.crawlId, e)}
                    className="rounded-md border px-2 py-1 text-xs transition-colors hover:bg-[var(--color-muted)]"
                    style={{ borderColor: "var(--color-border)" }}
                    title="Save as .seocrawl file"
                  >
                    Save
                  </button>
                )}
              </button>
            ))}
          </div>
        )}

        {/* Compare action bar */}
        {compareMode && selectedIds.size === 2 && (
          <div
            className="mt-4 flex items-center justify-between rounded-lg border p-4"
            style={{
              borderColor: "var(--color-primary)",
              backgroundColor: "var(--color-muted)",
            }}
          >
            <p className="text-sm font-medium">2 crawls selected for comparison</p>
            <button
              onClick={handleCompare}
              className="rounded-md px-4 py-2 text-sm font-medium"
              style={{
                backgroundColor: "var(--color-primary)",
                color: "var(--color-primary-foreground)",
              }}
            >
              Compare Crawls
            </button>
          </div>
        )}

        {compareMode && selectedIds.size < 2 && recentCrawls.length > 0 && (
          <p
            className="mt-2 text-center text-xs"
            style={{ color: "var(--color-muted-foreground)" }}
          >
            Select 2 completed crawls to compare
          </p>
        )}
      </section>
    </div>
  );
}
