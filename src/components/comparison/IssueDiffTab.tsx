/**
 * Issue diff tab: paginated table of new and resolved issues between two crawls.
 */

import { useCallback, useMemo, useState } from "react";
import { getIssueDiffs } from "@/lib/commands";
import { useServerData } from "@/hooks/useServerData";
import { DataTable } from "@/components/results/DataTable";
import { issueDiffColumns } from "./columns/issueDiffColumns";
import type {
  IssueDiffFilters,
  IssueDiffRow,
  PaginatedResponse,
  PaginationParams,
} from "@/types";

interface IssueDiffTabProps {
  baseCrawlId: string;
  compareCrawlId: string;
}

export function IssueDiffTab({ baseCrawlId, compareCrawlId }: IssueDiffTabProps) {
  const [filters, setFilters] = useState<IssueDiffFilters>({
    diffType: null,
  });

  const fetcher = useCallback(
    (
      pagination: PaginationParams,
      f: IssueDiffFilters,
    ): Promise<PaginatedResponse<IssueDiffRow>> =>
      getIssueDiffs(baseCrawlId, compareCrawlId, pagination, f),
    [baseCrawlId, compareCrawlId],
  );

  const columns = useMemo(() => issueDiffColumns, []);

  const {
    items,
    total,
    isLoading,
    isLoadingMore,
    hasMore,
    loadMore,
    sorting,
    setSorting,
    error,
  } = useServerData(fetcher, { filters });

  return (
    <div className="flex h-full flex-col">
      {/* Filter bar */}
      <div
        className="flex items-center gap-3 border-b px-4 py-2"
        style={{ borderColor: "var(--color-border)" }}
      >
        <select
          value={filters.diffType ?? ""}
          onChange={(e) =>
            setFilters({
              diffType: (e.target.value || null) as IssueDiffFilters["diffType"],
            })
          }
          className="rounded-md border bg-transparent px-2 py-1 text-sm"
          style={{ borderColor: "var(--color-border)" }}
        >
          <option value="">All issues</option>
          <option value="new">New issues</option>
          <option value="resolved">Resolved issues</option>
        </select>
        <span
          className="ml-auto text-xs tabular-nums"
          style={{ color: "var(--color-muted-foreground)" }}
        >
          {total.toLocaleString()} issues
        </span>
      </div>
      <div className="min-h-0 flex-1">
        {error ? (
          <div className="flex h-32 items-center justify-center">
            <p style={{ color: "var(--color-severity-error)" }}>
              Failed to load issue diffs: {error}
            </p>
          </div>
        ) : (
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
        )}
      </div>
    </div>
  );
}
