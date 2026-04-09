/**
 * Sortable column header with resize handle.
 *
 * Renders the header label with sort direction indicator and a
 * draggable resize handle on the right edge.
 */

import type { Column } from "@tanstack/react-table";
import { ArrowDown, ArrowUp, ArrowUpDown } from "lucide-react";
import { cn } from "@/lib/utils";

interface DataTableColumnHeaderProps<TData> {
  column: Column<TData, unknown>;
  title: string;
  className?: string;
}

export function DataTableColumnHeader<TData>({
  column,
  title,
  className,
}: DataTableColumnHeaderProps<TData>) {
  const canSort = column.getCanSort();
  const sorted = column.getIsSorted();

  return (
    <div
      className={cn(
        "flex items-center gap-1",
        canSort && "cursor-pointer select-none",
        className,
      )}
      onClick={canSort ? column.getToggleSortingHandler() : undefined}
    >
      <span className="text-fg-muted truncate text-xs font-medium">{title}</span>
      {canSort && (
        <span className="inline-flex size-4 shrink-0 items-center justify-center">
          {sorted === "asc" ? (
            <ArrowUp className="text-fg-default size-3" strokeWidth={1.75} />
          ) : sorted === "desc" ? (
            <ArrowDown className="text-fg-default size-3" strokeWidth={1.75} />
          ) : (
            <ArrowUpDown
              className="text-fg-subtle size-3 opacity-50"
              strokeWidth={1.75}
            />
          )}
        </span>
      )}
    </div>
  );
}
