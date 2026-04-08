import { type ColumnDef } from "@tanstack/react-table";
import type { IssueDiffRow } from "@/types";
import { Badge } from "@/components/ui/badge";
import { severityColor } from "@/lib/utils";

export const issueDiffColumns: ColumnDef<IssueDiffRow, unknown>[] = [
  {
    accessorKey: "url",
    header: "URL",
    size: 250,
    cell: ({ getValue }) => (
      <span className="font-mono text-xs" title={getValue<string>()}>
        {getValue<string>()}
      </span>
    ),
  },
  {
    accessorKey: "ruleId",
    header: "Rule",
    size: 160,
    cell: ({ getValue }) => (
      <span className="font-mono text-xs">{getValue<string>()}</span>
    ),
  },
  {
    accessorKey: "severity",
    header: "Severity",
    size: 80,
    cell: ({ getValue }) => {
      const sev = getValue<string>();
      return (
        <span
          className="text-xs font-medium"
          style={{ color: severityColor(sev as "error" | "warning" | "info") }}
        >
          {sev}
        </span>
      );
    },
  },
  {
    accessorKey: "category",
    header: "Category",
    size: 90,
    cell: ({ getValue }) => <span className="text-xs">{getValue<string>()}</span>,
  },
  {
    accessorKey: "message",
    header: "Message",
    size: 250,
    cell: ({ getValue }) => <span className="text-xs">{getValue<string>()}</span>,
  },
  {
    accessorKey: "diffType",
    header: "Change",
    size: 90,
    cell: ({ getValue }) => {
      const dt = getValue<string>();
      return dt === "new" ? (
        <Badge variant="destructive">New</Badge>
      ) : (
        <Badge variant="default">Resolved</Badge>
      );
    },
  },
];
