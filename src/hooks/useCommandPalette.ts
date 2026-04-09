/**
 * Command palette state hook.
 *
 * Manages open/close state for the CommandPalette dialog.
 * Used by both the palette component and the global hotkey handler.
 */

import { useCallback, useSyncExternalStore } from "react";
import { commandRegistry, type PaletteCommand } from "@/lib/commandRegistry";

// Global palette state (singleton -- only one palette in the app).
let isOpen = false;
const listeners = new Set<() => void>();

function notify() {
  for (const listener of listeners) {
    listener();
  }
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function getSnapshot() {
  return isOpen;
}

export function openCommandPalette() {
  isOpen = true;
  notify();
}

export function closeCommandPalette() {
  isOpen = false;
  notify();
}

export function toggleCommandPalette() {
  isOpen = !isOpen;
  notify();
}

export function useCommandPalette() {
  const open = useSyncExternalStore(subscribe, getSnapshot);

  const commands = useSyncExternalStore(
    (cb) => commandRegistry.subscribe(cb),
    () => commandRegistry.getAll(),
  );

  const setOpen = useCallback((value: boolean) => {
    isOpen = value;
    notify();
  }, []);

  return { isOpen: open, setOpen, commands };
}

/** Hook for views to register their commands on mount. */
export function useRegisterCommands(commands: PaletteCommand[]) {
  // Register on first render, unregister on unmount.
  // Using useSyncExternalStore pattern to avoid stale closures.
  const ids = commands.map((c) => c.id);

  // We register in a layout-effect-like pattern but use a ref approach
  // to avoid re-registering on every render.
  useSyncExternalStore(
    () => {
      commandRegistry.registerMany(commands);
      return () => commandRegistry.unregisterMany(ids);
    },
    () => null,
  );
}
