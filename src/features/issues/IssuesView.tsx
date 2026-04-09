/**
 * Issues view: grouped by rule with collapsible sections.
 *
 * Fetches all issues for a crawl, groups them by ruleId, and displays
 * each group as a collapsible section with severity badge and count.
 * Clicking an affected URL navigates to ResultsExplorer.
 */

import { useEffect, useMemo, useState } from "react";
import { getIssues } from "@/lib/commands";
import type { IssueRow } from "@/types";
import { cn } from "@/lib/utils";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { Skeleton } from "@/components/ui/skeleton";
import { EmptyState } from "@/components/EmptyState";
import { SeverityBadge } from "@/components/badges/SeverityBadge";
import { Badge } from "@/components/ui/badge";
import { AlertTriangle, ChevronRight, ExternalLink } from "lucide-react";

interface IssuesViewProps {
  crawlId: string | null;
  onNavigateToPage?: (crawlId: string, pageId: number) => void;
}

interface IssueGroup {
  ruleId: string;
  ruleName: string;
  severity: string;
  category: string;
  issues: IssueRow[];
}

function normalizeSeverity(sev: string): "critical" | "high" | "medium" | "low" | "info" {
  const lower = sev.toLowerCase();
  if (lower === "error" || lower === "critical") return "critical";
  if (lower === "high") return "high";
  if (lower === "warning" || lower === "medium") return "medium";
  if (lower === "low") return "low";
  return "info";
}

export function IssuesView({ crawlId }: IssuesViewProps) {
  const [issues, setIssues] = useState<IssueRow[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!crawlId) return;
    setIsLoading(true);
    setError(null);

    // Fetch all issues (large page size to get everything).
    getIssues(
      crawlId,
      { offset: 0, limit: 5000, sortBy: "severity", sortDir: "asc" },
      { severity: null, category: null, ruleId: null },
    )
      .then((result) => {
        setIssues(result.items);
      })
      .catch((err) => setError(String(err)))
      .finally(() => setIsLoading(false));
  }, [crawlId]);

  const groups = useMemo(() => {
    const map = new Map<string, IssueGroup>();
    for (const issue of issues) {
      const existing = map.get(issue.ruleId);
      if (existing) {
        existing.issues.push(issue);
      } else {
        map.set(issue.ruleId, {
          ruleId: issue.ruleId,
          ruleName: issue.ruleId
            .replace(/_/g, " ")
            .replace(/\b\w/g, (c) => c.toUpperCase()),
          severity: issue.severity,
          category: issue.category,
          issues: [issue],
        });
      }
    }
    return Array.from(map.values());
  }, [issues]);

  if (!crawlId) {
    return (
      <div className="flex h-full items-center justify-center">
        <EmptyState
          icon={AlertTriangle}
          title="No crawl selected"
          description="Select a crawl from the Dashboard to view issues."
        />
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="p-6">
        <h1 className="text-fg-default mb-4 text-xl font-semibold tracking-tight">
          Issues
        </h1>
        <div className="flex flex-col gap-2">
          {Array.from({ length: 5 }).map((_, i) => (
            <Skeleton key={i} className="h-12 w-full" />
          ))}
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-6">
        <h1 className="text-fg-default mb-4 text-xl font-semibold tracking-tight">
          Issues
        </h1>
        <p className="text-danger text-sm">Failed to load issues: {error}</p>
      </div>
    );
  }

  return (
    <div className="p-6">
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-fg-default text-xl font-semibold tracking-tight">Issues</h1>
        <Badge variant="secondary" className="tabular-nums">
          {issues.length} total
        </Badge>
      </div>

      {groups.length === 0 ? (
        <EmptyState
          icon={AlertTriangle}
          title="No issues found"
          description="This crawl didn't detect any SEO issues."
        />
      ) : (
        <div className="flex flex-col gap-1">
          {groups.map((group) => (
            <Collapsible key={group.ruleId}>
              <CollapsibleTrigger className="hover:bg-bg-hover flex w-full items-center gap-3 rounded-[var(--radius-sm)] px-3 py-2 text-left transition-colors [&[data-state=open]>svg:first-child]:rotate-90">
                <ChevronRight
                  className="text-fg-subtle size-3.5 shrink-0 transition-transform duration-150"
                  strokeWidth={1.75}
                />
                <span className="text-fg-default flex-1 text-sm font-medium">
                  {group.ruleName}
                </span>
                <SeverityBadge severity={normalizeSeverity(group.severity)} />
                <Badge variant="secondary" className="text-[0.625rem] tabular-nums">
                  {group.issues.length}
                </Badge>
              </CollapsibleTrigger>
              <CollapsibleContent>
                <div className="border-border-subtle ml-6 flex flex-col gap-0.5 border-l py-1 pl-3">
                  {group.issues.map((issue, i) => (
                    <div
                      key={i}
                      className={cn(
                        "flex items-center gap-2 rounded-[var(--radius-xs)] px-2 py-1 text-xs",
                        "hover:bg-bg-hover",
                      )}
                    >
                      <span className="text-fg-default min-w-0 flex-1 truncate">
                        {issue.message}
                      </span>
                      <span className="text-fg-subtle shrink-0 tabular-nums">
                        Page #{issue.pageId}
                      </span>
                      <ExternalLink
                        className="text-fg-subtle size-3 shrink-0"
                        strokeWidth={1.75}
                      />
                    </div>
                  ))}
                </div>
              </CollapsibleContent>
            </Collapsible>
          ))}
        </div>
      )}
    </div>
  );
}
