/**
 * Command palette state hook.
 *
 * Manages open/close state for the CommandPalette dialog.
 * Used by both the palette component and the global hotkey handler.
 */

import { useCallback, useEffect, useRef, useState, useSyncExternalStore } from "react";
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

  // Use useState for commands to avoid the infinite loop from useSyncExternalStore
  // returning a new array reference on every call.
  const [commands, setCommands] = useState<PaletteCommand[]>(() =>
    commandRegistry.getAll(),
  );

  useEffect(() => {
    return commandRegistry.subscribe(() => {
      setCommands(commandRegistry.getAll());
    });
  }, []);

  const setOpen = useCallback((value: boolean) => {
    isOpen = value;
    notify();
  }, []);

  return { isOpen: open, setOpen, commands };
}

/** Hook for views to register their commands on mount. */
export function useRegisterCommands(commands: PaletteCommand[]) {
  const registered = useRef(false);
  const ids = useRef(commands.map((c) => c.id));

  useEffect(() => {
    const currentIds = ids.current;
    if (!registered.current) {
      commandRegistry.registerMany(commands);
      registered.current = true;
    }
    return () => {
      commandRegistry.unregisterMany(currentIds);
      registered.current = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
}
