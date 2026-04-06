/**
 * Sitemap tab: cross-reference report showing URLs in sitemap vs. crawl.
 */

import { useCallback, useEffect, useMemo, useState } from "react";
import { getSitemapReport } from "@/lib/commands";
import { DataTable } from "./DataTable";
import { sitemapColumns } from "./columns/sitemapColumns";
import type { SitemapReportEntry } from "@/types";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface SitemapTabProps {
  crawlId: string;
}

type StatusFilter = "all" | "in_sitemap_not_crawled" | "crawled_not_in_sitemap";

export function SitemapTab({ crawlId }: SitemapTabProps) {
  const [allEntries, setAllEntries] = useState<SitemapReportEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");

  useEffect(() => {
    let stale = false;
    setIsLoading(true);
    setError(null);
    getSitemapReport(crawlId)
      .then((entries) => {
        if (!stale) setAllEntries(entries);
      })
      .catch((err) => {
        if (!stale) setError(String(err));
      })
      .finally(() => {
        if (!stale) setIsLoading(false);
      });
    return () => {
      stale = true;
    };
  }, [crawlId]);

  const filtered = useMemo(() => {
    if (statusFilter === "all") return allEntries;
    return allEntries.filter((e) => e.status === statusFilter);
  }, [allEntries, statusFilter]);

  const columns = useMemo(() => sitemapColumns, []);

  const noop = useCallback(() => {}, []);

  if (error) {
    return (
      <div className="flex h-full items-center justify-center">
        <span className="text-sm" style={{ color: "var(--color-severity-error)" }}>
          Failed to load sitemap report: {error}
        </span>
      </div>
    );
  }

  if (!isLoading && allEntries.length === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <span className="text-sm" style={{ color: "var(--color-muted-foreground)" }}>
          No sitemap data available for this crawl. Enable sitemap discovery in crawl
          configuration.
        </span>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-2 pb-4">
        <Select
          value={statusFilter}
          onValueChange={(val) => setStatusFilter(val as StatusFilter)}
        >
          <SelectTrigger className="h-8 w-48 text-sm">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All ({allEntries.length})</SelectItem>
            <SelectItem value="in_sitemap_not_crawled">
              In sitemap, not crawled (
              {allEntries.filter((e) => e.status === "in_sitemap_not_crawled").length})
            </SelectItem>
            <SelectItem value="crawled_not_in_sitemap">
              Crawled, not in sitemap (
              {allEntries.filter((e) => e.status === "crawled_not_in_sitemap").length})
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
      <div className="min-h-0 flex-1">
        <DataTable
          columns={columns}
          data={filtered}
          total={filtered.length}
          isLoading={isLoading}
          isLoadingMore={false}
          hasMore={false}
          onLoadMore={noop}
          sorting={[]}
          onSortingChange={noop}
        />
      </div>
    </div>
  );
}
