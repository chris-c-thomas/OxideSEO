import { useCallback, useEffect, useRef } from "react";
import {
  type ColumnDef,
  type OnChangeFn,
  type SortingState,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import { ArrowDown, ArrowUp, ArrowUpDown, Loader2 } from "lucide-react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

interface DataTableProps<TData> {
  columns: ColumnDef<TData, unknown>[];
  data: TData[];
  total: number;
  isLoading: boolean;
  isLoadingMore: boolean;
  hasMore: boolean;
  onLoadMore: () => void;
  sorting: SortingState;
  onSortingChange: OnChangeFn<SortingState>;
  onRowClick?: (row: TData) => void;
  rowHeight?: number;
}

const OVERSCAN = 20;
const LOAD_MORE_THRESHOLD = 20;

export function DataTable<TData>({
  columns,
  data,
  total,
  isLoading,
  isLoadingMore,
  hasMore,
  onLoadMore,
  sorting,
  onSortingChange,
  onRowClick,
  rowHeight = 36,
}: DataTableProps<TData>) {
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    manualSorting: true,
    state: { sorting },
    onSortingChange,
  });

  const { rows } = table.getRowModel();
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => rowHeight,
    overscan: OVERSCAN,
  });

  // Trigger loadMore when scrolling near the bottom
  const handleScroll = useCallback(() => {
    if (!hasMore || isLoadingMore) return;
    const el = parentRef.current;
    if (!el) return;
    const { scrollTop, scrollHeight, clientHeight } = el;
    const remainingRows = Math.ceil(
      (scrollHeight - scrollTop - clientHeight) / rowHeight,
    );
    if (remainingRows < LOAD_MORE_THRESHOLD) {
      onLoadMore();
    }
  }, [hasMore, isLoadingMore, onLoadMore, rowHeight]);

  useEffect(() => {
    const el = parentRef.current;
    if (!el) return;
    el.addEventListener("scroll", handleScroll, { passive: true });
    return () => el.removeEventListener("scroll", handleScroll);
  }, [handleScroll]);

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader2
          className="h-6 w-6 animate-spin"
          style={{ color: "var(--color-muted-foreground)" }}
        />
      </div>
    );
  }

  if (data.length === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-sm" style={{ color: "var(--color-muted-foreground)" }}>
          No results found.
        </p>
      </div>
    );
  }

  const virtualItems = virtualizer.getVirtualItems();

  return (
    <div className="flex h-full flex-col">
      <div
        className="text-xs tabular-nums"
        style={{ color: "var(--color-muted-foreground)", padding: "0 0 8px" }}
      >
        {data.length.toLocaleString()} of {total.toLocaleString()} rows loaded
      </div>

      <div
        ref={parentRef}
        className="flex-1 overflow-auto rounded-md border"
        style={{ borderColor: "var(--color-border)" }}
      >
        <Table>
          <TableHeader
            className="sticky top-0 z-10"
            style={{ background: "var(--color-background)" }}
          >
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  const canSort = header.column.getCanSort();
                  const sorted = header.column.getIsSorted();
                  return (
                    <TableHead
                      key={header.id}
                      style={{
                        width: header.getSize() !== 150 ? header.getSize() : undefined,
                      }}
                      className={canSort ? "cursor-pointer select-none" : ""}
                      onClick={
                        canSort ? header.column.getToggleSortingHandler() : undefined
                      }
                    >
                      <div className="flex items-center gap-1">
                        {header.isPlaceholder
                          ? null
                          : flexRender(
                              header.column.columnDef.header,
                              header.getContext(),
                            )}
                        {canSort && (
                          <span className="inline-flex h-4 w-4 items-center justify-center">
                            {sorted === "asc" ? (
                              <ArrowUp className="h-3 w-3" />
                            ) : sorted === "desc" ? (
                              <ArrowDown className="h-3 w-3" />
                            ) : (
                              <ArrowUpDown className="h-3 w-3 opacity-30" />
                            )}
                          </span>
                        )}
                      </div>
                    </TableHead>
                  );
                })}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {/* Spacer for virtual rows above */}
            {virtualItems.length > 0 && (virtualItems[0]?.start ?? 0) > 0 && (
              <tr>
                <td style={{ height: virtualItems[0]?.start ?? 0, padding: 0 }} />
              </tr>
            )}
            {virtualItems.map((virtualRow) => {
              const row = rows[virtualRow.index];
              if (!row) return null;
              return (
                <TableRow
                  key={row.id}
                  data-index={virtualRow.index}
                  className={onRowClick ? "cursor-pointer" : ""}
                  onClick={onRowClick ? () => onRowClick(row.original) : undefined}
                  style={{ height: rowHeight }}
                >
                  {row.getVisibleCells().map((cell) => (
                    <TableCell key={cell.id} className="truncate px-4 py-1.5">
                      {flexRender(cell.column.columnDef.cell, cell.getContext())}
                    </TableCell>
                  ))}
                </TableRow>
              );
            })}
            {/* Spacer for virtual rows below */}
            {virtualItems.length > 0 && (
              <tr>
                <td
                  style={{
                    height: virtualizer.getTotalSize() - (virtualItems.at(-1)?.end ?? 0),
                    padding: 0,
                  }}
                />
              </tr>
            )}
          </TableBody>
        </Table>

        {isLoadingMore && (
          <div className="flex items-center justify-center py-2">
            <Loader2
              className="h-4 w-4 animate-spin"
              style={{ color: "var(--color-muted-foreground)" }}
            />
            <span
              className="ml-2 text-xs"
              style={{ color: "var(--color-muted-foreground)" }}
            >
              Loading more...
            </span>
          </div>
        )}
      </div>
    </div>
  );
}
