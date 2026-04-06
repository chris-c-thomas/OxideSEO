import { type ColumnDef } from "@tanstack/react-table";
import type { ExternalLinkRow } from "@/types";
import { Badge } from "@/components/ui/badge";

export const externalLinkColumns: ColumnDef<ExternalLinkRow, unknown>[] = [
  {
    accessorKey: "sourcePage",
    header: "Source",
    size: 80,
    cell: ({ getValue }) => (
      <span className="text-xs tabular-nums">#{getValue<number>()}</span>
    ),
  },
  {
    accessorKey: "targetUrl",
    header: "Target URL",
    size: 350,
    cell: ({ getValue }) => (
      <span className="font-mono text-xs" title={getValue<string>()}>
        {getValue<string>()}
      </span>
    ),
  },
  {
    accessorKey: "statusCode",
    header: "Status",
    size: 80,
    cell: ({ getValue }) => {
      const code = getValue<number | null>();
      if (code == null) {
        return (
          <Badge variant="destructive" className="text-xs">
            Error
          </Badge>
        );
      }
      if (code >= 400) {
        return (
          <Badge variant="destructive" className="text-xs tabular-nums">
            {code}
          </Badge>
        );
      }
      return <span className="text-xs tabular-nums">{code}</span>;
    },
  },
  {
    accessorKey: "responseTimeMs",
    header: "Time (ms)",
    size: 90,
    cell: ({ getValue }) => {
      const ms = getValue<number | null>();
      if (ms == null) return <span className="text-xs opacity-40">—</span>;
      return <span className="text-xs tabular-nums">{ms.toLocaleString()}</span>;
    },
  },
  {
    accessorKey: "errorMessage",
    header: "Error",
    size: 200,
    enableSorting: false,
    cell: ({ getValue }) => {
      const msg = getValue<string | null>();
      if (!msg) return null;
      return (
        <span
          className="text-xs"
          title={msg}
          style={{ color: "var(--color-severity-error)" }}
        >
          {msg.length > 60 ? `${msg.slice(0, 57)}...` : msg}
        </span>
      );
    },
  },
];
