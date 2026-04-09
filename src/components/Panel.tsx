/**
 * Titled section container with optional actions slot.
 *
 * Used for grouping related content in dashboards and detail views.
 * Thin border, no shadow by default.
 */

import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

interface PanelProps {
  title: string;
  actions?: ReactNode;
  children: ReactNode;
  className?: string;
}

export function Panel({ title, actions, children, className }: PanelProps) {
  return (
    <div
      className={cn(
        "border-border-default bg-bg-surface rounded-[var(--radius-md)] border",
        className,
      )}
    >
      <div className="border-border-subtle flex items-center justify-between border-b px-4 py-3">
        <h3 className="text-fg-default text-sm font-semibold">{title}</h3>
        {actions && <div className="flex items-center gap-2">{actions}</div>}
      </div>
      <div className="p-4">{children}</div>
    </div>
  );
}
