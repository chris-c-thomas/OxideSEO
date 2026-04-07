/**
 * Individual plugin card showing name, version, kind, and enable/disable toggle.
 */

import { cn } from "@/lib/utils";
import type { PluginInfo } from "@/types";
import { Badge } from "@/components/ui/badge";

interface PluginCardProps {
  plugin: PluginInfo;
  onToggle: (name: string, enabled: boolean) => void;
  onClick: (name: string) => void;
}

const KIND_LABELS: Record<string, string> = {
  rule: "Rule",
  exporter: "Exporter",
  post_processor: "Post-Processor",
  ui_extension: "UI Extension",
};

export function PluginCard({ plugin, onToggle, onClick }: PluginCardProps) {
  return (
    <div
      className={cn(
        "flex cursor-pointer flex-col gap-2 rounded-lg border p-4 transition-colors hover:bg-[var(--color-muted)]",
        plugin.enabled
          ? "border-[var(--color-primary)]/30"
          : "border-[var(--color-border)]",
      )}
      onClick={() => onClick(plugin.name)}
      onKeyDown={(e) => {
        if (e.key === "Enter") onClick(plugin.name);
      }}
      role="button"
      tabIndex={0}
    >
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold">{plugin.name}</h3>
        <button
          className={cn(
            "relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors",
            plugin.enabled ? "bg-[var(--color-primary)]" : "bg-[var(--color-muted)]",
          )}
          role="switch"
          aria-checked={plugin.enabled}
          onClick={(e) => {
            e.stopPropagation();
            onToggle(plugin.name, !plugin.enabled);
          }}
        >
          <span
            className={cn(
              "pointer-events-none inline-block h-4 w-4 rounded-full bg-white shadow-sm transition-transform",
              plugin.enabled ? "translate-x-4" : "translate-x-0",
            )}
          />
        </button>
      </div>

      <p className="text-xs text-[var(--color-muted-foreground)]">{plugin.description}</p>

      <div className="flex items-center gap-2">
        <Badge variant="outline" className="text-xs">
          {KIND_LABELS[plugin.kind] ?? plugin.kind}
        </Badge>
        <span className="text-xs text-[var(--color-muted-foreground)]">
          v{plugin.version}
        </span>
        {plugin.isNative && (
          <Badge variant="destructive" className="text-xs">
            Native
          </Badge>
        )}
      </div>

      {plugin.loadError && (
        <p className="text-xs text-red-600 dark:text-red-400">{plugin.loadError}</p>
      )}
    </div>
  );
}
