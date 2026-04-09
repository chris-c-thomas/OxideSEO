/**
 * HTTP status code badge.
 *
 * Renders a colored chip with mono/tabular-nums styling.
 * Color is determined by status code range (2xx/3xx/4xx/5xx).
 */

import { cn } from "@/lib/utils";

interface StatusCodeBadgeProps {
  code: number;
  className?: string;
}

function getStatusColor(code: number): string {
  if (code >= 200 && code < 300) return "bg-success/10 text-success";
  if (code >= 300 && code < 400) return "bg-info/10 text-info";
  if (code >= 400 && code < 500) return "bg-warning/10 text-warning";
  if (code >= 500) return "bg-danger/10 text-danger";
  return "bg-bg-muted text-fg-subtle";
}

export function StatusCodeBadge({ code, className }: StatusCodeBadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-[var(--radius-sm)] px-1.5 py-0.5 font-mono text-[0.6875rem] font-medium tabular-nums",
        getStatusColor(code),
        className,
      )}
    >
      {code}
    </span>
  );
}
