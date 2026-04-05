import { type ColumnDef } from "@tanstack/react-table";
import type { LinkRow } from "@/types";
import { Badge } from "@/components/ui/badge";
import { Check } from "lucide-react";

export const linkColumns: ColumnDef<LinkRow, unknown>[] = [
  {
    accessorKey: "sourcePage",
    header: "Source",
    size: 250,
    cell: ({ getValue }) => (
      <span className="text-xs tabular-nums">#{getValue<number>()}</span>
    ),
  },
  {
    accessorKey: "targetUrl",
    header: "Target URL",
    size: 300,
    cell: ({ getValue }) => (
      <span className="font-mono text-xs" title={getValue<string>()}>
        {getValue<string>()}
      </span>
    ),
  },
  {
    accessorKey: "anchorText",
    header: "Anchor Text",
    size: 180,
    enableSorting: false,
    cell: ({ getValue }) => {
      const val = getValue<string | null>();
      if (!val) return <span className="text-xs italic opacity-40">empty</span>;
      return <span className="text-xs">{val}</span>;
    },
  },
  {
    accessorKey: "linkType",
    header: "Type",
    size: 80,
    cell: ({ getValue }) => <Badge variant="outline">{getValue<string>()}</Badge>,
  },
  {
    accessorKey: "isInternal",
    header: "Internal",
    size: 80,
    enableSorting: false,
    cell: ({ getValue }) =>
      getValue<boolean>() ? (
        <Check
          className="h-3.5 w-3.5"
          style={{ color: "var(--color-status-completed)" }}
        />
      ) : (
        <span className="text-xs opacity-50">Ext</span>
      ),
  },
  {
    accessorKey: "nofollow",
    header: "Nofollow",
    size: 80,
    enableSorting: false,
    cell: ({ getValue }) =>
      getValue<boolean>() ? <Badge variant="secondary">nofollow</Badge> : null,
  },
];
