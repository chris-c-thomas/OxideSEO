/**
 * Dashboard view: recent crawls, summary metrics, and quick-start actions.
 */

import { useEffect, useState } from "react";
import type { AppView } from "@/App";
import {
  deleteCrawl,
  getRecentCrawls,
  openCrawlFile,
  pauseCrawl,
  resumeCrawl,
  rerunCrawl,
  saveCrawlFile,
  stopCrawl,
} from "@/lib/commands";
import { cn, formatNumber } from "@/lib/utils";
import { useCrawlStore } from "@/stores/crawlStore";
import { toast } from "sonner";
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
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { Checkbox } from "@/components/ui/checkbox";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Globe,
  AlertTriangle,
  AlertCircle,
  Clock,
  FolderOpen,
  PlusCircle,
  GitCompare,
  MoreHorizontal,
  Pause,
  Play,
  Square,
  RotateCw,
  Trash2,
  Save,
  Search,
  Activity,
  Eye,
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

interface PendingAction {
  type: "stop" | "delete";
  crawlId: string;
}

export function Dashboard({ onNavigate }: DashboardProps) {
  const [recentCrawls, setRecentCrawls] = useState<CrawlSummary[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [compareMode, setCompareMode] = useState(false);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [pendingAction, setPendingAction] = useState<PendingAction | null>(null);

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

  const handleSaveCrawl = async (crawlId: string) => {
    try {
      await saveCrawlFile(crawlId);
      toast.success("Crawl saved to file.");
    } catch (err) {
      toast.error(`Failed to save crawl: ${String(err)}`);
    }
  };

  const handlePause = async (crawlId: string) => {
    try {
      await pauseCrawl(crawlId);
      loadCrawls();
    } catch (err) {
      toast.error(`Failed to pause crawl: ${String(err)}`);
    }
  };

  const handleResume = async (crawlId: string) => {
    try {
      await resumeCrawl(crawlId);
      loadCrawls();
    } catch (err) {
      toast.error(`Failed to resume crawl: ${String(err)}`);
    }
  };

  const handleConfirmAction = async () => {
    if (!pendingAction) return;
    const { type, crawlId } = pendingAction;
    setPendingAction(null);

    switch (type) {
      case "stop":
        try {
          await stopCrawl(crawlId);
          toast.success("Crawl stopped.");
          loadCrawls();
        } catch (err) {
          toast.error(`Failed to stop crawl: ${String(err)}`);
        }
        break;
      case "delete":
        try {
          await deleteCrawl(crawlId);
          setRecentCrawls((prev) => prev.filter((c) => c.crawlId !== crawlId));
          setSelectedIds((prev) => {
            const next = new Set(prev);
            next.delete(crawlId);
            return next;
          });
          const { activeCrawlId, clearCrawl } = useCrawlStore.getState();
          if (crawlId === activeCrawlId) {
            clearCrawl();
            onNavigate("dashboard");
          }
          toast.success("Crawl deleted.");
        } catch (err) {
          toast.error(`Failed to delete crawl: ${String(err)}`);
        }
        break;
      default: {
        const _exhaustive: never = type;
        void _exhaustive;
      }
    }
  };

  const handleRerun = async (crawlId: string) => {
    try {
      const newCrawlId = await rerunCrawl(crawlId);
      toast.success("Crawl re-started with same configuration.");
      onNavigate("crawl-monitor", newCrawlId);
    } catch (err) {
      toast.error(`Failed to re-run crawl: ${String(err)}`);
    }
  };

  const isActive = (crawl: CrawlSummary) =>
    crawl.status === "running" || crawl.status === "paused";

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
                  className={cn(
                    "flex w-full items-center gap-3 rounded-[var(--radius-sm)] px-3 py-2.5 text-left transition-colors",
                    selectedIds.has(crawl.crawlId)
                      ? "bg-brand-subtle"
                      : "hover:bg-bg-hover",
                    compareMode && !selectable && "opacity-40",
                  )}
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
                    <div className="flex items-center gap-2">
                      {crawl.status === "running" && (
                        <span className="bg-status-running size-2 shrink-0 animate-pulse rounded-full" />
                      )}
                      <p className="text-fg-default truncate text-sm font-medium">
                        {crawl.startUrl}
                      </p>
                    </div>
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
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button
                            variant="ghost"
                            size="sm"
                            className="size-7 p-0"
                            aria-label="Crawl actions"
                            onClick={(e) => e.stopPropagation()}
                          >
                            <MoreHorizontal className="size-4" strokeWidth={1.75} />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent
                          align="end"
                          onClick={(e) => e.stopPropagation()}
                        >
                          {/* Active crawl actions */}
                          {crawl.status === "running" && (
                            <DropdownMenuItem onClick={() => handlePause(crawl.crawlId)}>
                              <Pause className="size-4" strokeWidth={1.75} />
                              Pause
                            </DropdownMenuItem>
                          )}
                          {crawl.status === "paused" && (
                            <DropdownMenuItem onClick={() => handleResume(crawl.crawlId)}>
                              <Play className="size-4" strokeWidth={1.75} />
                              Resume
                            </DropdownMenuItem>
                          )}
                          {isActive(crawl) && (
                            <DropdownMenuItem
                              onClick={() =>
                                setPendingAction({ type: "stop", crawlId: crawl.crawlId })
                              }
                            >
                              <Square className="size-4" strokeWidth={1.75} />
                              Stop
                            </DropdownMenuItem>
                          )}
                          {isActive(crawl) && (
                            <DropdownMenuItem
                              onClick={() => onNavigate("crawl-monitor", crawl.crawlId)}
                            >
                              <Activity className="size-4" strokeWidth={1.75} />
                              View Monitor
                            </DropdownMenuItem>
                          )}

                          {/* Completed/stopped crawl actions */}
                          {!isActive(crawl) && (
                            <DropdownMenuItem onClick={() => handleRerun(crawl.crawlId)}>
                              <RotateCw className="size-4" strokeWidth={1.75} />
                              Re-run
                            </DropdownMenuItem>
                          )}
                          {!isActive(crawl) && (
                            <DropdownMenuItem
                              onClick={() => onNavigate("results", crawl.crawlId)}
                            >
                              <Eye className="size-4" strokeWidth={1.75} />
                              View Results
                            </DropdownMenuItem>
                          )}

                          {/* Common actions */}
                          <DropdownMenuItem
                            onClick={() => handleSaveCrawl(crawl.crawlId)}
                          >
                            <Save className="size-4" strokeWidth={1.75} />
                            Export
                          </DropdownMenuItem>

                          <DropdownMenuSeparator />

                          <DropdownMenuItem
                            className="text-danger focus:text-danger"
                            onClick={() =>
                              setPendingAction({ type: "delete", crawlId: crawl.crawlId })
                            }
                          >
                            <Trash2 className="size-4" strokeWidth={1.75} />
                            Delete
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    )}
                  </div>
                </button>
              );
            })}
          </div>
        )}
      </Panel>

      <ConfirmDialog
        open={pendingAction !== null}
        title={pendingAction?.type === "delete" ? "Delete Crawl" : "Stop Crawl"}
        description={
          pendingAction?.type === "delete"
            ? "This will permanently delete the crawl and all associated data. This action cannot be undone."
            : "This will stop the crawl. In-flight requests will complete but no new URLs will be fetched."
        }
        confirmLabel={pendingAction?.type === "delete" ? "Delete" : "Stop"}
        variant="destructive"
        onConfirm={handleConfirmAction}
        onCancel={() => setPendingAction(null)}
      />
    </div>
  );
}
