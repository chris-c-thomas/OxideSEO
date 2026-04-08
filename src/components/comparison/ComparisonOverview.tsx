/**
 * Comparison overview: side-by-side crawl summaries and delta stats.
 */

import { formatNumber } from "@/lib/utils";
import type { CrawlComparisonSummary } from "@/types";

type ComparisonTab = "overview" | "pages" | "issues" | "metadata";

interface ComparisonOverviewProps {
  summary: CrawlComparisonSummary;
  onTabChange: (tab: ComparisonTab) => void;
}

export function ComparisonOverview({ summary, onTabChange }: ComparisonOverviewProps) {
  const { baseCrawl, compareCrawl } = summary;

  return (
    <div className="space-y-6 p-4">
      {/* Side-by-side crawl summaries */}
      <div className="grid grid-cols-2 gap-4">
        <CrawlCard label="Base Crawl (older)" crawl={baseCrawl} />
        <CrawlCard label="Compare Crawl (newer)" crawl={compareCrawl} />
      </div>

      {/* Delta stats */}
      <h3 className="text-sm font-semibold">Changes</h3>
      <div className="grid grid-cols-4 gap-3">
        <DeltaCard
          label="New Pages"
          value={summary.newPages}
          color="var(--color-status-completed)"
          onClick={() => onTabChange("pages")}
        />
        <DeltaCard
          label="Removed Pages"
          value={summary.removedPages}
          color="var(--color-severity-error)"
          onClick={() => onTabChange("pages")}
        />
        <DeltaCard
          label="Status Changes"
          value={summary.changedStatusCode}
          color="var(--color-severity-warning)"
          onClick={() => onTabChange("pages")}
        />
        <DeltaCard
          label="Title Changes"
          value={summary.changedTitle}
          color="var(--color-primary)"
          onClick={() => onTabChange("metadata")}
        />
        <DeltaCard
          label="Meta Desc Changes"
          value={summary.changedMetaDesc}
          color="var(--color-primary)"
          onClick={() => onTabChange("metadata")}
        />
        <DeltaCard
          label="New Issues"
          value={summary.newIssues}
          color="var(--color-severity-error)"
          onClick={() => onTabChange("issues")}
        />
        <DeltaCard
          label="Resolved Issues"
          value={summary.resolvedIssues}
          color="var(--color-status-completed)"
          onClick={() => onTabChange("issues")}
        />
      </div>
    </div>
  );
}

function CrawlCard({
  label,
  crawl,
}: {
  label: string;
  crawl: CrawlComparisonSummary["baseCrawl"];
}) {
  return (
    <div className="rounded-lg border p-4" style={{ borderColor: "var(--color-border)" }}>
      <p
        className="mb-2 text-xs font-medium"
        style={{ color: "var(--color-muted-foreground)" }}
      >
        {label}
      </p>
      <p className="truncate text-sm font-medium">{crawl.startUrl}</p>
      <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
        {crawl.startedAt ?? "N/A"}
      </p>
      <div className="mt-2 flex gap-4 text-xs tabular-nums">
        <span>{formatNumber(crawl.urlsCrawled)} pages</span>
        <span style={{ color: "var(--color-severity-error)" }}>
          {crawl.issueCounts.errors} errors
        </span>
        <span style={{ color: "var(--color-severity-warning)" }}>
          {crawl.issueCounts.warnings} warnings
        </span>
      </div>
    </div>
  );
}

function DeltaCard({
  label,
  value,
  color,
  onClick,
}: {
  label: string;
  value: number;
  color: string;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className="rounded-lg border p-3 text-left transition-colors hover:bg-[var(--color-muted)]"
      style={{ borderColor: "var(--color-border)" }}
    >
      <p
        className="text-xl font-bold tabular-nums"
        style={{ color: value > 0 ? color : undefined }}
      >
        {formatNumber(value)}
      </p>
      <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
        {label}
      </p>
    </button>
  );
}
