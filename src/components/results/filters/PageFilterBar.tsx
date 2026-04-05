import { useEffect, useRef, useState } from "react";
import type { PageFilters } from "@/types";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { X } from "lucide-react";

interface PageFilterBarProps {
  filters: PageFilters;
  onChange: (filters: PageFilters) => void;
}

const STATUS_OPTIONS = [
  { label: "All Statuses", value: "all" },
  { label: "2xx Success", value: "2xx" },
  { label: "3xx Redirect", value: "3xx" },
  { label: "4xx Client Error", value: "4xx" },
  { label: "5xx Server Error", value: "5xx" },
];

const TYPE_OPTIONS = [
  { label: "All Types", value: "all" },
  { label: "HTML", value: "text/html" },
  { label: "CSS", value: "text/css" },
  { label: "JavaScript", value: "application/javascript" },
  { label: "Image", value: "image/" },
];

function statusValueToCodes(value: string): number[] | null {
  switch (value) {
    case "2xx":
      return [200, 201, 202, 203, 204];
    case "3xx":
      return [300, 301, 302, 303, 304, 307, 308];
    case "4xx":
      return [400, 401, 403, 404, 405, 410, 429, 451];
    case "5xx":
      return [500, 502, 503, 504];
    default:
      return null;
  }
}

function codesToStatusValue(codes: number[] | null): string {
  if (!codes || codes.length === 0) return "all";
  const first = codes[0];
  if (first === undefined) return "all";
  if (first >= 200 && first < 300) return "2xx";
  if (first >= 300 && first < 400) return "3xx";
  if (first >= 400 && first < 500) return "4xx";
  if (first >= 500) return "5xx";
  return "all";
}

export function PageFilterBar({ filters, onChange }: PageFilterBarProps) {
  const [searchText, setSearchText] = useState(filters.urlSearch ?? "");
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => {
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, []);

  const handleSearchChange = (value: string) => {
    setSearchText(value);
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      onChange({ ...filters, urlSearch: value || null });
    }, 300);
  };

  const hasFilters = filters.urlSearch || filters.statusCodes || filters.contentType;

  const clearAll = () => {
    setSearchText("");
    onChange({
      urlSearch: null,
      statusCodes: null,
      contentType: null,
    });
  };

  return (
    <div className="flex items-center gap-2 pb-4">
      <Input
        placeholder="Search URLs..."
        value={searchText}
        onChange={(e) => handleSearchChange(e.target.value)}
        className="h-8 w-64 text-sm"
      />

      <Select
        value={codesToStatusValue(filters.statusCodes)}
        onValueChange={(val) =>
          onChange({ ...filters, statusCodes: statusValueToCodes(val) })
        }
      >
        <SelectTrigger className="h-8 w-36 text-sm">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {STATUS_OPTIONS.map((opt) => (
            <SelectItem key={opt.value} value={opt.value}>
              {opt.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      <Select
        value={filters.contentType ?? "all"}
        onValueChange={(val) =>
          onChange({ ...filters, contentType: val === "all" ? null : val })
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

      {hasFilters && (
        <Button variant="ghost" size="sm" className="h-8 px-2" onClick={clearAll}>
          <X className="mr-1 h-3 w-3" />
          Clear
        </Button>
      )}
    </div>
  );
}
