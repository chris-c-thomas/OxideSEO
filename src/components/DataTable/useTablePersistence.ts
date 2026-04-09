/**
 * Table state persistence hook.
 *
 * Syncs TanStack Table column order, visibility, sizing, and sorting
 * state to the uiStore for cross-session persistence.
 */

import { useCallback, useEffect, useRef } from "react";
import type {
  ColumnOrderState,
  ColumnSizingState,
  SortingState,
  VisibilityState,
} from "@tanstack/react-table";
import { useUiStore, type TablePersistedState } from "@/stores/uiStore";

interface UseTablePersistenceOptions {
  viewId: string;
}

interface UseTablePersistenceResult {
  initialState: Partial<TablePersistedState>;
  persistColumnOrder: (order: ColumnOrderState) => void;
  persistColumnVisibility: (visibility: VisibilityState) => void;
  persistColumnSizing: (sizing: ColumnSizingState) => void;
  persistSorting: (sorting: SortingState) => void;
}

export function useTablePersistence({
  viewId,
}: UseTablePersistenceOptions): UseTablePersistenceResult {
  const getTableState = useUiStore((s) => s.getTableState);
  const setTableState = useUiStore((s) => s.setTableState);

  const initialState = getTableState(viewId) ?? {};

  // Debounce writes to localStorage.
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const debouncedSet = useCallback(
    (partial: Partial<TablePersistedState>) => {
      clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => {
        setTableState(viewId, partial);
      }, 500);
    },
    [viewId, setTableState],
  );

  useEffect(() => {
    return () => clearTimeout(timerRef.current);
  }, []);

  return {
    initialState,
    persistColumnOrder: (columnOrder) => debouncedSet({ columnOrder }),
    persistColumnVisibility: (columnVisibility) => debouncedSet({ columnVisibility }),
    persistColumnSizing: (columnSizing) => debouncedSet({ columnSizing }),
    persistSorting: (sorting) => debouncedSet({ sorting }),
  };
}
