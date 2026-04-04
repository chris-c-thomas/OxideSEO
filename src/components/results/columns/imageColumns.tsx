import { type ColumnDef } from "@tanstack/react-table";
import type { LinkRow } from "@/types";
import { Badge } from "@/components/ui/badge";
import { Check } from "lucide-react";

export const imageColumns: ColumnDef<LinkRow, unknown>[] = [
  {
    accessorKey: "targetUrl",
    header: "Image URL",
    size: 350,
    cell: ({ getValue }) => (
      <span className="font-mono text-xs" title={getValue<string>()}>
        {getValue<string>()}
      </span>
    ),
  },
  {
    accessorKey: "sourcePage",
    header: "Source Page",
    size: 250,
    cell: ({ getValue }) => (
      <span className="text-xs tabular-nums">#{getValue<number>()}</span>
    ),
  },
  {
    accessorKey: "anchorText",
    header: "Alt Text",
    size: 200,
    enableSorting: false,
    cell: ({ getValue }) => {
      const val = getValue<string | null>();
      if (!val || val.trim() === "") {
        return <Badge variant="destructive">missing</Badge>;
      }
      return <span className="text-xs">{val}</span>;
    },
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
];
