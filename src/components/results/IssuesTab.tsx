import { useCallback, useMemo, useState } from "react";
import { getIssues } from "@/lib/commands";
import { useServerData } from "@/hooks/useServerData";
import { DataTable } from "./DataTable";
import { createIssueColumns } from "./columns/issueColumns";
import { IssueFilterBar } from "./filters/IssueFilterBar";
import type {
  IssueFilters,
  IssueRow,
  PaginatedResponse,
  PaginationParams,
} from "@/types";

interface IssuesTabProps {
  crawlId: string;
  onPageClick?: (pageId: number) => void;
}

export function IssuesTab({ crawlId, onPageClick }: IssuesTabProps) {
  const [filters, setFilters] = useState<IssueFilters>({
    severity: null,
    category: null,
    ruleId: null,
  });

  const fetcher = useCallback(
    (
      pagination: PaginationParams,
      f: IssueFilters,
    ): Promise<PaginatedResponse<IssueRow>> => getIssues(crawlId, pagination, f),
    [crawlId],
  );

  const columns = useMemo(() => createIssueColumns(onPageClick), [onPageClick]);

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

  return (
    <div className="flex h-full flex-col">
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
