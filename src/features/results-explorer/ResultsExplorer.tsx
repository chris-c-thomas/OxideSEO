/**
 * Results explorer with tabbed data views and detail drawer.
 *
 * Replaces src/components/results/ResultsExplorer.tsx with shadcn Tabs
 * and token-based styling. Keeps existing tab sub-components for now
 * (PagesTab, IssuesTab, etc.) which still use the old DataTable.
 */

import { useEffect, useState } from "react";
import { getCrawlSummary } from "@/lib/commands";
import type { CrawlSummary } from "@/types";
import { PagesTab } from "@/components/results/PagesTab";
import { IssuesTab } from "@/components/results/IssuesTab";
import { LinksTab } from "@/components/results/LinksTab";
import { ImagesTab } from "@/components/results/ImagesTab";
import { SitemapTab } from "@/components/results/SitemapTab";
import { ExternalLinksTab } from "@/components/results/ExternalLinksTab";
import { AiInsightsTab } from "@/components/results/AiInsightsTab";
import { SiteTreeTab } from "@/components/results/SiteTreeTab";
import { PageDetail } from "@/components/results/PageDetail";
import { ExportDialog } from "@/components/export/ExportDialog";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { EmptyState } from "@/components/EmptyState";
import { SeverityBadge } from "@/components/badges/SeverityBadge";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Download, FileSearch, AlertCircle } from "lucide-react";

interface ResultsExplorerProps {
  crawlId: string | null;
}

type ResultsTab =
  | "pages"
  | "issues"
  | "links"
  | "images"
  | "sitemap"
  | "external"
  | "ai"
  | "tree";

const TABS: { id: ResultsTab; label: string }[] = [
  { id: "pages", label: "All Pages" },
  { id: "issues", label: "Issues" },
  { id: "links", label: "Links" },
  { id: "images", label: "Images" },
  { id: "sitemap", label: "Sitemap" },
  { id: "external", label: "External" },
  { id: "ai", label: "AI Insights" },
  { id: "tree", label: "Site Tree" },
];

export function ResultsExplorer({ crawlId }: ResultsExplorerProps) {
  const [activeTab, setActiveTab] = useState<ResultsTab>("pages");
  const [selectedPageId, setSelectedPageId] = useState<number | null>(null);
  const [summary, setSummary] = useState<CrawlSummary | null>(null);
  const [summaryError, setSummaryError] = useState<string | null>(null);
  const [showExportDialog, setShowExportDialog] = useState(false);

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
        <EmptyState
          icon={FileSearch}
          title="No crawl selected"
          description="Select a crawl from the Dashboard to view results."
        />
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="border-border-subtle flex items-center gap-4 border-b px-4 pt-4">
        <h1 className="text-fg-default text-base font-semibold tracking-tight">
          Results
        </h1>

        <Tabs
          value={activeTab}
          onValueChange={(v) => setActiveTab(v as ResultsTab)}
          className="flex-1"
        >
          <TabsList className="h-8">
            {TABS.map((tab) => (
              <TabsTrigger key={tab.id} value={tab.id} className="text-xs">
                {tab.label}
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>

        <Button variant="outline" size="sm" onClick={() => setShowExportDialog(true)}>
          <Download className="size-3.5" strokeWidth={1.75} />
          Export
        </Button>
      </div>

      {/* Summary bar */}
      {summaryError && (
        <Alert variant="destructive" className="mx-4 mt-2">
          <AlertCircle className="size-4" />
          <AlertDescription>Failed to load summary: {summaryError}</AlertDescription>
        </Alert>
      )}
      {summary && !summaryError && (
        <div className="border-border-subtle flex items-center gap-3 border-b px-4 py-2">
          <span className="text-fg-muted text-xs tabular-nums">
            {summary.urlsCrawled.toLocaleString()} pages
          </span>
          {summary.issueCounts.errors > 0 && <SeverityBadge severity="critical" />}
          {summary.issueCounts.warnings > 0 && <SeverityBadge severity="medium" />}
          {summary.issueCounts.info > 0 && (
            <Badge variant="secondary" className="text-[0.625rem]">
              {summary.issueCounts.info} info
            </Badge>
          )}
        </div>
      )}

      {/* Tab content */}
      <div className="flex-1 overflow-hidden p-4">
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
        {activeTab === "sitemap" && <SitemapTab crawlId={crawlId} />}
        {activeTab === "external" && <ExternalLinksTab crawlId={crawlId} />}
        {activeTab === "ai" && <AiInsightsTab crawlId={crawlId} />}
        {activeTab === "tree" && (
          <SiteTreeTab
            crawlId={crawlId}
            onPageClick={(pageId) => setSelectedPageId(pageId)}
          />
        )}
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

      {/* Export dialog */}
      <ExportDialog
        crawlId={crawlId}
        open={showExportDialog}
        onClose={() => setShowExportDialog(false)}
        activeTab={activeTab}
      />
    </div>
  );
}
