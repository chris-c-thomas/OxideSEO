/**
 * Metadata diff tab: pages where title or meta description changed between crawls.
 */

import { useCallback, useState } from "react";
import { getMetadataDiffs } from "@/lib/commands";
import { useServerData } from "@/hooks/useServerData";
import type {
  MetadataDiffFilters,
  PageDiffRow,
  PaginatedResponse,
  PaginationParams,
} from "@/types";

interface MetadataDiffTabProps {
  baseCrawlId: string;
  compareCrawlId: string;
}

export function MetadataDiffTab({ baseCrawlId, compareCrawlId }: MetadataDiffTabProps) {
  const [filters, setFilters] = useState<MetadataDiffFilters>({
    urlSearch: null,
  });

  const fetcher = useCallback(
    (
      pagination: PaginationParams,
      f: MetadataDiffFilters,
    ): Promise<PaginatedResponse<PageDiffRow>> =>
      getMetadataDiffs(baseCrawlId, compareCrawlId, pagination, f),
    [baseCrawlId, compareCrawlId],
  );

  const { items, total, isLoading, isLoadingMore, hasMore, loadMore, error } =
    useServerData(fetcher, {
      filters,
    });

  return (
    <div className="flex h-full flex-col">
      {/* Filter bar */}
      <div
        className="flex items-center gap-3 border-b px-4 py-2"
        style={{ borderColor: "var(--color-border)" }}
      >
        <input
          type="text"
          placeholder="Search URLs..."
          value={filters.urlSearch ?? ""}
          onChange={(e) => setFilters({ urlSearch: e.target.value || null })}
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
        <div
          className="custom-scrollbar overflow-auto"
          style={{ maxHeight: "calc(100vh - 260px)" }}
        >
          {error ? (
            <div className="flex h-32 items-center justify-center">
              <p style={{ color: "var(--color-severity-error)" }}>
                Failed to load metadata diffs: {error}
              </p>
            </div>
          ) : isLoading ? (
            <div className="flex h-32 items-center justify-center">
              <p style={{ color: "var(--color-muted-foreground)" }}>Loading...</p>
            </div>
          ) : items.length === 0 ? (
            <div className="flex h-32 items-center justify-center">
              <p style={{ color: "var(--color-muted-foreground)" }}>
                No metadata changes found.
              </p>
            </div>
          ) : (
            <table className="w-full text-sm">
              <thead>
                <tr style={{ backgroundColor: "var(--color-muted)" }}>
                  <th className="px-3 py-2 text-left font-medium">URL</th>
                  <th className="px-3 py-2 text-left font-medium">Base Title</th>
                  <th className="px-3 py-2 text-left font-medium">Compare Title</th>
                  <th className="px-3 py-2 text-left font-medium">Base Meta Desc</th>
                  <th className="px-3 py-2 text-left font-medium">Compare Meta Desc</th>
                </tr>
              </thead>
              <tbody>
                {items.map((row, i) => (
                  <tr
                    key={`${row.url}-${i}`}
                    className="border-t"
                    style={{ borderColor: "var(--color-border)" }}
                  >
                    <td className="px-3 py-1.5 font-mono text-xs" title={row.url}>
                      {row.url.length > 60 ? row.url.slice(0, 59) + "\u2026" : row.url}
                    </td>
                    <td className="px-3 py-1.5 text-xs">{row.baseTitle ?? "--"}</td>
                    <td
                      className="px-3 py-1.5 text-xs"
                      style={{
                        backgroundColor:
                          row.baseTitle !== row.compareTitle
                            ? "var(--color-muted)"
                            : undefined,
                      }}
                    >
                      {row.compareTitle ?? "--"}
                    </td>
                    <td className="px-3 py-1.5 text-xs">{row.baseMetaDesc ?? "--"}</td>
                    <td
                      className="px-3 py-1.5 text-xs"
                      style={{
                        backgroundColor:
                          row.baseMetaDesc !== row.compareMetaDesc
                            ? "var(--color-muted)"
                            : undefined,
                      }}
                    >
                      {row.compareMetaDesc ?? "--"}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
          {hasMore && !isLoadingMore && (
            <div className="flex justify-center py-3">
              <button
                onClick={loadMore}
                className="rounded-md border px-4 py-1.5 text-xs"
                style={{ borderColor: "var(--color-border)" }}
              >
                Load more
              </button>
            </div>
          )}
          {isLoadingMore && (
            <p
              className="py-3 text-center text-xs"
              style={{ color: "var(--color-muted-foreground)" }}
            >
              Loading more...
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
