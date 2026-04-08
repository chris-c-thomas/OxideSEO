/**
 * Crawl comparison view: tabbed interface showing differences between two crawls.
 */

import { useEffect, useState } from "react";
import { getComparisonSummary } from "@/lib/commands";
import { cn } from "@/lib/utils";
import type { CrawlComparisonSummary } from "@/types";
import { ComparisonOverview } from "./ComparisonOverview";
import { PageDiffTab } from "./PageDiffTab";
import { IssueDiffTab } from "./IssueDiffTab";
import { MetadataDiffTab } from "./MetadataDiffTab";

interface CrawlComparisonProps {
  baseCrawlId: string | null;
  compareCrawlId: string | null;
}

type ComparisonTab = "overview" | "pages" | "issues" | "metadata";

const TABS: { id: ComparisonTab; label: string }[] = [
  { id: "overview", label: "Overview" },
  { id: "pages", label: "Pages Diff" },
  { id: "issues", label: "Issues Diff" },
  { id: "metadata", label: "Metadata Diff" },
];

export function CrawlComparison({ baseCrawlId, compareCrawlId }: CrawlComparisonProps) {
  const [activeTab, setActiveTab] = useState<ComparisonTab>("overview");
  const [summary, setSummary] = useState<CrawlComparisonSummary | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!baseCrawlId || !compareCrawlId) return;
    setIsLoading(true);
    setError(null);
    getComparisonSummary(baseCrawlId, compareCrawlId)
      .then(setSummary)
      .catch((err) => setError(String(err)))
      .finally(() => setIsLoading(false));
  }, [baseCrawlId, compareCrawlId]);

  if (!baseCrawlId || !compareCrawlId) {
    return (
      <div className="flex h-full items-center justify-center">
        <p style={{ color: "var(--color-muted-foreground)" }}>
          Select two crawls from the Dashboard to compare.
        </p>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <p style={{ color: "var(--color-muted-foreground)" }}>Loading comparison...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full items-center justify-center">
        <p style={{ color: "var(--color-severity-error)" }}>
          Failed to load comparison: {error}
        </p>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="border-b px-6 py-4" style={{ borderColor: "var(--color-border)" }}>
        <h1 className="text-xl font-bold tracking-tight">Crawl Comparison</h1>
        {summary && (
          <p className="mt-1 text-xs" style={{ color: "var(--color-muted-foreground)" }}>
            {summary.baseCrawl.startedAt ?? "N/A"} vs{" "}
            {summary.compareCrawl.startedAt ?? "N/A"}
          </p>
        )}
      </div>

      {/* Tab bar */}
      <div
        className="flex gap-1 border-b px-6"
        style={{ borderColor: "var(--color-border)" }}
      >
        {TABS.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={cn(
              "border-b-2 px-3 py-2 text-sm font-medium transition-colors",
              activeTab === tab.id
                ? "border-[var(--color-primary)]"
                : "border-transparent",
            )}
            style={{
              color:
                activeTab === tab.id
                  ? "var(--color-foreground)"
                  : "var(--color-muted-foreground)",
            }}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      <div className="min-h-0 flex-1">
        {activeTab === "overview" && summary && (
          <ComparisonOverview summary={summary} onTabChange={setActiveTab} />
        )}
        {activeTab === "pages" && (
          <PageDiffTab baseCrawlId={baseCrawlId} compareCrawlId={compareCrawlId} />
        )}
        {activeTab === "issues" && (
          <IssueDiffTab baseCrawlId={baseCrawlId} compareCrawlId={compareCrawlId} />
        )}
        {activeTab === "metadata" && (
          <MetadataDiffTab baseCrawlId={baseCrawlId} compareCrawlId={compareCrawlId} />
        )}
      </div>
    </div>
  );
}
