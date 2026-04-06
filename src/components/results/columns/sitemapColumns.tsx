import { type ColumnDef } from "@tanstack/react-table";
import type { SitemapReportEntry } from "@/types";
import { Badge } from "@/components/ui/badge";

export const sitemapColumns: ColumnDef<SitemapReportEntry, unknown>[] = [
  {
    accessorKey: "url",
    header: "URL",
    size: 400,
    cell: ({ getValue }) => (
      <span className="font-mono text-xs" title={getValue<string>()}>
        {getValue<string>()}
      </span>
    ),
  },
  {
    accessorKey: "status",
    header: "Status",
    size: 200,
    cell: ({ getValue }) => {
      const status = getValue<string>();
      if (status === "in_sitemap_not_crawled") {
        return (
          <Badge variant="destructive" className="text-xs">
            In sitemap, not crawled
          </Badge>
        );
      }
      if (status === "crawled_not_in_sitemap") {
        return (
          <Badge variant="secondary" className="text-xs">
            Crawled, not in sitemap
          </Badge>
        );
      }
      return <Badge variant="outline">{status}</Badge>;
    },
  },
  {
    accessorKey: "pageStatusCode",
    header: "Status Code",
    size: 100,
    cell: ({ getValue }) => {
      const code = getValue<number | null>();
      if (code == null) return <span className="text-xs opacity-40">—</span>;
      return <span className="text-xs tabular-nums">{code}</span>;
    },
  },
];
