import { type ColumnDef } from "@tanstack/react-table";
import type { IssueRow, Severity } from "@/types";
import { Badge } from "@/components/ui/badge";

function severityVariant(
  s: Severity,
): "default" | "destructive" | "secondary" | "outline" {
  switch (s) {
    case "error":
      return "destructive";
    case "warning":
      return "default";
    case "info":
      return "secondary";
  }
}

export function createIssueColumns(
  onPageClick?: (pageId: number) => void,
): ColumnDef<IssueRow, unknown>[] {
  return [
    {
      accessorKey: "severity",
      header: "Severity",
      size: 90,
      cell: ({ getValue }) => {
        const sev = getValue<Severity>();
        return <Badge variant={severityVariant(sev)}>{sev}</Badge>;
      },
    },
    {
      accessorKey: "category",
      header: "Category",
      size: 100,
      cell: ({ getValue }) => <Badge variant="outline">{getValue<string>()}</Badge>,
    },
    {
      accessorKey: "ruleId",
      header: "Rule",
      size: 150,
      cell: ({ getValue }) => (
        <span className="font-mono text-xs">{getValue<string>()}</span>
      ),
    },
    {
      accessorKey: "message",
      header: "Message",
      enableSorting: false,
      cell: ({ getValue }) => <span className="text-xs">{getValue<string>()}</span>,
    },
    {
      accessorKey: "pageId",
      header: "Page",
      size: 80,
      cell: ({ getValue }) => {
        const pageId = getValue<number>();
        if (onPageClick) {
          return (
            <button
              className="text-xs underline"
              style={{ color: "var(--color-primary)" }}
              onClick={(e) => {
                e.stopPropagation();
                onPageClick(pageId);
              }}
            >
              #{pageId}
            </button>
          );
        }
        return <span className="text-xs tabular-nums">#{pageId}</span>;
      },
    },
  ];
}
