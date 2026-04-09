/**
 * Dashboard KPI tile.
 *
 * Displays a large numeric value with label, optional delta indicator,
 * and sparkline slot. Uses tabular-nums for aligned number rendering.
 */

import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

interface MetricCardProps {
  label: string;
  value: string | number;
  delta?: {
    value: number;
    trend: "up" | "down" | "flat";
  };
  sparkline?: ReactNode;
  icon?: LucideIcon;
  className?: string;
}

export function MetricCard({
  label,
  value,
  delta,
  sparkline,
  icon: Icon,
  className,
}: MetricCardProps) {
  return (
    <div
      className={cn(
        "border-border-default bg-bg-surface rounded-[var(--radius-md)] border p-4",
        className,
      )}
    >
      <div className="flex items-center justify-between">
        <span className="text-fg-muted text-xs font-medium">{label}</span>
        {Icon && <Icon className="text-fg-subtle size-4" strokeWidth={1.75} />}
      </div>
      <div className="mt-2 flex items-baseline gap-2">
        <span className="text-fg-default text-2xl font-semibold tabular-nums">
          {typeof value === "number" ? value.toLocaleString() : value}
        </span>
        {delta && (
          <span
            className={cn(
              "text-xs font-medium tabular-nums",
              delta.trend === "up" && "text-success",
              delta.trend === "down" && "text-danger",
              delta.trend === "flat" && "text-fg-subtle",
            )}
          >
            {delta.trend === "up" && "+"}
            {delta.trend === "down" && ""}
            {delta.value}%
          </span>
        )}
      </div>
      {sparkline && <div className="mt-3">{sparkline}</div>}
    </div>
  );
}
