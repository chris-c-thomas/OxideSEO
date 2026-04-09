/**
 * Multi-select faceted filter chip.
 *
 * Renders a button that opens a popover with searchable checkboxes.
 * Shows count of selected values. Used for filtering by status code,
 * content type, severity, etc.
 */

import { useState } from "react";
import { Check, ListFilter, X } from "lucide-react";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { Separator } from "@/components/ui/separator";

interface FacetedFilterOption {
  label: string;
  value: string;
  count?: number;
}

interface FacetedFilterProps {
  title: string;
  options: FacetedFilterOption[];
  selected: Set<string>;
  onSelectionChange: (selected: Set<string>) => void;
}

export function FacetedFilter({
  title,
  options,
  selected,
  onSelectionChange,
}: FacetedFilterProps) {
  const [open, setOpen] = useState(false);

  const toggleValue = (value: string) => {
    const next = new Set(selected);
    if (next.has(value)) {
      next.delete(value);
    } else {
      next.add(value);
    }
    onSelectionChange(next);
  };

  const clearAll = () => {
    onSelectionChange(new Set());
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button variant="outline" size="sm" className="h-7 gap-1 border-dashed text-xs">
          <ListFilter className="size-3.5" strokeWidth={1.75} />
          {title}
          {selected.size > 0 && (
            <>
              <Separator orientation="vertical" className="mx-1 h-4" />
              <Badge variant="secondary" className="px-1 text-[0.625rem]">
                {selected.size}
              </Badge>
            </>
          )}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[200px] p-0" align="start">
        <Command>
          <CommandInput placeholder={`Filter ${title.toLowerCase()}...`} />
          <CommandList>
            <CommandEmpty>No options found.</CommandEmpty>
            <CommandGroup>
              {options.map((option) => {
                const isSelected = selected.has(option.value);
                return (
                  <CommandItem
                    key={option.value}
                    value={option.value}
                    onSelect={() => toggleValue(option.value)}
                  >
                    <div
                      className={cn(
                        "border-border-default flex size-4 items-center justify-center rounded-[var(--radius-xs)] border",
                        isSelected && "border-brand bg-brand text-fg-on-accent",
                      )}
                    >
                      {isSelected && <Check className="size-3" strokeWidth={2} />}
                    </div>
                    <span className="flex-1 truncate text-xs">{option.label}</span>
                    {option.count != null && (
                      <span className="text-fg-subtle text-[0.625rem] tabular-nums">
                        {option.count}
                      </span>
                    )}
                  </CommandItem>
                );
              })}
            </CommandGroup>
          </CommandList>
          {selected.size > 0 && (
            <>
              <Separator />
              <div className="p-1">
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-7 w-full justify-center text-xs"
                  onClick={clearAll}
                >
                  <X className="size-3" strokeWidth={1.75} />
                  Clear filters
                </Button>
              </div>
            </>
          )}
        </Command>
      </PopoverContent>
    </Popover>
  );
}
