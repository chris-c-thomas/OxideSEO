import type { LinkFilters } from "@/types";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { X } from "lucide-react";

interface LinkFilterBarProps {
  filters: LinkFilters;
  onChange: (filters: LinkFilters) => void;
}

const TYPE_OPTIONS = [
  { label: "All Types", value: "all" },
  { label: "Anchor", value: "a" },
  { label: "Image", value: "img" },
  { label: "Script", value: "script" },
  { label: "Stylesheet", value: "link" },
  { label: "Canonical", value: "canonical" },
  { label: "Redirect", value: "redirect" },
];

const SCOPE_OPTIONS = [
  { label: "All Scope", value: "all" },
  { label: "Internal", value: "internal" },
  { label: "External", value: "external" },
];

const BROKEN_OPTIONS = [
  { label: "All Status", value: "all" },
  { label: "Broken Only", value: "broken" },
  { label: "Working Only", value: "working" },
];

export function LinkFilterBar({ filters, onChange }: LinkFilterBarProps) {
  const hasFilters =
    filters.linkType || filters.isInternal !== null || filters.isBroken !== null;

  const clearAll = () => {
    onChange({
      linkType: null,
      isInternal: null,
      isBroken: null,
      anchorTextMissing: null,
    });
  };

  return (
    <div className="flex items-center gap-2 pb-4">
      <Select
        value={filters.linkType ?? "all"}
        onValueChange={(val) =>
          onChange({ ...filters, linkType: val === "all" ? null : val })
        }
      >
        <SelectTrigger className="h-8 w-36 text-sm">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {TYPE_OPTIONS.map((opt) => (
            <SelectItem key={opt.value} value={opt.value}>
              {opt.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      <Select
        value={
          filters.isInternal === true
            ? "internal"
            : filters.isInternal === false
              ? "external"
              : "all"
        }
        onValueChange={(val) =>
          onChange({
            ...filters,
            isInternal: val === "internal" ? true : val === "external" ? false : null,
          })
        }
      >
        <SelectTrigger className="h-8 w-32 text-sm">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {SCOPE_OPTIONS.map((opt) => (
            <SelectItem key={opt.value} value={opt.value}>
              {opt.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      <Select
        value={
          filters.isBroken === true
            ? "broken"
            : filters.isBroken === false
              ? "working"
              : "all"
        }
        onValueChange={(val) =>
          onChange({
            ...filters,
            isBroken: val === "broken" ? true : val === "working" ? false : null,
          })
        }
      >
        <SelectTrigger className="h-8 w-36 text-sm">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {BROKEN_OPTIONS.map((opt) => (
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
