/**
 * Dashboard view: recent crawls, summary metrics, and quick-start actions.
 *
 * Replaces the original src/components/layout/Dashboard.tsx with the new
 * design system (token classes, shadcn components, MetricCard, etc.).
 */

import { useEffect, useState } from "react";
import type { AppView } from "@/App";
import { getRecentCrawls, openCrawlFile, saveCrawlFile } from "@/lib/commands";
import { formatNumber } from "@/lib/utils";
import type { CrawlSummary } from "@/types";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { MetricCard } from "@/components/MetricCard";
import { Sparkline } from "@/components/Sparkline";
import { Panel } from "@/components/Panel";
import { EmptyState } from "@/components/EmptyState";
import { SeverityBadge } from "@/components/badges/SeverityBadge";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Globe,
  AlertTriangle,
  AlertCircle,
  Clock,
  FolderOpen,
  PlusCircle,
  GitCompare,
  Save,
  Search,
} from "lucide-react";

interface DashboardProps {
  onNavigate: (view: AppView, crawlId?: string, secondCrawlId?: string) => void;
}

function stateVariant(
  status: string,
): "default" | "secondary" | "destructive" | "outline" {
  if (status === "completed") return "default";
  if (status === "error") return "destructive";
  return "secondary";
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

  const toggleCrawlSelection = (crawlId: string) => {
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

  // Compute aggregate metrics from recent crawls.
  const totalUrls = recentCrawls.reduce((sum, c) => sum + c.urlsCrawled, 0);
  const totalErrors = recentCrawls.reduce((sum, c) => sum + c.issueCounts.errors, 0);
  const totalWarnings = recentCrawls.reduce((sum, c) => sum + c.issueCounts.warnings, 0);
  const totalInfo = recentCrawls.reduce((sum, c) => sum + c.issueCounts.info, 0);
  const sparklineData = recentCrawls
    .slice(0, 10)
    .reverse()
    .map((c) => c.urlsCrawled);

  return (
    <div className="mx-auto max-w-5xl p-6">
      {/* Header */}
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-fg-default text-xl font-semibold tracking-tight">
            Dashboard
          </h1>
          <p className="text-fg-muted mt-0.5 text-xs">
            Overview of your SEO crawls and audits.
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={toggleCompareMode}>
            <GitCompare className="size-3.5" strokeWidth={1.75} />
            {compareMode ? "Cancel" : "Compare"}
          </Button>
          <Button variant="outline" size="sm" onClick={handleOpenFile}>
            <FolderOpen className="size-3.5" strokeWidth={1.75} />
            Open File
          </Button>
          <Button size="sm" onClick={() => onNavigate("crawl-config")}>
            <PlusCircle className="size-3.5" strokeWidth={1.75} />
            New Crawl
          </Button>
        </div>
      </div>

      {/* Error */}
      {error && (
        <Alert variant="destructive" className="mb-4">
          <AlertCircle className="size-4" />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {/* Metrics grid */}
      {!isLoading && recentCrawls.length > 0 && (
        <div className="mb-6 grid grid-cols-4 gap-3">
          <MetricCard
            label="Total URLs"
            value={totalUrls}
            icon={Globe}
            sparkline={<Sparkline data={sparklineData} />}
          />
          <MetricCard label="Errors" value={totalErrors} icon={AlertCircle} />
          <MetricCard label="Warnings" value={totalWarnings} icon={AlertTriangle} />
          <MetricCard label="Info" value={totalInfo} icon={Clock} />
        </div>
      )}

      {/* Recent Crawls */}
      <Panel
        title="Recent Crawls"
        actions={
          compareMode && selectedIds.size === 2 ? (
            <Button size="sm" onClick={handleCompare}>
              Compare Selected
            </Button>
          ) : compareMode ? (
            <span className="text-fg-muted text-xs">Select 2 completed crawls</span>
          ) : undefined
        }
      >
        {isLoading ? (
          <div className="flex flex-col gap-2">
            {Array.from({ length: 4 }).map((_, i) => (
              <Skeleton key={i} className="h-14 w-full" />
            ))}
          </div>
        ) : recentCrawls.length === 0 ? (
          <EmptyState
            icon={Search}
            title="No crawls yet"
            description="Start your first crawl to see results here."
            action={{
              label: "Configure Crawl",
              onClick: () => onNavigate("crawl-config"),
            }}
          />
        ) : (
          <div className="flex flex-col gap-1">
            {recentCrawls.map((crawl) => {
              const selectable = isSelectable(crawl);
              return (
                <button
                  key={crawl.crawlId}
                  onClick={
                    compareMode
                      ? selectable
                        ? () => toggleCrawlSelection(crawl.crawlId)
                        : undefined
                      : () => onNavigate("results", crawl.crawlId)
                  }
                  className={`flex w-full items-center gap-3 rounded-[var(--radius-sm)] px-3 py-2.5 text-left transition-colors ${
                    selectedIds.has(crawl.crawlId)
                      ? "bg-accent-subtle"
                      : "hover:bg-bg-hover"
                  } ${compareMode && !selectable ? "opacity-40" : ""}`}
                >
                  {compareMode && (
                    <Checkbox
                      checked={selectedIds.has(crawl.crawlId)}
                      disabled={!selectable}
                      className="shrink-0"
                      onCheckedChange={() => toggleCrawlSelection(crawl.crawlId)}
                    />
                  )}
                  <div className="min-w-0 flex-1">
                    <p className="text-fg-default truncate text-sm font-medium">
                      {crawl.startUrl}
                    </p>
                    <p className="text-fg-muted text-[0.6875rem]">
                      {crawl.startedAt ?? "Not started"}
                    </p>
                  </div>
                  <div className="flex items-center gap-3 text-right">
                    <span className="text-fg-muted text-xs tabular-nums">
                      {formatNumber(crawl.urlsCrawled)} pages
                    </span>
                    <div className="flex items-center gap-2">
                      {crawl.issueCounts.errors > 0 && (
                        <SeverityBadge severity="critical" />
                      )}
                      {crawl.issueCounts.warnings > 0 && (
                        <SeverityBadge severity="medium" />
                      )}
                    </div>
                    <Badge variant={stateVariant(crawl.status)} className="capitalize">
                      {crawl.status}
                    </Badge>
                    {!compareMode && (
                      <Button
                        variant="ghost"
                        size="sm"
                        className="size-7 p-0"
                        onClick={(e) => handleSaveCrawl(crawl.crawlId, e)}
                        title="Save as .seocrawl"
                      >
                        <Save className="size-3.5" strokeWidth={1.75} />
                      </Button>
                    )}
                  </div>
                </button>
              );
            })}
          </div>
        )}
      </Panel>
    </div>
  );
}
