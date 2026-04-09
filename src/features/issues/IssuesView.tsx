/**
 * Issues view: paginated, server-side issue table with filters.
 *
 * Uses useServerData for proper server-side pagination and sorting,
 * respecting the architecture invariant that the frontend never holds
 * the full dataset in memory.
 */

import { useCallback, useMemo, useState } from "react";
import { getIssues } from "@/lib/commands";
import { useServerData } from "@/hooks/useServerData";
import { DataTable } from "@/components/results/DataTable";
import { createIssueColumns } from "@/components/results/columns/issueColumns";
import { IssueFilterBar } from "@/components/results/filters/IssueFilterBar";
import { EmptyState } from "@/components/EmptyState";
import type {
  IssueFilters,
  IssueRow,
  PaginatedResponse,
  PaginationParams,
} from "@/types";
import { AlertTriangle } from "lucide-react";

interface IssuesViewProps {
  crawlId: string | null;
}

export function IssuesView({ crawlId }: IssuesViewProps) {
  const [filters, setFilters] = useState<IssueFilters>({
    severity: null,
    category: null,
    ruleId: null,
  });

  const fetcher = useCallback(
    (
      pagination: PaginationParams,
      f: IssueFilters,
    ): Promise<PaginatedResponse<IssueRow>> => {
      if (!crawlId) return Promise.resolve({ items: [], total: 0, offset: 0, limit: 0 });
      return getIssues(crawlId, pagination, f);
    },
    [crawlId],
  );

  const columns = useMemo(() => createIssueColumns(), []);

  const {
    items,
    total,
    isLoading,
    isLoadingMore,
    hasMore,
    loadMore,
    sorting,
    setSorting,
  } = useServerData(fetcher, { filters, initialSortBy: "severity" });

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

  return (
    <div className="flex h-full flex-col p-4">
      <div className="mb-3">
        <h1 className="text-fg-default text-base font-semibold tracking-tight">Issues</h1>
      </div>
      <IssueFilterBar filters={filters} onChange={setFilters} />
      <div className="min-h-0 flex-1">
        <DataTable
          columns={columns}
          data={items}
          total={total}
          isLoading={isLoading}
          isLoadingMore={isLoadingMore}
          hasMore={hasMore}
          onLoadMore={loadMore}
          sorting={sorting}
          onSortingChange={setSorting}
        />
      </div>
    </div>
  );
}
