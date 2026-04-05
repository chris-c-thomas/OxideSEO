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

interface ImageFilterBarProps {
  filters: LinkFilters;
  onChange: (filters: LinkFilters) => void;
}

const ALT_OPTIONS = [
  { label: "All Images", value: "all" },
  { label: "Missing Alt Text", value: "missing" },
];

const SCOPE_OPTIONS = [
  { label: "All Scope", value: "all" },
  { label: "Internal", value: "internal" },
  { label: "External", value: "external" },
];

export function ImageFilterBar({ filters, onChange }: ImageFilterBarProps) {
  const hasFilters = filters.anchorTextMissing || filters.isInternal !== null;

  const clearAll = () => {
    onChange({
      linkType: "img", // keep pre-applied image filter
      isInternal: null,
      isBroken: null,
      anchorTextMissing: null,
    });
  };

  return (
    <div className="flex items-center gap-2 pb-4">
      <Select
        value={filters.anchorTextMissing ? "missing" : "all"}
        onValueChange={(val) =>
          onChange({
            ...filters,
            anchorTextMissing: val === "missing" ? true : null,
          })
        }
      >
        <SelectTrigger className="h-8 w-44 text-sm">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {ALT_OPTIONS.map((opt) => (
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

      {hasFilters && (
        <Button variant="ghost" size="sm" className="h-8 px-2" onClick={clearAll}>
          <X className="mr-1 h-3 w-3" />
          Clear
        </Button>
      )}
    </div>
  );
}
