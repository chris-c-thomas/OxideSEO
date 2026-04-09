/**
 * Multi-key shortcut display.
 *
 * Renders a sequence of Kbd components separated by spaces.
 * Automatically maps modifier keys to platform-appropriate labels
 * (e.g., "Cmd" on macOS, "Ctrl" on Windows/Linux).
 */

import { Kbd } from "@/components/Kbd";
import { cn } from "@/lib/utils";

interface KeyboardHintProps {
  keys: string[];
  className?: string;
}

const IS_MAC =
  typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.userAgent);

const MODIFIER_MAP: Record<string, string> = IS_MAC
  ? { Mod: "\u2318", Shift: "\u21E7", Alt: "\u2325", Ctrl: "\u2303" }
  : { Mod: "Ctrl", Shift: "Shift", Alt: "Alt", Ctrl: "Ctrl" };

function mapKey(key: string): string {
  return MODIFIER_MAP[key] ?? key;
}

export function KeyboardHint({ keys, className }: KeyboardHintProps) {
  return (
    <span className={cn("inline-flex items-center gap-0.5", className)}>
      {keys.map((key, i) => (
        <Kbd key={i}>{mapKey(key)}</Kbd>
      ))}
    </span>
  );
}
