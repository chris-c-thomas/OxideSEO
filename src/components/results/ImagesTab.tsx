import { useCallback, useMemo, useState } from "react";
import { getLinks } from "@/lib/commands";
import { useServerData } from "@/hooks/useServerData";
import { DataTable } from "./DataTable";
import { imageColumns } from "./columns/imageColumns";
import { ImageFilterBar } from "./filters/ImageFilterBar";
import type { LinkFilters, LinkRow, PaginatedResponse, PaginationParams } from "@/types";

interface ImagesTabProps {
  crawlId: string;
}

export function ImagesTab({ crawlId }: ImagesTabProps) {
  // Pre-apply linkType: "img" to only show image links
  const [filters, setFilters] = useState<LinkFilters>({
    linkType: "img",
    isInternal: null,
    isBroken: null,
    anchorTextMissing: null,
  });

  const fetcher = useCallback(
    (pagination: PaginationParams, f: LinkFilters): Promise<PaginatedResponse<LinkRow>> =>
      getLinks(crawlId, pagination, f),
    [crawlId],
  );

  const columns = useMemo(() => imageColumns, []);

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
      <ImageFilterBar filters={filters} onChange={setFilters} />
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
