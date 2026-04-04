import { type ColumnDef } from "@tanstack/react-table";
import type { PageRow } from "@/types";
import { formatBytes } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";

function statusCodeVariant(
  code: number | null,
): "default" | "secondary" | "destructive" | "outline" {
  if (!code) return "outline";
  if (code >= 400) return "destructive";
  if (code >= 300) return "secondary";
  return "default";
}

export const pageColumns: ColumnDef<PageRow, unknown>[] = [
  {
    accessorKey: "url",
    header: "URL",
    size: 300,
    cell: ({ getValue }) => (
      <span className="font-mono text-xs" title={getValue<string>()}>
        {getValue<string>()}
      </span>
    ),
  },
  {
    accessorKey: "statusCode",
    header: "Status",
    size: 70,
    cell: ({ getValue }) => {
      const code = getValue<number | null>();
      if (code == null) return <span className="text-xs opacity-40">--</span>;
      return <Badge variant={statusCodeVariant(code)}>{code}</Badge>;
    },
  },
  {
    accessorKey: "title",
    header: "Title",
    size: 250,
    cell: ({ getValue }) => {
      const val = getValue<string | null>();
      if (!val) return <span className="text-xs italic opacity-40">missing</span>;
      return <span className="text-xs">{val}</span>;
    },
  },
  {
    accessorKey: "h1",
    header: "H1",
    size: 180,
    enableSorting: false,
    cell: ({ getValue }) => {
      const val = getValue<string | null>();
      if (!val) return <span className="text-xs italic opacity-40">missing</span>;
      return <span className="text-xs">{val}</span>;
    },
  },
  {
    accessorKey: "contentType",
    header: "Type",
    size: 100,
    enableSorting: false,
    cell: ({ getValue }) => {
      const val = getValue<string | null>();
      if (!val) return null;
      const short =
        val.split(";")[0]?.replace("text/", "").replace("application/", "") ?? val;
      return <span className="text-xs">{short}</span>;
    },
  },
  {
    accessorKey: "responseTimeMs",
    header: "Time",
    size: 90,
    cell: ({ getValue }) => {
      const ms = getValue<number | null>();
      if (ms == null) return null;
      const color =
        ms > 1000
          ? "var(--color-severity-error)"
          : ms > 500
            ? "var(--color-severity-warning)"
            : undefined;
      return (
        <span className="text-xs tabular-nums" style={{ color }}>
          {ms}ms
        </span>
      );
    },
  },
  {
    accessorKey: "bodySize",
    header: "Size",
    size: 80,
    cell: ({ getValue }) => {
      const bytes = getValue<number | null>();
      if (bytes == null) return null;
      return <span className="text-xs tabular-nums">{formatBytes(bytes)}</span>;
    },
  },
  {
    accessorKey: "depth",
    header: "Depth",
    size: 60,
    cell: ({ getValue }) => (
      <span className="text-xs tabular-nums">{getValue<number>()}</span>
    ),
  },
];
