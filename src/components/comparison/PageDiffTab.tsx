/**
 * Page diff tab: paginated table of pages that were added, removed, or changed status.
 */

import { useCallback, useMemo, useState } from "react";
import { getPageDiffs } from "@/lib/commands";
import { useServerData } from "@/hooks/useServerData";
import { DataTable } from "@/components/results/DataTable";
import { pageDiffColumns } from "./columns/pageDiffColumns";
import type {
  PageDiffFilters,
  PageDiffRow,
  PaginatedResponse,
  PaginationParams,
} from "@/types";

interface PageDiffTabProps {
  baseCrawlId: string;
  compareCrawlId: string;
}

export function PageDiffTab({ baseCrawlId, compareCrawlId }: PageDiffTabProps) {
  const [filters, setFilters] = useState<PageDiffFilters>({
    diffType: null,
    urlSearch: null,
  });

  const fetcher = useCallback(
    (
      pagination: PaginationParams,
      f: PageDiffFilters,
    ): Promise<PaginatedResponse<PageDiffRow>> =>
      getPageDiffs(baseCrawlId, compareCrawlId, pagination, f),
    [baseCrawlId, compareCrawlId],
  );

  const columns = useMemo(() => pageDiffColumns, []);

  const {
    items,
    total,
    isLoading,
    isLoadingMore,
    hasMore,
    loadMore,
    sorting,
    setSorting,
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
            setFilters((f) => ({
              ...f,
              diffType: (e.target.value || null) as PageDiffFilters["diffType"],
            }))
          }
          className="rounded-md border bg-transparent px-2 py-1 text-sm"
          style={{ borderColor: "var(--color-border)" }}
        >
          <option value="">All changes</option>
          <option value="new">New pages</option>
          <option value="removed">Removed pages</option>
          <option value="status_code_changed">Status changed</option>
        </select>
        <input
          type="text"
          placeholder="Search URLs..."
          value={filters.urlSearch ?? ""}
          onChange={(e) =>
            setFilters((f) => ({ ...f, urlSearch: e.target.value || null }))
          }
          className="rounded-md border bg-transparent px-2 py-1 text-sm"
          style={{ borderColor: "var(--color-border)", minWidth: "200px" }}
        />
        <span
          className="ml-auto text-xs tabular-nums"
          style={{ color: "var(--color-muted-foreground)" }}
        >
          {total.toLocaleString()} changes
        </span>
      </div>
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
