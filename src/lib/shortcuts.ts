/**
 * Global keyboard shortcut definitions.
 *
 * "Mod" is mapped to Cmd on macOS and Ctrl on Windows/Linux
 * by the useHotkeys hook.
 */

export interface ShortcutDef {
  key: string;
  mod?: boolean;
  shift?: boolean;
  alt?: boolean;
}

export const SHORTCUTS = {
  commandPalette: { key: "k", mod: true },
  settings: { key: ",", mod: true },
  newCrawl: { key: "n", mod: true },
  focusSearch: { key: "f", mod: true },
  exportView: { key: "e", mod: true },
  switchView1: { key: "1", mod: true },
  switchView2: { key: "2", mod: true },
  switchView3: { key: "3", mod: true },
  switchView4: { key: "4", mod: true },
  switchView5: { key: "5", mod: true },
  closeOverlay: { key: "Escape" },
  focusGlobalSearch: { key: "/" },
  nextRow: { key: "j" },
  prevRow: { key: "k" },
} as const satisfies Record<string, ShortcutDef>;

/** Human-readable shortcut labels for display in tooltips and palette. */
export function shortcutToKeys(def: ShortcutDef): string[] {
  const keys: string[] = [];
  if (def.mod) keys.push("Mod");
  if (def.shift) keys.push("Shift");
  if (def.alt) keys.push("Alt");
  keys.push(def.key.length === 1 ? def.key.toUpperCase() : def.key);
  return keys;
}
