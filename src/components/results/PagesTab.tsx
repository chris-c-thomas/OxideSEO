import { useCallback, useMemo, useState } from "react";
import { getCrawlResults } from "@/lib/commands";
import { useServerData } from "@/hooks/useServerData";
import { DataTable } from "./DataTable";
import { pageColumns } from "./columns/pageColumns";
import { PageFilterBar } from "./filters/PageFilterBar";
import type { PageFilters, PageRow, PaginatedResponse, PaginationParams } from "@/types";

interface PagesTabProps {
  crawlId: string;
  onRowClick?: (page: PageRow) => void;
}

export function PagesTab({ crawlId, onRowClick }: PagesTabProps) {
  const [filters, setFilters] = useState<PageFilters>({
    urlSearch: null,
    statusCodes: null,
    minSeverity: null,
    contentType: null,
  });

  const fetcher = useCallback(
    (pagination: PaginationParams, f: PageFilters): Promise<PaginatedResponse<PageRow>> =>
      getCrawlResults(crawlId, pagination, f),
    [crawlId],
  );

  const columns = useMemo(() => pageColumns, []);

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
      <PageFilterBar filters={filters} onChange={setFilters} />
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
          onRowClick={onRowClick}
        />
      </div>
    </div>
  );
}
