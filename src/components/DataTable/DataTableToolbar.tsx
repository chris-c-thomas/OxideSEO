/**
 * Data table toolbar with search, column visibility, and density toggle.
 *
 * Sits above the DataTable and provides controls for filtering,
 * column management, and display density.
 */

import type { Table } from "@tanstack/react-table";
import { Columns3, Search } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { useUiStore, type Density } from "@/stores/uiStore";

interface DataTableToolbarProps<TData> {
  table: Table<TData>;
  totalRows: number;
  loadedRows: number;
  searchValue?: string;
  onSearchChange?: (value: string) => void;
  searchPlaceholder?: string;
  children?: React.ReactNode;
}

const DENSITY_LABELS: Record<Density, string> = {
  compact: "Compact",
  default: "Default",
  comfortable: "Comfortable",
};

export function DataTableToolbar<TData>({
  table,
  totalRows,
  loadedRows,
  searchValue,
  onSearchChange,
  searchPlaceholder = "Search...",
  children,
}: DataTableToolbarProps<TData>) {
  const density = useUiStore((s) => s.density);
  const setDensity = useUiStore((s) => s.setDensity);

  return (
    <div className="flex items-center gap-2 pb-2">
      {/* Search */}
      {onSearchChange && (
        <div className="relative">
          <Search
            className="text-fg-subtle absolute top-1/2 left-2 size-3.5 -translate-y-1/2"
            strokeWidth={1.75}
          />
          <Input
            value={searchValue ?? ""}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder={searchPlaceholder}
            className="h-7 w-[200px] pl-7 text-xs"
          />
        </div>
      )}

      {/* Filter slots */}
      {children}

      {/* Spacer */}
      <div className="flex-1" />

      {/* Row count */}
      <span className="text-fg-subtle text-[0.6875rem] tabular-nums">
        {loadedRows.toLocaleString()} of {totalRows.toLocaleString()} rows
      </span>

      {/* Density toggle */}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline" size="sm" className="h-7 text-xs">
            {DENSITY_LABELS[density]}
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuLabel>Row density</DropdownMenuLabel>
          <DropdownMenuSeparator />
          {(Object.keys(DENSITY_LABELS) as Density[]).map((d) => (
            <DropdownMenuCheckboxItem
              key={d}
              checked={density === d}
              onCheckedChange={() => setDensity(d)}
            >
              {DENSITY_LABELS[d]}
            </DropdownMenuCheckboxItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>

      {/* Column visibility */}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline" size="sm" className="h-7 gap-1 text-xs">
            <Columns3 className="size-3.5" strokeWidth={1.75} />
            Columns
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="max-h-[300px] overflow-y-auto">
          <DropdownMenuLabel>Toggle columns</DropdownMenuLabel>
          <DropdownMenuSeparator />
          {table
            .getAllColumns()
            .filter((col) => col.getCanHide())
            .map((col) => (
              <DropdownMenuCheckboxItem
                key={col.id}
                checked={col.getIsVisible()}
                onCheckedChange={(value) => col.toggleVisibility(!!value)}
              >
                {typeof col.columnDef.header === "string" ? col.columnDef.header : col.id}
              </DropdownMenuCheckboxItem>
            ))}
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
