/**
 * Lightweight keyboard shortcut hook.
 *
 * Registers keydown handlers on document. Skips when focus is inside
 * input/textarea/contenteditable (except for Escape). Platform-aware:
 * uses metaKey on macOS, ctrlKey on Windows/Linux.
 */

import { useEffect, useRef } from "react";
import type { ShortcutDef } from "@/lib/shortcuts";

const IS_MAC =
  typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.userAgent);

type HotkeyMap = Record<string, { shortcut: ShortcutDef; handler: () => void }>;

function isInputFocused(): boolean {
  const el = document.activeElement;
  if (!el) return false;
  const tag = el.tagName;
  if (tag === "INPUT" || tag === "TEXTAREA") return true;
  if ((el as HTMLElement).isContentEditable) return true;
  return false;
}

function matchesShortcut(e: KeyboardEvent, def: ShortcutDef): boolean {
  const modKey = IS_MAC ? e.metaKey : e.ctrlKey;

  if (def.mod && !modKey) return false;
  if (!def.mod && modKey) return false;
  if (def.shift && !e.shiftKey) return false;
  if (def.alt && !e.altKey) return false;

  return e.key.toLowerCase() === def.key.toLowerCase();
}

export function useHotkeys(hotkeys: HotkeyMap) {
  const hotkeysRef = useRef(hotkeys);
  hotkeysRef.current = hotkeys;

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const entries = Object.values(hotkeysRef.current);

      for (const { shortcut, handler } of entries) {
        if (!matchesShortcut(e, shortcut)) continue;

        // Allow Escape always; skip others when in inputs.
        if (shortcut.key !== "Escape" && isInputFocused()) continue;

        e.preventDefault();
        e.stopPropagation();
        handler();
        return;
      }
    }

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, []);
}
