/**
 * Single keyboard key cap display.
 *
 * Renders a styled <kbd> element for displaying individual keys
 * in shortcut hints and the command palette.
 */

import { cn } from "@/lib/utils";

interface KbdProps {
  children: string;
  className?: string;
}

export function Kbd({ children, className }: KbdProps) {
  return (
    <kbd
      className={cn(
        "border-border-default bg-bg-muted text-fg-muted inline-flex h-5 min-w-5 items-center justify-center rounded-[var(--radius-xs)] border px-1.5 font-mono text-[0.6875rem] font-medium",
        className,
      )}
    >
      {children}
    </kbd>
  );
}
