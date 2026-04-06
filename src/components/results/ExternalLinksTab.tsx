/**
 * External Links tab: paginated table of external link check results
 * with broken/working filter.
 */

import { useCallback, useMemo, useState } from "react";
import { getExternalLinks } from "@/lib/commands";
import { useServerData } from "@/hooks/useServerData";
import { DataTable } from "./DataTable";
import { externalLinkColumns } from "./columns/externalLinkColumns";
import type { ExternalLinkRow, PaginatedResponse, PaginationParams } from "@/types";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { X } from "lucide-react";

interface ExternalLinksTabProps {
  crawlId: string;
}

interface ExternalLinkFilters {
  isBroken: boolean | null;
}

export function ExternalLinksTab({ crawlId }: ExternalLinksTabProps) {
  const [filters, setFilters] = useState<ExternalLinkFilters>({
    isBroken: null,
  });

  const fetcher = useCallback(
    (
      pagination: PaginationParams,
      f: ExternalLinkFilters,
    ): Promise<PaginatedResponse<ExternalLinkRow>> =>
      getExternalLinks(crawlId, pagination, f.isBroken ?? undefined),
    [crawlId],
  );

  const columns = useMemo(() => externalLinkColumns, []);

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
      <div className="flex items-center gap-2 pb-4">
        <Select
          value={
            filters.isBroken === true
              ? "broken"
              : filters.isBroken === false
                ? "working"
                : "all"
          }
          onValueChange={(val) =>
            setFilters({
              isBroken: val === "broken" ? true : val === "working" ? false : null,
            })
          }
        >
          <SelectTrigger className="h-8 w-36 text-sm">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Status</SelectItem>
            <SelectItem value="broken">Broken Only</SelectItem>
            <SelectItem value="working">Working Only</SelectItem>
          </SelectContent>
        </Select>

        {filters.isBroken !== null && (
          <Button
            variant="ghost"
            size="sm"
            className="h-8 px-2"
            onClick={() => setFilters({ isBroken: null })}
          >
            <X className="mr-1 h-3 w-3" />
            Clear
          </Button>
        )}

        <span
          className="ml-auto text-xs tabular-nums"
          style={{ color: "var(--color-muted-foreground)" }}
        >
          {total.toLocaleString()} external links
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
