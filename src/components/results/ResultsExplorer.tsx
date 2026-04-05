import { useEffect, useState } from "react";
import { cn } from "@/lib/utils";
import { getCrawlSummary } from "@/lib/commands";
import type { CrawlSummary } from "@/types";
import { PagesTab } from "./PagesTab";
import { IssuesTab } from "./IssuesTab";
import { LinksTab } from "./LinksTab";
import { ImagesTab } from "./ImagesTab";
import { PageDetail } from "./PageDetail";
import { Badge } from "@/components/ui/badge";

interface ResultsExplorerProps {
  crawlId: string | null;
}

type ResultsTab = "pages" | "issues" | "links" | "images";

const TABS: { id: ResultsTab; label: string }[] = [
  { id: "pages", label: "All Pages" },
  { id: "issues", label: "Issues" },
  { id: "links", label: "Links" },
  { id: "images", label: "Images" },
];

export function ResultsExplorer({ crawlId }: ResultsExplorerProps) {
  const [activeTab, setActiveTab] = useState<ResultsTab>("pages");
  const [selectedPageId, setSelectedPageId] = useState<number | null>(null);
  const [summary, setSummary] = useState<CrawlSummary | null>(null);
  const [summaryError, setSummaryError] = useState<string | null>(null);

  useEffect(() => {
    if (!crawlId) return;
    let stale = false;
    setSummaryError(null);
    getCrawlSummary(crawlId)
      .then((result) => {
        if (!stale) setSummary(result);
      })
      .catch((err) => {
        if (!stale) setSummaryError(String(err));
      });
    return () => {
      stale = true;
    };
  }, [crawlId]);

  if (!crawlId) {
    return (
      <div className="flex h-full items-center justify-center">
        <p style={{ color: "var(--color-muted-foreground)" }}>
          Select a crawl from the Dashboard to view results.
        </p>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      {/* Header with tabs */}
      <div
        className="flex items-center gap-4 border-b px-6 pt-6"
        style={{ borderColor: "var(--color-border)" }}
      >
        <h1 className="text-lg font-bold tracking-tight">Results</h1>
        <div className="flex gap-1">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={cn(
                "rounded-t-md px-3 py-2 text-sm transition-colors",
                activeTab === tab.id && "font-medium",
              )}
              style={{
                backgroundColor:
                  activeTab === tab.id ? "var(--color-background)" : "transparent",
                borderBottom:
                  activeTab === tab.id
                    ? "2px solid var(--color-primary)"
                    : "2px solid transparent",
              }}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </div>

      {/* Summary bar */}
      {summaryError && (
        <div
          className="border-b px-6 py-2"
          style={{ borderColor: "var(--color-border)" }}
        >
          <span className="text-xs" style={{ color: "var(--color-severity-error)" }}>
            Failed to load summary: {summaryError}
          </span>
        </div>
      )}
      {summary && !summaryError && (
        <div
          className="flex items-center gap-4 border-b px-6 py-2"
          style={{ borderColor: "var(--color-border)" }}
        >
          <span
            className="text-xs tabular-nums"
            style={{ color: "var(--color-muted-foreground)" }}
          >
            {summary.urlsCrawled.toLocaleString()} pages
          </span>
          {summary.issueCounts.errors > 0 && (
            <Badge variant="destructive" className="text-xs">
              {summary.issueCounts.errors} errors
            </Badge>
          )}
          {summary.issueCounts.warnings > 0 && (
            <Badge variant="default" className="text-xs">
              {summary.issueCounts.warnings} warnings
            </Badge>
          )}
          {summary.issueCounts.info > 0 && (
            <Badge variant="secondary" className="text-xs">
              {summary.issueCounts.info} info
            </Badge>
          )}
        </div>
      )}

      {/* Tab content area */}
      <div className="flex-1 overflow-hidden p-6">
        {activeTab === "pages" && (
          <PagesTab crawlId={crawlId} onRowClick={(page) => setSelectedPageId(page.id)} />
        )}
        {activeTab === "issues" && (
          <IssuesTab
            crawlId={crawlId}
            onPageClick={(pageId) => setSelectedPageId(pageId)}
          />
        )}
        {activeTab === "links" && <LinksTab crawlId={crawlId} />}
        {activeTab === "images" && <ImagesTab crawlId={crawlId} />}
      </div>

      {/* Page detail sheet */}
      {selectedPageId !== null && (
        <PageDetail
          crawlId={crawlId}
          pageId={selectedPageId}
          open={selectedPageId !== null}
          onClose={() => setSelectedPageId(null)}
        />
      )}
    </div>
  );
}
