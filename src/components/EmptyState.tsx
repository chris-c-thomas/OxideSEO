/**
 * Empty state pattern component.
 *
 * Displays an icon, title, description, and optional call-to-action
 * for views with no data. Illustration-free.
 */

import type { LucideIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface EmptyStateProps {
  icon: LucideIcon;
  title: string;
  description: string;
  action?: {
    label: string;
    onClick: () => void;
  };
  className?: string;
}

export function EmptyState({
  icon: Icon,
  title,
  description,
  action,
  className,
}: EmptyStateProps) {
  return (
    <div
      className={cn("flex flex-col items-center justify-center gap-4 py-16", className)}
    >
      <div className="bg-bg-muted rounded-[var(--radius-lg)] p-3">
        <Icon className="text-fg-subtle size-6" strokeWidth={1.75} />
      </div>
      <div className="flex flex-col items-center gap-1 text-center">
        <h3 className="text-fg-default text-sm font-medium">{title}</h3>
        <p className="text-fg-muted max-w-[280px] text-xs">{description}</p>
      </div>
      {action && (
        <Button size="sm" onClick={action.onClick}>
          {action.label}
        </Button>
      )}
    </div>
  );
}
