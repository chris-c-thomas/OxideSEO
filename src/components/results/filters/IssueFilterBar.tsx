import type { IssueFilters, RuleCategory, Severity } from "@/types";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { X } from "lucide-react";

interface IssueFilterBarProps {
  filters: IssueFilters;
  onChange: (filters: IssueFilters) => void;
}

const SEVERITY_OPTIONS: { label: string; value: Severity | "all" }[] = [
  { label: "All Severities", value: "all" },
  { label: "Error", value: "error" },
  { label: "Warning", value: "warning" },
  { label: "Info", value: "info" },
];

const CATEGORY_OPTIONS: { label: string; value: RuleCategory | "all" }[] = [
  { label: "All Categories", value: "all" },
  { label: "Meta", value: "meta" },
  { label: "Content", value: "content" },
  { label: "Links", value: "links" },
  { label: "Images", value: "images" },
  { label: "Performance", value: "performance" },
  { label: "Security", value: "security" },
];

export function IssueFilterBar({ filters, onChange }: IssueFilterBarProps) {
  const hasFilters = filters.severity || filters.category || filters.ruleId;

  const clearAll = () => {
    onChange({ severity: null, category: null, ruleId: null });
  };

  return (
    <div className="flex items-center gap-2 pb-4">
      <Select
        value={filters.severity ?? "all"}
        onValueChange={(val) =>
          onChange({ ...filters, severity: val === "all" ? null : (val as Severity) })
        }
      >
        <SelectTrigger className="h-8 w-36 text-sm">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {SEVERITY_OPTIONS.map((opt) => (
            <SelectItem key={opt.value} value={opt.value}>
              {opt.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      <Select
        value={filters.category ?? "all"}
        onValueChange={(val) =>
          onChange({ ...filters, category: val === "all" ? null : (val as RuleCategory) })
        }
      >
        <SelectTrigger className="h-8 w-36 text-sm">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {CATEGORY_OPTIONS.map((opt) => (
            <SelectItem key={opt.value} value={opt.value}>
              {opt.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      {hasFilters && (
        <Button variant="ghost" size="sm" className="h-8 px-2" onClick={clearAll}>
          <X className="mr-1 h-3 w-3" />
          Clear
        </Button>
      )}
    </div>
  );
}
