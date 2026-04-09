/**
 * UI state Zustand store.
 *
 * Manages sidebar collapsed state, density mode, and per-view table
 * persistence (column order, visibility, sizing, sorting). Persisted
 * to localStorage so UI preferences survive app restarts. Hydrated
 * from localStorage at module load time.
 */

import type { SortingState } from "@tanstack/react-table";
import { create } from "zustand";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type Density = "compact" | "default" | "comfortable";

export interface TablePersistedState {
  columnOrder: string[];
  columnVisibility: Record<string, boolean>;
  columnSizing: Record<string, number>;
  sorting: SortingState;
}

interface UiState {
  sidebarCollapsed: boolean;
  sidebarWidth: number;
  density: Density;
  tableStates: Record<string, TablePersistedState>;
}

interface UiActions {
  toggleSidebar: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  setSidebarWidth: (width: number) => void;
  setDensity: (density: Density) => void;
  setTableState: (viewId: string, state: Partial<TablePersistedState>) => void;
  getTableState: (viewId: string) => TablePersistedState | undefined;
}

type UiStore = UiState & UiActions;

// ---------------------------------------------------------------------------
// Persistence helpers
// ---------------------------------------------------------------------------

const STORAGE_KEY = "oxide-seo-ui";

const VALID_DENSITIES = new Set<string>(["compact", "default", "comfortable"]);

/** Load and validate persisted UI state from localStorage. */
function loadState(): Partial<UiState> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed: unknown = JSON.parse(raw);
    if (typeof parsed !== "object" || parsed === null) return {};

    const obj = parsed as Record<string, unknown>;
    const result: Partial<UiState> = {};

    if (typeof obj.sidebarCollapsed === "boolean") {
      result.sidebarCollapsed = obj.sidebarCollapsed;
    }
    if (typeof obj.sidebarWidth === "number" && obj.sidebarWidth > 0) {
      result.sidebarWidth = obj.sidebarWidth;
    }
    if (typeof obj.density === "string" && VALID_DENSITIES.has(obj.density)) {
      result.density = obj.density as Density;
    }
    if (
      typeof obj.tableStates === "object" &&
      obj.tableStates !== null &&
      !Array.isArray(obj.tableStates)
    ) {
      result.tableStates = obj.tableStates as Record<string, TablePersistedState>;
    }

    return result;
  } catch {
    // Corrupted storage -- fall through to defaults.
  }
  return {};
}

function saveState(state: UiState) {
  try {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        sidebarCollapsed: state.sidebarCollapsed,
        sidebarWidth: state.sidebarWidth,
        density: state.density,
        tableStates: state.tableStates,
      }),
    );
  } catch {
    // Storage full or unavailable -- ignore.
  }
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

const DEFAULTS: UiState = {
  sidebarCollapsed: false,
  sidebarWidth: 224,
  density: "default",
  tableStates: {},
};

const persisted = loadState();

export const useUiStore = create<UiStore>((set, get) => ({
  ...DEFAULTS,
  ...persisted,

  toggleSidebar: () =>
    set((s) => {
      const next = { sidebarCollapsed: !s.sidebarCollapsed };
      saveState({ ...s, ...next });
      return next;
    }),

  setSidebarCollapsed: (collapsed) =>
    set((s) => {
      saveState({ ...s, sidebarCollapsed: collapsed });
      return { sidebarCollapsed: collapsed };
    }),

  setSidebarWidth: (width) =>
    set((s) => {
      saveState({ ...s, sidebarWidth: width });
      return { sidebarWidth: width };
    }),

  setDensity: (density) =>
    set((s) => {
      saveState({ ...s, density });
      return { density };
    }),

  setTableState: (viewId, partial) =>
    set((s) => {
      const existing = s.tableStates[viewId] ?? {
        columnOrder: [],
        columnVisibility: {},
        columnSizing: {},
        sorting: [],
      };
      const updated = { ...existing, ...partial };
      const tableStates = { ...s.tableStates, [viewId]: updated };
      saveState({ ...s, tableStates });
      return { tableStates };
    }),

  getTableState: (viewId) => get().tableStates[viewId],
}));

// ---------------------------------------------------------------------------
// Density -> row height mapping
// ---------------------------------------------------------------------------

export const DENSITY_ROW_HEIGHT: Record<Density, number> = {
  compact: 24,
  default: 28,
  comfortable: 32,
};
