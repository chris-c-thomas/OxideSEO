/**
 * Severity badge component.
 *
 * Renders a colored pill indicating issue severity using the
 * --sev-* design tokens. Used in issue tables and detail views.
 */

import { cn } from "@/lib/utils";

type Severity = "critical" | "high" | "medium" | "low" | "info";

interface SeverityBadgeProps {
  severity: Severity;
  className?: string;
}

const SEVERITY_STYLES: Record<Severity, string> = {
  critical: "bg-sev-critical/10 text-sev-critical",
  high: "bg-sev-high/10 text-sev-high",
  medium: "bg-sev-medium/10 text-sev-medium",
  low: "bg-sev-low/10 text-sev-low",
  info: "bg-bg-muted text-sev-info",
};

const SEVERITY_LABELS: Record<Severity, string> = {
  critical: "Critical",
  high: "High",
  medium: "Medium",
  low: "Low",
  info: "Info",
};

export function SeverityBadge({ severity, className }: SeverityBadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-[var(--radius-sm)] px-1.5 py-0.5 text-[0.6875rem] font-medium",
        SEVERITY_STYLES[severity],
        className,
      )}
      aria-label={`Severity: ${SEVERITY_LABELS[severity]}`}
    >
      {SEVERITY_LABELS[severity]}
    </span>
  );
}
