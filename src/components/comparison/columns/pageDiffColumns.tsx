import { type ColumnDef } from "@tanstack/react-table";
import type { PageDiffRow } from "@/types";
import { Badge } from "@/components/ui/badge";

function diffTypeBadge(diffType: string) {
  switch (diffType) {
    case "new":
      return <Badge variant="default">New</Badge>;
    case "removed":
      return <Badge variant="destructive">Removed</Badge>;
    case "status_code_changed":
      return <Badge variant="secondary">Changed</Badge>;
    default:
      return <Badge variant="outline">{diffType}</Badge>;
  }
}

export const pageDiffColumns: ColumnDef<PageDiffRow, unknown>[] = [
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
    accessorKey: "diffType",
    header: "Change",
    size: 100,
    cell: ({ getValue }) => diffTypeBadge(getValue<string>()),
  },
  {
    accessorKey: "baseStatusCode",
    header: "Base Status",
    size: 90,
    cell: ({ getValue }) => {
      const code = getValue<number | null>();
      return code != null ? (
        <span className="tabular-nums">{code}</span>
      ) : (
        <span className="opacity-40">--</span>
      );
    },
  },
  {
    accessorKey: "compareStatusCode",
    header: "Compare Status",
    size: 90,
    cell: ({ getValue }) => {
      const code = getValue<number | null>();
      return code != null ? (
        <span className="tabular-nums">{code}</span>
      ) : (
        <span className="opacity-40">--</span>
      );
    },
  },
  {
    accessorKey: "baseTitle",
    header: "Base Title",
    size: 200,
    cell: ({ getValue }) => {
      const v = getValue<string | null>();
      return v ? (
        <span className="text-xs">{v}</span>
      ) : (
        <span className="opacity-40">--</span>
      );
    },
  },
  {
    accessorKey: "compareTitle",
    header: "Compare Title",
    size: 200,
    cell: ({ getValue }) => {
      const v = getValue<string | null>();
      return v ? (
        <span className="text-xs">{v}</span>
      ) : (
        <span className="opacity-40">--</span>
      );
    },
  },
];
